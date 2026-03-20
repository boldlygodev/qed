//! External command processor — spawns a subprocess, pipes selected text
//! through stdin, captures stdout.

use std::io::Write;
use std::process::{Command, Stdio};

use super::{Processor, ProcessorError};

/// Runs an external command as a processor.
///
/// The selected text is written to the command's stdin.
/// The command's stdout becomes the processor output.
/// Non-zero exit codes produce a `ProcessorError::ExternalFailed`.
#[derive(Debug)]
pub(crate) struct ExternalCommandProcessor {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
}

impl Processor for ExternalCommandProcessor {
    fn execute(&self, input: &str) -> Result<String, ProcessorError> {
        let mut child = Command::new(&self.command)
            .args(&self.args)
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
            // Ignore write errors — the process may have exited early
            let _ = stdin.write_all(input.as_bytes());
        }

        let output = child.wait_with_output().map_err(|e| ProcessorError::ExternalFailed {
            command: self.command.clone(),
            exit_code: None,
            stderr: e.to_string(),
        })?;

        if !output.status.success() {
            return Err(ProcessorError::ExternalFailed {
                command: self.command.clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let mut result = String::from_utf8_lossy(&output.stdout).into_owned();

        // Preserve line structure: if the input ended with a newline (replacement
        // mode), ensure the output does too. If the input was empty (zero-width
        // insertion), use the command's output verbatim.
        if input.ends_with('\n') && !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }
}
