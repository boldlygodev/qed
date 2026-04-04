//! The `qed:file()` processor — materializes selected text to a temporary file
//! and injects `${QED_FILE}` into the downstream external command's environment.
//!
//! At compile time, `qed:file()` is fused with the next external command in the
//! chain to create a [`FileHandoffProcessor`]. The external command's arguments
//! are kept unexpanded so `${QED_FILE}` references can be resolved at runtime
//! with the actual temp file path. When no external command follows, a
//! [`FileMarker`] passthrough remains that warns on empty input.

use std::io::Write;

use super::{Processor, ProcessorError};

/// Compile-time sentinel for `qed:file()`. Passes input through unchanged.
///
/// Remains in the chain when `qed:file()` is not followed by an external
/// command. Emits a warning when the input region is empty (insertion point).
#[derive(Debug)]
pub(crate) struct FileMarker {
    pub(crate) span: crate::span::Span,
}

impl Processor for FileMarker {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        if input.is_empty() {
            return Err(ProcessorError::FileEmptyRegion { span: self.span });
        }
        Ok(input.to_owned())
    }

    fn is_file_marker(&self) -> bool {
        true
    }
}

/// Wraps an external command, writing the input to a temp file and resolving
/// `${QED_FILE}` in the command arguments before spawning.
#[derive(Debug)]
pub(crate) struct FileHandoffProcessor {
    pub(crate) command: String,
    /// Raw argument strings — `${QED_FILE}` is NOT expanded at compile time;
    /// it is substituted at runtime with the actual temp file path.
    pub(crate) raw_args: Vec<String>,
}

impl Processor for FileHandoffProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        use std::process::{Command, Stdio};

        // Write input to a temp file
        let mut tmp =
            tempfile::NamedTempFile::new().map_err(|e| ProcessorError::ProcessorFailed {
                processor: "qed:file".into(),
                reason: format!("failed to create temp file: {e}"),
            })?;
        tmp.write_all(input.as_bytes())
            .map_err(|e| ProcessorError::ProcessorFailed {
                processor: "qed:file".into(),
                reason: format!("failed to write temp file: {e}"),
            })?;
        let tmp_path = tmp.into_temp_path();
        let tmp_path_str = tmp_path.to_string_lossy();

        // Resolve ${QED_FILE} in args at runtime
        let resolved_args: Vec<String> = self
            .raw_args
            .iter()
            .map(|arg| arg.replace("${QED_FILE}", &tmp_path_str))
            .collect();

        // Spawn the external command with QED_FILE also set in env
        let mut child = Command::new(&self.command)
            .args(&resolved_args)
            .env("QED_FILE", tmp_path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProcessorError::ExternalFailed {
                command: self.command.clone(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input.as_bytes());
        }

        let output = child
            .wait_with_output()
            .map_err(|e| ProcessorError::ExternalFailed {
                command: self.command.clone(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

        // Clean up temp file (drop triggers removal)
        drop(tmp_path);

        if !output.status.success() {
            return Err(ProcessorError::ExternalFailed {
                command: self.command.clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        let mut result = String::from_utf8_lossy(&output.stdout).into_owned();

        if input.ends_with('\n') && !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }
}
