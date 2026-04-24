use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

// @spec TINFRA-020
#[derive(Deserialize, Clone)]
pub(crate) struct Manifest {
    pub(crate) scenario: Vec<Scenario>,
}

// @spec TINFRA-020, TINFRA-021
#[derive(Deserialize, Clone)]
pub(crate) struct Scenario {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) script: String,
    pub(crate) input: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) output: String,
    #[serde(default)]
    pub(crate) exit_code: i32,
    pub(crate) invoke: Vec<String>,
    #[serde(default)]
    pub(crate) env: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) mock: Vec<MockDecl>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct MockDecl {
    pub(crate) command: String,
    pub(crate) input: Option<String>,
    pub(crate) stdout: Option<String>,
    pub(crate) stderr: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) expected_args: Option<Vec<String>>,
}

// @spec TINFRA-010, TINFRA-011, TINFRA-012
pub(crate) fn discover_manifests(tests_dir: &Path) -> Result<Vec<(String, Manifest)>, Vec<String>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();

    walk_for_manifests(tests_dir, tests_dir, 0, &mut results, &mut errors);

    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}

fn walk_for_manifests(
    base_dir: &Path,
    current_dir: &Path,
    depth: usize,
    results: &mut Vec<(String, Manifest)>,
    errors: &mut Vec<String>,
) {
    if depth > 2 {
        return;
    }

    let entries = match std::fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(format!(
                "failed to read directory {}: {e}",
                current_dir.display()
            ));
            return;
        }
    };

    let mut subdirs = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!(
                    "failed to read entry in {}: {e}",
                    current_dir.display()
                ));
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            subdirs.push(path);
        } else if path.file_name().map_or(false, |n| n == "manifest.toml") {
            let suite_name = current_dir
                .strip_prefix(base_dir)
                .expect("current_dir is always under base_dir")
                .to_string_lossy()
                .replace('\\', "/");

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("failed to read {}: {e}", path.display()));
                    continue;
                }
            };

            match toml::from_str::<Manifest>(&content) {
                Ok(manifest) => results.push((suite_name, manifest)),
                Err(e) => errors.push(format!("failed to parse {}: {e}", path.display())),
            }
        }
    }

    for subdir in subdirs {
        walk_for_manifests(base_dir, &subdir, depth + 1, results, errors);
    }
}
