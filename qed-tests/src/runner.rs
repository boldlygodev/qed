use crate::manifest::Scenario;
use crate::scenario;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Locate the `qed` binary in the target directory.
fn find_qed_binary() -> PathBuf {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("qed-tests crate must be inside the workspace")
        .to_path_buf();

    // Check common target locations
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let binary = workspace_root.join("target").join(profile).join("qed");
    if binary.exists() {
        return binary;
    }

    // Fallback: try the CARGO_BIN_EXE approach via target directory
    panic!(
        "qed binary not found at {}. Build it first with `cargo build --bin qed`.",
        binary.display()
    );
}

pub(crate) fn run_trial(
    harness_script: &Path,
    suite_dir: &Path,
    scenario_entry: &Scenario,
    invocation_index: usize,
    suite_name: &str,
) -> Result<(), String> {
    let tmpdir = std::env::temp_dir().join(format!(
        "qed-test-{}-{}-{}-{}",
        suite_name,
        scenario_entry.id,
        invocation_index,
        std::process::id(),
    ));

    // Create temp directory structure
    std::fs::create_dir_all(&tmpdir).map_err(|e| format!("failed to create temp dir: {e}"))?;
    std::fs::create_dir_all(tmpdir.join("bin"))
        .map_err(|e| format!("failed to create bin dir: {e}"))?;
    std::fs::create_dir_all(tmpdir.join("mock-state"))
        .map_err(|e| format!("failed to create mock-state dir: {e}"))?;

    // Symlink the qed binary into the temp bin directory
    let qed_binary = find_qed_binary();
    let qed_link = tmpdir.join("bin/qed");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&qed_binary, &qed_link)
        .map_err(|e| format!("failed to symlink qed binary: {e}"))?;
    #[cfg(not(unix))]
    std::fs::copy(&qed_binary, &qed_link)
        .map_err(|e| format!("failed to copy qed binary: {e}"))?;

    // Generate and write scenario.sh
    let scenario_content =
        scenario::generate(suite_dir, scenario_entry, invocation_index, suite_name);
    std::fs::write(tmpdir.join("scenario.sh"), &scenario_content)
        .map_err(|e| format!("failed to write scenario.sh: {e}"))?;

    // Run the bash harness
    let output = Command::new("bash")
        .arg(harness_script)
        .arg(&tmpdir)
        .output()
        .map_err(|e| format!("failed to execute run-scenario.sh: {e}"))?;

    // Clean up unconditionally
    let _ = std::fs::remove_dir_all(&tmpdir);

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut msg = String::new();
        if !stdout.is_empty() {
            msg.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !msg.is_empty() {
                msg.push('\n');
            }
            msg.push_str(&stderr);
        }
        Err(msg)
    }
}
