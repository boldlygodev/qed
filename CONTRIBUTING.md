# Contributing to qed

> **Note:** qed is in early development (Phase 0 scaffold complete). The core
> language, pipeline, and test harness are not yet implemented. Design docs in
> `docs/` are the authoritative source for all decisions — read them before
> writing any code.

---

## Prerequisites

| Tool        | Purpose                  | Install                                                    |
| ----------- | ------------------------ | ---------------------------------------------------------- |
| Rust stable | Compiler and Cargo       | `mise install` (see below) or [rustup](https://rustup.rs/) |
| bash        | Integration test harness | Pre-installed on macOS/Linux                               |

**Recommended:** use [mise](https://mise.jdx.dev/) — it reads `mise.toml` and
installs the pinned Rust toolchain automatically.

```sh
# From inside the repo
mise install
```

Without mise, install Rust stable via rustup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Getting started

```sh
git clone git@github.com:boldlygodev/qed.git
cd qed/main          # or whatever your worktree path is
mise install         # installs rust stable (no-op if already present)
mise run build       # cargo build --workspace
```

---

## Daily workflow

All common tasks are defined in `mise.toml` and run via `mise run <task>`:

| Task               | Command                                   | Description                  |
| ------------------ | ----------------------------------------- | ---------------------------- |
| `build`            | `cargo build --workspace`                 | Debug build                  |
| `build:release`    | `cargo build --release`                   | Release build                |
| `check`            | `cargo check --workspace`                 | Type-check without building  |
| `test`             | `cargo test --workspace`                  | All tests                    |
| `test:unit`        | `cargo test --package qed-core`           | Unit tests only              |
| `test:integration` | `cargo test --package qed-tests`          | Integration harness          |
| `fmt`              | `cargo fmt`                               | Format code                  |
| `fmt:check`        | `cargo fmt --check`                       | Check formatting (CI mode)   |
| `lint`             | `cargo clippy --workspace -- -D warnings` | Lint with warnings-as-errors |
| `ci`               | runs fmt:check → lint → test              | Full CI check suite          |

Run `mise tasks` to see the current task list.

You can also invoke Cargo commands directly — mise just provides named shortcuts.

---

## Code style

- **Format:** run `mise run fmt` (or `cargo fmt`) before every commit
- **Lint:** run `mise run lint` (or `cargo clippy --workspace -- -D warnings`) — warnings are errors
- **CI gate:** `mise run ci` runs the full check suite locally before pushing

See `docs/qed-rust-conventions.md` for the full list of conventions including
visibility rules, error handling patterns, exhaustive matching, and newtypes.

---

## Key documents

Read these before writing any code. They are the authoritative source for all
design decisions.

| Document                            | What it covers                                                      |
| ----------------------------------- | ------------------------------------------------------------------- |
| `docs/qed-design.md`                | Language design, selectors, processors, formal grammar              |
| `docs/qed-implementation-design.md` | Pipeline architecture, buffer/fragment model, AST and IR types      |
| `docs/qed-project-structure.md`     | Workspace layout, crate responsibilities, feature flag wiring       |
| `docs/qed-roadmap.md`               | Phased build plan — what to build and in what order                 |
| `docs/qed-rust-conventions.md`      | Codebase conventions: error handling, visibility, naming, ownership |
| `docs/qed-dev-workflow.md`          | Build, test, lint commands; switching parser feature flags          |
| `.claude/tests/harness.md`          | Integration test harness specification                              |

---

## Project structure

```
qed/
  Cargo.toml          # workspace root (3 members)
  mise.toml           # tool versions, env vars, tasks
  rust-toolchain.toml # Rust stable pin
  qed-core/           # library crate — all domain logic
  qed/                # binary crate — thin CLI entry point
  qed-tests/          # integration test harness (libtest-mimic)
  tests/              # test suites: manifests, inputs, scripts, goldens
  docs/               # design documents
```

See `docs/qed-project-structure.md` for the full module breakdown.

---

## Build phases

Development follows a phased roadmap (`docs/qed-roadmap.md`). The current
status and next steps are tracked in `.claude/CLAUDE.md` under **Current Phase**.
