use crate::compile::Script;

use super::{Buffer, Fragment, FragmentContent, fragment};

/// Execute a compiled script against a buffer, producing the output string.
pub(crate) fn execute(script: &Script, buffer: &Buffer) -> String {
    // Build requests: (StatementId, SelectorId) for fragmentation
    let requests: Vec<_> = script
        .statements
        .iter()
        .map(|s| (s.id, s.selector))
        .collect();

    let fragments = fragment::fragment(buffer, &requests, &script.selectors);

    let mut output = String::new();

    for frag in &fragments {
        match frag {
            Fragment::Passthrough(content) => {
                output.push_str(&resolve_content(content, buffer));
            }
            Fragment::Selected { content, tags } => {
                let text = resolve_content(content, buffer);

                // Find the first matching statement's processor
                let processed = tags.iter().find_map(|(stmt_id, _sel_id)| {
                    script
                        .statements
                        .iter()
                        .find(|s| s.id == *stmt_id)
                        .and_then(|stmt| stmt.processor.execute(&text).ok())
                });

                match processed {
                    Some(result) => output.push_str(&result),
                    None => output.push_str(&text), // fallback: pass through
                }
            }
        }
    }

    output
}

fn resolve_content(content: &FragmentContent, buffer: &Buffer) -> String {
    match content {
        FragmentContent::Borrowed(range) => buffer.slice(*range).to_owned(),
        FragmentContent::Owned(s) => s.clone(),
    }
}
