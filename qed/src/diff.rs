//! Unified diff generation for `--dry-run` mode.

use similar::TextDiff;

/// Produce a unified diff between `original` and `modified`.
///
/// Returns an empty string when the two inputs are identical (avoiding
/// the `--- a` / `+++ b` header that `similar` emits even for no-ops).
pub(crate) fn unified_diff(original: &str, modified: &str) -> String {
    if original == modified {
        return String::new();
    }

    TextDiff::from_lines(original, modified)
        .unified_diff()
        .header("a", "b")
        .missing_newline_hint(false)
        .to_string()
}
