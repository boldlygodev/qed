# Claude Code Orientation — `qed`

`qed` is a modern CLI stream editor written in Rust.
Its core primitive is `selector | processor` — select a region of lines, pipe it through a transformation.
Implemented as a Cargo workspace with three crates.

---

## Key Documents

Read these before writing any code.
They are the authoritative source for all design decisions.

| Document | What it covers |
|---|---|
| `docs/qed-design.md` | Language design, selectors, processors, invocation flags, formal grammar |
| `docs/qed-implementation-design.md` | Pipeline architecture, buffer/fragment model, AST types, IR types, compilation pass, error routing, CLI struct |
| `docs/qed-project-structure.md` | Workspace layout, crate responsibilities, module breakdown, feature flag wiring |
| `docs/qed-roadmap.md` | Phased build plan — what to build, in what order, with checkpoints |
| `docs/qed-rust-conventions.md` | Codebase conventions: error handling, visibility, naming, newtypes, trait objects, ownership |
| `docs/qed-dev-workflow.md` | Build, run, test, lint commands; switching parser feature flags; adding dependencies |
| `.claude/tests/harness.md` | Integration test harness specification — Rust/bash split, `scenario.sh` format, golden comparison rules, mock scripts |

Test scenarios (inputs, scripts, goldens, manifests) live in the per-feature `.md` files under `.claude/tests/`:
`selectors.md`, `processors.md`, `patterns.md`, `invocation.md`, `error-handling.md`,
`generation.md`, `stream-control.md`, `external-processors.md`, `script-files.md`,
and their corresponding `-edge-cases.md` variants, plus `usecases.md`.

---

## Workspace Layout

```
qed/
  Cargo.toml              # workspace root
  .claude/CLAUDE.md       # this file
  qed-core/               # library crate — all domain logic
    Cargo.toml
    src/
      lib.rs
      span.rs
      error.rs
      diagnostic.rs
      parse/
        mod.rs
        ast.rs
        rd/               # hand-written recursive descent (feature: parser-rd, default)
        chumsky/          # combinator parser (feature: parser-chumsky)
      compile/
      selector/
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

**Phase 0 — Workspace Scaffold** (starting point).
See `docs/qed-roadmap.md` for the full 12-phase plan.
Phases 0–2 establish the foundation (scaffold, core types, fragmentation algorithm).
Phase 3 evaluates the parser POC.
Phase 4 is the walking skeleton + harness first-green milestone.

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

**Parser feature flags:** `parser-rd` is the default.
`parser-chumsky` is the alternative under evaluation.
Do not use `#[cfg(feature = "...")]` anywhere outside `parse/mod.rs`.

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

Integration tests run via `cargo test --package qed-tests`.
Trial names are `<suite>::<scenario-id>::<invocation-index>`.

```sh
cargo test --package qed-tests selectors::at-literal::0
```

Each trial generates a `scenario.sh`, invokes `tests/harness/run-scenario.sh`,
and reports pass/fail from that script's exit code.
Golden files use `.txt` (exact), `.pattern` (full-string regex), or `.*` (glob).
See `.claude/tests/harness.md` for full specification.

---

## Open Implementation Notes

Two items flagged in the test scenario docs for verification during implementation:

- **UUID v5 exact golden** (`.claude/tests/generation.md`) — the value `c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694`
  in `uuid-v5-line.txt` must be verified against actual Rust UUID library output
  and updated if incorrect.
  Address during Phase 8 (Generation Processors).

- **`uuid-v7-after` script** (`.claude/tests/generation.md`) — the script uses
  `after("header") | qed:replace("", qed:uuid())` as a workaround.
  If generation processors work directly in `after` pipelines
  (i.e. `after("header") | qed:uuid()` is valid), update the script to use the
  simpler form.
  Confirm during Phase 8.

One open design concern (deferred from `docs/qed-implementation-design.md`):

- **End-of-run diagnostic summary** — whether to emit a summary line
  (e.g. `2 warnings, 1 error`) is unresolved.
  No summary is emitted by default.
  Revisit during Phase 10 (Diagnostics) once the diagnostic format is validated
  against real scripts.
