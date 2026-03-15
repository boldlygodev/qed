# `qed` Development Workflow

Day-to-day guide for building, testing, and iterating on `qed`.
Assumes the workspace is set up per `qed-project-structure.md`.

---

## Prerequisites

- Rust toolchain via [rustup](https://rustup.rs/) — stable channel
- `cargo` (included with rustup)
- `bash` (for the integration test harness)

Install the toolchain:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:

```sh
rustc --version
cargo --version
```

---

## Build

All commands run from the workspace root unless otherwise noted.

```sh
# Build everything (default parser: recursive descent)
cargo build

# Build with the chumsky parser instead
cargo build --package qed-core --no-default-features --features parser-chumsky

# Build release
cargo build --release
```

The `qed` binary lands at `target/debug/qed` (or `target/release/qed`).

---

## Run

```sh
# From a one-liner
cargo run -- 'at("foo") | qed:delete()' path/to/file.txt

# From a script file
cargo run -- -f path/to/script.qed path/to/file.txt

# From stdin
echo -e "foo\nbar\nbaz" | cargo run -- 'at("bar") | qed:delete()'

# With the chumsky parser
cargo run --no-default-features --features parser-chumsky -- 'at("foo") | qed:delete()'
```

After `cargo build`, you can call `./target/debug/qed` directly to avoid the
`cargo run` overhead on repeated invocations.

---

## Test

### Unit tests

Unit tests live alongside the code they test in `#[cfg(test)]` modules.

```sh
# Run all unit tests
cargo test

# Run unit tests for one crate only
cargo test --package qed-core

# Run a specific test by name (substring match)
cargo test parse_selector

# Run with output visible (useful when debugging)
cargo test -- --nocapture
```

### Integration tests (harness)

The integration test harness lives in `qed-tests/`.
It uses `libtest-mimic` and reads scenario manifests from `tests/`.

```sh
# Run all integration test suites
cargo test --package qed-tests

# Run one suite by name
cargo test --package qed-tests selectors

# Run one scenario by full name
cargo test --package qed-tests selectors::at-literal::0

# Show output from failing scenarios (includes bash diagnostic output)
cargo test --package qed-tests -- --nocapture
```

Trial names follow the pattern `<suite>::<scenario-id>::<invocation-index>` —
for example `selectors::at-literal::0`.

### Run everything

```sh
cargo test --workspace
```

---

## Switching Parsers

The two parser implementations are gated by mutually exclusive feature flags.
`parser-rd` (hand-written recursive descent) is the default.

```sh
# Default — recursive descent
cargo build
cargo test

# Chumsky parser
cargo build --no-default-features --features parser-chumsky
cargo test --no-default-features --features parser-chumsky

# Shorthand alias (add to your shell profile once the workspace is stable)
alias qed-rd="cargo run --"
alias qed-chumsky="cargo run --no-default-features --features parser-chumsky --"
```

The two parsers must produce identical output for all inputs.
A useful sanity check during development:

```sh
diff \
  <(echo 'at("foo") | qed:delete()' | cargo run -- /dev/stdin < input.txt) \
  <(echo 'at("foo") | qed:delete()' | cargo run --no-default-features --features parser-chumsky -- /dev/stdin < input.txt)
```

---

## Linting and Formatting

```sh
# Format all code
cargo fmt

# Check formatting without modifying files (use in CI)
cargo fmt --check

# Run lints
cargo clippy --workspace

# Lints as errors (matches CI strictness)
cargo clippy --workspace -- -D warnings
```

Clippy and `fmt` both operate across the full workspace.
Run them before every commit.

---

## Checking Without Building

`cargo check` is much faster than `cargo build` — use it when you just
want to verify the code compiles and types check out.

```sh
cargo check --workspace
```

---

## Adding Dependencies

Dependencies are declared per-crate in each `Cargo.toml`, not at the workspace root.

```sh
# Add a dependency to qed-core
cargo add --package qed-core regex

# Add an optional dependency (for feature-gated code)
cargo add --package qed-core chumsky --optional

# Add a dev-only dependency (for tests)
cargo add --package qed-tests --dev libtest-mimic
```

After adding a dependency, run `cargo check` to confirm it resolves correctly.

---

## Reading Errors

Rust error messages are verbose by design.
A few habits make them easier to navigate:

- **Start at the bottom of the error output** — `cargo` prints the most
  specific error last, above the final `error: could not compile` line.
- **Look for the `-->` line** — it gives the exact file and line number.
- **`error[Exxxx]`** — the error code links to detailed documentation.
  Run `rustc --explain Exxxx` for a full explanation with examples.

```sh
# Get a detailed explanation for an error code
rustc --explain E0502
```

---

## Useful Cargo Commands

```sh
# Show the full dependency tree
cargo tree

# Show which features are enabled for a package
cargo tree --package qed-core --edges features

# Show expanded macro output (useful when debugging derive macros)
cargo expand --package qed-core

# Check for outdated dependencies
cargo outdated   # requires: cargo install cargo-outdated

# Run only tests matching a filter, with output
cargo test parse -- --nocapture
```

---

## Project Layout Reminder

```
qed/
  Cargo.toml          # workspace root — no code here
  qed-core/           # all domain logic (library crate)
  qed/                # CLI entry point (binary crate)
  qed-tests/          # integration test harness
  tests/              # test suites (manifests, inputs, scripts, goldens)
```

See `qed-project-structure.md` for the full module breakdown inside `qed-core`.
