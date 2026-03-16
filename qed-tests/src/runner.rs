use crate::manifest::Scenario;
use crate::scenario;
use std::path::Path;
use std::process::Command;

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
