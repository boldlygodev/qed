//! Environment variable expansion for pattern values and processor arguments.
//!
//! `${VAR}` references are expanded from the process environment at compile
//! time.  `\${VAR}` is the escape form — the backslash is stripped and the
//! `${VAR}` text is kept literally.  When `no_env` is `true`, all `${…}`
//! sequences are treated as literal text (no expansion).

/// A record of an unset variable encountered during expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnsetVar {
    /// The variable name that was not found in the environment.
    pub(crate) name: String,
    /// Byte offset of the `$` in the *input* string.
    pub(crate) offset: usize,
}

/// Expand `${VAR}` references in `input` from the environment.
///
/// Returns the expanded string and a list of variables that were referenced
/// but not set (each expands to the empty string).
///
/// # Expansion rules
///
/// | Input         | Output                    | Notes                          |
/// |---------------|---------------------------|--------------------------------|
/// | `${VAR}`      | value of `VAR`            | empty string + warning if unset|
/// | `\${VAR}`     | `${VAR}`                  | backslash stripped             |
/// | `$` otherwise | `$`                       | bare `$` passes through        |
/// | any when `no_env` | input unchanged        | no expansion at all            |
pub(crate) fn expand_env_vars(input: &str, no_env: bool) -> (String, Vec<UnsetVar>) {
    if no_env {
        return (input.to_owned(), Vec::new());
    }

    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut warnings = Vec::new();
    let mut i = 0;

    while i < len {
        // Escaped dollar: `\${` → literal `${`
        if bytes[i] == b'\\' && i + 2 < len && bytes[i + 1] == b'$' && bytes[i + 2] == b'{' {
            // Skip the backslash, emit `$` and continue (the `{…}` will be
            // consumed as ordinary characters on subsequent iterations).
            out.push('$');
            i += 2; // skip `\$`, loop will pick up `{`
            continue;
        }

        // Expansion: `${IDENT}`
        if bytes[i] == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
            let dollar_offset = i;
            let name_start = i + 2; // past `${`
            // Scan for closing `}`
            if let Some(close) = bytes[name_start..].iter().position(|&b| b == b'}') {
                let name = &input[name_start..name_start + close];
                if !name.is_empty() && is_valid_ident(name) {
                    match std::env::var(name) {
                        Ok(val) => out.push_str(&val),
                        Err(_) => {
                            warnings.push(UnsetVar {
                                name: name.to_owned(),
                                offset: dollar_offset,
                            });
                            // Expand to empty string
                        }
                    }
                    i = name_start + close + 1; // past `}`
                    continue;
                }
            }
            // Malformed or empty — pass through literally
            out.push('$');
            i += 1;
            continue;
        }

        out.push(bytes[i] as char);
        i += 1;
    }

    (out, warnings)
}

/// Check whether `s` is a valid environment variable identifier.
///
/// Accepts ASCII letters, digits, and underscores.  The first character
/// must not be a digit.
fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: expand with env expansion enabled.
    fn expand(input: &str) -> (String, Vec<UnsetVar>) {
        expand_env_vars(input, false)
    }

    #[test]
    fn literal_passthrough() {
        let (out, warns) = expand("hello world");
        assert_eq!(out, "hello world");
        assert!(warns.is_empty());
    }

    #[test]
    fn basic_expansion() {
        // Use a variable guaranteed to exist in any test environment.
        let key = "QED_TEST_EXPAND_BASIC";
        unsafe { std::env::set_var(key, "expanded_value") };
        let (out, warns) = expand(&format!("before${{{key}}}after"));
        assert_eq!(out, "beforeexpanded_valueafter");
        assert!(warns.is_empty());
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn unset_var_expands_to_empty_with_warning() {
        let key = "QED_TEST_EXPAND_UNSET_XYZ";
        unsafe { std::env::remove_var(key) };
        let (out, warns) = expand(&format!("a${{{key}}}b"));
        assert_eq!(out, "ab");
        assert_eq!(warns.len(), 1);
        assert_eq!(warns[0].name, key);
        assert_eq!(warns[0].offset, 1); // `$` is at byte 1
    }

    #[test]
    fn escaped_dollar_brace() {
        let (out, warns) = expand(r"\${VAR}");
        assert_eq!(out, "${VAR}");
        assert!(warns.is_empty());
    }

    #[test]
    fn bare_dollar_no_brace_passthrough() {
        let (out, warns) = expand("price is $5");
        assert_eq!(out, "price is $5");
        assert!(warns.is_empty());
    }

    #[test]
    fn dollar_ident_without_braces_passthrough() {
        let (out, warns) = expand("$VAR is not expanded");
        assert_eq!(out, "$VAR is not expanded");
        assert!(warns.is_empty());
    }

    #[test]
    fn no_env_disables_expansion() {
        let key = "QED_TEST_EXPAND_NOENV";
        unsafe { std::env::set_var(key, "should_not_appear") };
        let (out, warns) = expand_env_vars(&format!("${{{key}}}"), true);
        assert_eq!(out, format!("${{{key}}}"));
        assert!(warns.is_empty());
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn mixed_expanded_and_escaped() {
        let key = "QED_TEST_EXPAND_MIX";
        unsafe { std::env::set_var(key, "val") };
        let (out, warns) = expand(&format!("a${{{key}}}b\\${{LITERAL}}c"));
        assert_eq!(out, "avalb${LITERAL}c");
        assert!(warns.is_empty());
        unsafe { std::env::remove_var(key) };
    }

    #[test]
    fn empty_var_name_passthrough() {
        let (out, warns) = expand("${}");
        assert_eq!(out, "${}");
        assert!(warns.is_empty());
    }

    #[test]
    fn unterminated_brace_passthrough() {
        let (out, warns) = expand("${VAR");
        assert_eq!(out, "${VAR");
        assert!(warns.is_empty());
    }

    #[test]
    fn invalid_ident_passthrough() {
        let (out, warns) = expand("${123}");
        assert_eq!(out, "${123}");
        assert!(warns.is_empty());
    }

    #[test]
    fn multiple_expansions() {
        let k1 = "QED_TEST_MULTI_A";
        let k2 = "QED_TEST_MULTI_B";
        unsafe { std::env::set_var(k1, "one") };
        unsafe { std::env::set_var(k2, "two") };
        let (out, warns) = expand(&format!("${{{k1}}}-${{{k2}}}"));
        assert_eq!(out, "one-two");
        assert!(warns.is_empty());
        unsafe { std::env::remove_var(k1) };
        unsafe { std::env::remove_var(k2) };
    }
}
