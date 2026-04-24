//! CLI entry point for **qed**.
//!
//! Reads a qed script (inline via positional arg, or from a file via `-f`)
//! and input text from stdin or a file, runs the script, and writes the
//! transformed output to stdout. Diagnostics go to stderr.
//!
//! Exit codes:
//! - `0` — success
//! - `1` — script execution error (parse, compile, or runtime failure)
//! - `2` — usage error (no script, conflicting flags, I/O failure)

mod diff;

use clap::{CommandFactory, Parser};
use std::io::Read;
use std::path::PathBuf;

// @spec CLI-001, CLI-002, CLI-003, CLI-004, CLI-060, CLI-061, CLI-062
#[derive(Parser)]
#[command(name = "qed", version, about = "Stream editor")]
struct Cli {
    /// Read script from file
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    /// Modify input file directly (atomic write)
    #[arg(
        short = 'i',
        long,
        conflicts_with = "output",
        conflicts_with = "dry_run"
    )]
    in_place: bool,

    /// Suppress passthrough output; only selected regions are emitted
    #[arg(short = 'x', long)]
    extract: bool,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Preview changes as a unified diff
    #[arg(short = 'd', long)]
    dry_run: bool,

    /// Global on-error mode (fail, warn, skip)
    #[arg(long, default_value = "fail")]
    on_error: qed_core::OnError,

    /// Disable environment variable expansion
    #[arg(long)]
    no_env: bool,

    /// Generate shell completions and exit
    #[arg(long, hide = true, value_name = "SHELL")]
    completions: Option<clap_complete::Shell>,

    /// Positional arguments: [SCRIPT] [FILE] or [FILE] (when -f is used)
    args: Vec<String>,
}

// @spec CLI-010, CLI-011, CLI-012, CLI-013, CLI-014, CLI-020, CLI-021, CLI-022, CLI-023, CLI-040, CLI-041, CLI-042, CLI-043, CLI-050, CLI-051, CLI-052
fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut Cli::command(), "qed", &mut std::io::stdout());
        return;
    }

    // Interpret positional args based on whether -f is used.
    // Without -f: args[0] = script, args[1] = input file (optional)
    // With -f:    args[0] = input file (optional)
    let (script_source, input_path) = if let Some(ref path) = cli.file {
        if cli.args.len() > 1 {
            eprintln!("qed: too many arguments");
            std::process::exit(2);
        }
        let script = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("qed: cannot read script file: {e}");
            std::process::exit(2);
        });
        let input = cli.args.first().map(PathBuf::from);
        (script, input)
    } else {
        if cli.args.is_empty() {
            eprintln!("qed: no script provided");
            std::process::exit(2);
        }
        if cli.args.len() > 2 {
            eprintln!("qed: too many arguments");
            std::process::exit(2);
        }
        let script = cli.args[0].clone();
        let input = cli.args.get(1).map(PathBuf::from);
        (script, input)
    };

    // --in-place requires an input file
    if cli.in_place && input_path.is_none() {
        eprintln!("qed: --in-place requires an input file");
        std::process::exit(2);
    }

    // Read input from file or stdin.
    let input = if let Some(ref path) = input_path {
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("qed: cannot read input file: {e}");
            std::process::exit(2);
        })
    } else {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| {
                eprintln!("qed: cannot read stdin: {e}");
                std::process::exit(2);
            });
        buf
    };

    let options = qed_core::RunOptions {
        no_env: cli.no_env,
        on_error: cli.on_error,
        extract: cli.extract,
    };

    match qed_core::run(&script_source, &input, &options) {
        Ok(result) => {
            // Route output: --dry-run → diff, --output → file, --in-place → atomic overwrite, else → stdout
            if cli.dry_run {
                print!("{}", diff::unified_diff(&input, &result.output));
            } else if let Some(ref path) = cli.output {
                std::fs::write(path, &result.output).unwrap_or_else(|e| {
                    eprintln!("qed: cannot write output file: {e}");
                    std::process::exit(2);
                });
            } else if cli.in_place {
                // Atomic write: temp file in same directory, then rename.
                let path = input_path.as_ref().expect("validated above");
                let tmp_path = path.with_extension("qed-tmp");
                std::fs::write(&tmp_path, &result.output).unwrap_or_else(|e| {
                    eprintln!("qed: cannot write temp file: {e}");
                    std::process::exit(2);
                });
                std::fs::rename(&tmp_path, path).unwrap_or_else(|e| {
                    // Clean up temp file on rename failure.
                    let _ = std::fs::remove_file(&tmp_path);
                    eprintln!("qed: cannot rename temp file: {e}");
                    std::process::exit(2);
                });
            } else {
                print!("{}", result.output);
            }

            // Emit raw stderr lines (from qed:warn, qed:fail, qed:debug:print).
            for line in &result.stderr_lines {
                eprint!("{line}");
            }

            for d in &result.diagnostics {
                if d.selector_text.is_empty() {
                    eprintln!(
                        "qed: {level:<9}{loc}: {msg}",
                        level = format!("{}:", d.level),
                        loc = d.location,
                        msg = d.message,
                    );
                } else {
                    eprintln!(
                        "qed: {level:<9}{loc}: {sel}: {msg}",
                        level = format!("{}:", d.level),
                        loc = d.location,
                        sel = d.selector_text,
                        msg = d.message,
                    );
                }
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
