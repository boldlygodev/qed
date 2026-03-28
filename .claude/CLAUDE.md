# Claude Code Orientation — `qed`

`qed` is a modern CLI stream editor written in Rust.
Its core primitive is `selector | processor` — select a region of lines, pipe it through a transformation.
Implemented as a Cargo workspace with three crates.

---

## Key Documents

Read these before writing any code.
They are the authoritative source for all design decisions.

| Document                            | What it covers                                                                                                        |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `docs/qed-design.md`                | Language design, selectors, processors, invocation flags, formal grammar                                              |
| `docs/qed-implementation-design.md` | Pipeline architecture, buffer/fragment model, AST types, IR types, compilation pass, error routing, CLI struct        |
| `docs/qed-project-structure.md`     | Workspace layout, crate responsibilities, module breakdown, feature flag wiring                                       |
| `docs/qed-roadmap.md`               | Phased build plan — what to build, in what order, with checkpoints                                                    |
| `docs/qed-rust-conventions.md`      | Codebase conventions: error handling, visibility, naming, newtypes, trait objects, ownership                          |
| `docs/qed-dev-workflow.md`          | Build, run, test, lint commands; switching parser feature flags; adding dependencies                                  |
| `.claude/tests/harness.md`          | Integration test harness specification — Rust/bash split, `scenario.sh` format, golden comparison rules, mock scripts |

Test scenarios (inputs, scripts, goldens, manifests) live in the per-feature `.md` files under `.claude/tests/`:
`selectors.md`, `processors.md`, `patterns.md`, `invocation.md`, `error-handling.md`,
`generation.md`, `stream-control.md`, `external-processors.md`, `script-files.md`,
and their corresponding `-edge-cases.md` variants, plus `usecases.md`.

---

## Workspace Layout

```
qed/
  Cargo.toml              # workspace root
  mise.toml               # tool versions, env vars, mise tasks
  mise.lock               # generated lockfile (committed)
  rust-toolchain.toml     # Rust stable channel pin
  .claude/CLAUDE.md       # this file
  qed-core/               # library crate — all domain logic
    Cargo.toml
    src/
      lib.rs
      span.rs
      error.rs
      parse/
        mod.rs
        ast.rs
        error.rs
        rd/               # hand-written recursive descent parser
          cursor.rs
          parser.rs
      compile/
        env.rs
      processor/
      exec/
  qed/                    # binary crate — CLI entry point only
    Cargo.toml
    src/
      main.rs
  qed-tests/              # integration test harness (libtest-mimic)
    Cargo.toml
    src/
      main.rs
  tests/                  # test suites: manifests, inputs, scripts, goldens
```

---

## Current Phase

**Phase 10 — Diagnostics** in progress.
See `docs/qed-roadmap.md` for the full 12-phase plan.
Phases 0–9 are complete. 10A complete. 344/396 integration tests pass.
Sub-phases: 10A ✓ (fallback on no-match), 10B (stream-control processors),
10C (location padding), 10D (checkpoint).

---

## Critical Conventions

These are non-negotiable across the entire codebase.

**Language:** American English in all code, comments, docs, and diagnostic messages.

**Visibility:** use `pub` only for `qed-core`'s intentional public API.
Use `pub(crate)` for cross-module internals.
Leave everything else private.
Never use `pub` by default.

**Error handling:** use `?` for single-error propagation.
Use the accumulator pattern (`Vec<CompileError>`) only in the compilation pass.
Never use `unwrap()` outside of tests — use `expect("reason")` for genuinely
impossible cases, propagate everything else.

**Exhaustive matching:** never use `_` in a `match` to suppress an unhandled
variant.
If a new variant is added, the compiler should force all match sites to be updated.

**Parser:** recursive descent (`parse/rd/`) is the sole parser implementation.
The chumsky alternative was evaluated and removed in Phase 3.

**Newtypes:** `StatementId(usize)` and `SelectorId(usize)` are newtypes.
Never pass a raw `usize` where one of these is expected.

**No `\n` in `Paragraph` logic:** not applicable here — but do not
concatenate strings with embedded newlines in processor output.
Return clean line content; the execution engine handles newline joining.

---

## Pipeline

```
source → parse → AST → compile → Script → execute → output
```

- **`parse`** → `Program` (AST)
- **`compile`** → `Script` (IR: compiled selectors, processors)
- **`exec`** → fragmentation, processor dispatch, output emission

The fragment model is the heart of execution.
A `FragmentList` partitions the input buffer into `Passthrough` and `Selected`
fragments.
Selected fragments carry `(StatementId, SelectorId)` tags.
Statements execute in order; each processor's output is re-fragmented against
remaining statements before being spliced back in.
The original `Buffer` is never mutated.

---

## Test Harness

Integration tests run via `cargo test --package qed-tests --test integration`.
Trial names are `<suite>::<scenario-id>::<invocation-index>`.

```sh
cargo test --package qed-tests --test integration "selectors::at-literal-single-match::0"
```

Each trial generates a `scenario.sh`, invokes `tests/harness/run-scenario.sh`,
and reports pass/fail from that script's exit code.
Golden files use `.txt` (exact), `.pattern` (full-string regex), or `.*` (glob).
See `.claude/tests/harness.md` for full specification.

---

## Open Implementation Notes

One open design concern (deferred from `docs/qed-implementation-design.md`):

- **End-of-run diagnostic summary** — whether to emit a summary line
  (e.g. `2 warnings, 1 error`) is unresolved.
  No summary is emitted by default.
  Revisit during Phase 10 (Diagnostics) once the diagnostic format is validated
  against real scripts.
