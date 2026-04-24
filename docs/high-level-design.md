# High-Level Design: qed

## Problem

Traditional UNIX stream editors (sed, awk, perl -pe) solve a real need — targeted
in-place transformation of text streams — but their syntax is opaque, their
region-selection model is implicit, and composing multiple transformations requires
brittle chaining of separate invocations.
Users who need to select a region of lines and pipe it through a transformation have
no tool that makes both operations first-class.

## Approach

A single composable primitive: `selector | processor`.

A **selector** targets a contiguous region of input lines (by line number, pattern
match, range, or logical combination).
A **processor** transforms the selected lines (substitution, deletion, insertion,
external command, etc.).
Multiple statements are composed by writing them in sequence; each one sees the
output of previous transforms.

The language is interpreted from a script string or file; the execution model is a
single-pass buffer pipeline with deterministic ordering.

## Target Users

- **Developers** automating text-file edits in shell pipelines and scripts.
- **Power users** who know sed/awk but want readable, maintainable one-liners.
- **Script authors** who need structured region selection without reaching for Python
  or Perl.

They optimize for clarity and correctness over raw throughput.
They will read scripts written months ago and expect to understand them without a
manual.

## Goals

- `selector | processor` as the sole top-level construct — no special cases.
- Syntax readable to someone who has never seen qed before.
- A graduate of Phase 12 (`v1.0`) handles all common sed/awk one-liner equivalents.
- 494+ tests pass with zero warnings on stable Rust.
- Shell completions, helpful diagnostics, and a man-page-quality reference.

## Non-Goals

- Not a general programming language — no variables, loops, or control flow in
  scripts (use shell for orchestration).
- Not a line-field processor — structured field extraction is awk's domain.
- Not a streaming/infinite-input processor — the buffer model loads the full input.
- Not a drop-in sed replacement — different syntax by design.

## System Design

Three-crate Cargo workspace:

```
qed-core     library — all domain logic (parse, compile, execute)
qed          binary — thin CLI wrapper, argument parsing, I/O wiring
qed-tests    integration test harness (libtest-mimic)
```

Pipeline inside `qed-core`:

```
source → parse → Program (AST) → compile → Script (IR) → execute → output
```

**Fragment model**: the execution engine partitions the input `Buffer` into a
`FragmentList` of `Passthrough` and `Selected` regions.
Selected fragments carry `(StatementId, SelectorId)` tags.
Statements execute in declaration order; each processor's output is re-fragmented
against remaining statements before being spliced back in.
The original `Buffer` is never mutated.

```mermaid
flowchart LR
    A[stdin / file] --> B[Buffer]
    B --> C[parse]
    C --> D[Program AST]
    D --> E[compile]
    E --> F[Script IR]
    F --> G[execute]
    G --> H[stdout / file]
```

## Key Design Decisions

**Recursive descent parser over PEG/chumsky.**
The chumsky alternative was prototyped in Phase 3 and removed.
Hand-written RD gives precise error spans, simpler recovery, and no external
compile-time dependency on a parser-combinator crate.

**Buffer model over streaming.**
Selectors like `first..last` require look-ahead across the full input.
A streaming model would force two-pass buffering anyway; the buffer model makes the
contract explicit and simplifies the fragment engine.

**Fragment list as immutable view.**
Keeping the original `Buffer` unmodified and expressing all changes as fragment
replacements means every statement sees a consistent view of "original input" vs.
"in-flight output".
This eliminates a class of ordering bugs that arise when editors mutate in place.

**Newtypes for IDs.**
`StatementId(usize)` and `SelectorId(usize)` prevent silent cross-assignment.
The compiler enforces the boundary at zero cost.

## Success Metrics

- All 494 tests pass on `cargo test` with zero warnings.
- `qed '1..3 | s/foo/bar/'` is readable to a developer unfamiliar with the tool.
- Common sed/awk one-liners have a shorter or equally short qed equivalent in
  `docs/qed-examples.md`.
- No `unwrap()` calls outside tests; all error paths surface a user-readable
  diagnostic.

## References

- `docs/qed-design.md` — language specification and formal grammar
- `docs/qed-implementation-design.md` — pipeline architecture and buffer/fragment model
- `docs/qed-project-structure.md` — workspace layout and crate responsibilities
- `docs/qed-roadmap.md` — phased build plan
- `docs/qed-rust-conventions.md` — codebase conventions
