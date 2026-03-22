//! CLI entry point for **qed**.
//!
//! Reads a qed script (inline via positional arg, or from a file via `-f`)
//! and input text from stdin, runs the script, and writes the transformed
//! output to stdout. Diagnostics go to stderr.
//!
//! Exit codes:
//! - `0` — success
//! - `1` — script execution error (parse, compile, or runtime failure)
//! - `2` — usage error (no script, conflicting flags, I/O failure)

use clap::Parser;
use std::io::Read;

#[derive(Parser)]
#[command(name = "qed", about = "Stream editor")]
struct Cli {
    /// Inline qed script
    script: Option<String>,

    /// Read script from file
    #[arg(short = 'f', long = "file")]
    file: Option<std::path::PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let script_source = match (&cli.script, &cli.file) {
        (Some(s), None) => s.clone(),
        (None, Some(path)) => std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("qed: cannot read script file: {e}");
            std::process::exit(2);
        }),
        (Some(_), Some(_)) => {
            eprintln!("qed: cannot specify both inline script and -f");
            std::process::exit(2);
        }
        (None, None) => {
            eprintln!("qed: no script provided");
            std::process::exit(2);
        }
    };

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .unwrap_or_else(|e| {
            eprintln!("qed: cannot read stdin: {e}");
            std::process::exit(2);
        });

    match qed_core::run(&script_source, &input) {
        Ok(result) => {
            print!("{}", result.output);
            for d in &result.diagnostics {
                eprintln!(
                    "qed: {level:<9}{loc}: {sel}: {msg}",
                    level = format!("{}:", d.level),
                    loc = d.location,
                    sel = d.selector_text,
                    msg = d.message,
                );
            }
            if result.has_errors {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("qed: {err}");
            std::process::exit(1);
        }
    }
}
