use crate::manifest::Scenario;
use std::path::Path;

// @spec TINFRA-022, TINFRA-023, TINFRA-024
pub(crate) fn generate(
    suite_dir: &Path,
    scenario: &Scenario,
    invocation_index: usize,
    suite_name: &str,
) -> String {
    let mut out = String::new();
    let suite_dir = suite_dir.display();
    let scenario_id = format!("{suite_name}::{}", scenario.id);

    // Scalar fields
    out.push_str(&format!("SCENARIO_ID='{scenario_id}'\n"));
    let desc_escaped = scenario.description.replace('\'', "'\\''");
    out.push_str(&format!("SCENARIO_DESC='{desc_escaped}'\n"));
    out.push_str(&format!("SUITE_DIR='{suite_dir}'\n"));
    out.push_str(&format!(
        "SCRIPT='{suite_dir}/scripts/{}'\n",
        scenario.script
    ));
    out.push_str(&format!(
        "INPUT_SRC='{suite_dir}/inputs/{}'\n",
        scenario.input
    ));
    out.push_str(&format!(
        "STDOUT_GOLDEN='{suite_dir}/goldens/stdout/{}'\n",
        scenario.stdout
    ));
    out.push_str(&format!(
        "STDERR_GOLDEN='{suite_dir}/goldens/stderr/{}'\n",
        scenario.stderr
    ));
    out.push_str(&format!(
        "OUTPUT_GOLDEN='{suite_dir}/goldens/output/{}'\n",
        scenario.output
    ));
    out.push_str(&format!("EXPECTED_EXIT_CODE={}\n", scenario.exit_code));

    // Invocation — single-quoted with internal single quotes escaped
    let invocation = scenario.invoke[invocation_index].trim();
    let escaped = invocation.replace('\'', "'\\''");
    out.push_str(&format!("INVOCATION='{escaped}'\n"));

    // Mock declarations
    out.push_str(&format!("MOCK_COUNT={}\n", scenario.mock.len()));
    for (i, mock) in scenario.mock.iter().enumerate() {
        out.push_str(&format!("MOCK_{i}_COMMAND='{}'\n", mock.command));

        let mock_input = mock
            .input
            .as_ref()
            .map(|v| format!("{suite_dir}/mocks/input/{v}"))
            .unwrap_or_default();
        out.push_str(&format!("MOCK_{i}_INPUT='{mock_input}'\n"));

        let mock_stdout = mock
            .stdout
            .as_ref()
            .map(|v| format!("{suite_dir}/mocks/stdout/{v}"))
            .unwrap_or_default();
        out.push_str(&format!("MOCK_{i}_STDOUT='{mock_stdout}'\n"));

        let mock_stderr = mock
            .stderr
            .as_ref()
            .map(|v| format!("{suite_dir}/mocks/stderr/{v}"))
            .unwrap_or_default();
        out.push_str(&format!("MOCK_{i}_STDERR='{mock_stderr}'\n"));

        out.push_str(&format!(
            "MOCK_{i}_EXIT_CODE={}\n",
            mock.exit_code.unwrap_or(0)
        ));

        if let Some(args) = &mock.expected_args {
            out.push_str(&format!("MOCK_{i}_EXPECTED_ARGS_COUNT={}\n", args.len()));
            for (j, arg) in args.iter().enumerate() {
                // Single-quote to preserve literal ${QED_FILE} references
                out.push_str(&format!("MOCK_{i}_EXPECTED_ARG_{j}='{arg}'\n"));
            }
        } else {
            out.push_str(&format!("MOCK_{i}_EXPECTED_ARGS_COUNT=0\n"));
        }
    }

    // Extra environment variables
    for (key, value) in &scenario.env {
        out.push_str(&format!("export {key}=\"{value}\"\n"));
    }

    out
}
