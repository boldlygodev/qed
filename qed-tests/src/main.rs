//! Integration test harness for qed.
//!
//! Uses [`libtest_mimic`] to run each test scenario as a named trial.
//! The Rust side handles manifest discovery, scenario assembly, and golden
//! comparison; the actual qed invocation is delegated to
//! `tests/harness/run-scenario.sh` so that tests exercise the real binary.
//!
//! Trial names follow the pattern `<suite>::<scenario-id>::<invocation-index>`.
//!
//! ```sh
//! cargo test --package qed-tests --test integration "selectors::at-literal-single-match::0"
//! ```

mod manifest;
mod runner;
mod scenario;

use libtest_mimic::{Arguments, Trial};
use std::path::PathBuf;

fn main() {
    let args = Arguments::from_args();

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("qed-tests crate must be inside the workspace")
        .to_path_buf();

    let tests_dir = workspace_root.join("tests");
    let harness_script = tests_dir.join("harness/run-scenario.sh");

    let manifests = match manifest::discover_manifests(&tests_dir) {
        Ok(m) => m,
        Err(errors) => {
            eprintln!("fatal: failed to discover test manifests:");
            for err in &errors {
                eprintln!("  {err}");
            }
            std::process::exit(1);
        }
    };

    let mut trials = Vec::new();

    for (suite_name, manifest) in &manifests {
        let suite_dir = tests_dir.join(suite_name);

        for scenario_entry in &manifest.scenario {
            for invocation_index in 0..scenario_entry.invoke.len() {
                let trial_name = format!("{suite_name}::{}::{invocation_index}", scenario_entry.id);

                let harness_script = harness_script.clone();
                let suite_dir = suite_dir.clone();
                let scenario_entry = scenario_entry.clone();
                let suite_name = suite_name.clone();

                trials.push(Trial::test(trial_name, move || {
                    runner::run_trial(
                        &harness_script,
                        &suite_dir,
                        &scenario_entry,
                        invocation_index,
                        &suite_name,
                    )
                    .map_err(|msg| msg.into())
                }));
            }
        }
    }

    libtest_mimic::run(&args, trials).exit();
}
