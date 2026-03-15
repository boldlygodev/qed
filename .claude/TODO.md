# TODO

---

## Design

- [x] **`qed:number()` format** — resolved.
  Colon-space separator (`1: foo`).
  Minimal padding by default; optional `width` param for fixed-width right-alignment.
  Stream line numbers by default; optional `start` param to set origin (`start:1` gives region-relative numbering).
  Documented in `qed-design.md` § Processors.

- [x] **`qed:debug:count()` format** — resolved.
  Format: `qed: debug: 5:12-20: at("foo"): 3 matches`.
  Follows the universal diagnostic format (see Diagnostics section).
  Documented in `qed-design.md` § Diagnostics.

- [x] **`after() | generation-processor` composition** — resolved.
  Generation processors used directly in `after` or `before` pipelines insert their output
  as a new line at the cursor position. Both the `qed:replace()` substitution form and the
  direct insertion form are valid.
  Documented in `qed-design.md` § Processors → Generation.

- [x] **stderr diagnostic message format** — resolved.
  Format: `qed: <severity>: <location>: <source-expression>: <message>`.
  Severity padded to `warning:` width; location padded to widest span in script (computed
  from AST before execution); source expression and message unpadded.
  1-based line and byte offsets. One diagnostic per event, no end-of-run summary.
  Documented in `qed-design.md` § Diagnostics and `qed-implementation-design.md` § Resolved Concerns.

---

## Implementation Design

- [x] **stdout behaviour on non-zero exit** — resolved.
  Emit lines as fragments become tag-free; free memory as you go.
  No output rollback on failure. `set -o pipefail` recommended when piping.
  `--on-error=skip/warn` users accept downstream contract.
  Documented in `qed-design.md` § Error Handling and `qed-implementation-design.md` § Resolved Concerns.

- [x] **`--dry-run` context line count** — resolved. 3 lines (standard default).
  Documented in `qed-design.md` § Diagnostics and `qed-implementation-design.md` § Resolved Concerns.

- [x] **`${QED_FILE}` cleanup timing** — resolved.
  Temp files are cleaned up when `${QED_FILE}` goes out of scope — when the downstream command exits.
  Documented in `qed-implementation-design.md` § Resolved Concerns.

- [x] **Parser library decision** — resolved.
  Both implementations (hand-written recursive descent and chumsky combinator) will be
  spiked against the `nth` expression grammar in Phase 3 of the implementation roadmap.
  Evaluation criteria: error quality, span accuracy, grammar coverage, debuggability,
  compile time. The loser is deleted after evaluation.
  Feature flags: `parser-rd` (default), `parser-chumsky` (opt-in).
  Routing in `parse/mod.rs` via `#[cfg(feature = "...")]`.
  Full specification in `qed-project-structure.md` § Parser and `qed-roadmap.md` § Phase 3.

---

## Implementation Documentation

New documents written during the pre-implementation planning phase.
All should be committed to the repository root alongside the existing design docs.

- [x] **`qed-project-structure.md`** — workspace layout, crate responsibilities,
  hybrid module breakdown inside `qed-core`, feature flag wiring for the parser POC,
  parser evaluation criteria, pruning steps, and key dependencies.

- [x] **`qed-roadmap.md`** — 12-phase implementation plan.
  Phases 0–2: scaffold, core types, fragmentation algorithm.
  Phase 3: parser POC evaluation.
  Phase 4: walking skeleton + harness first-green milestone.
  Phases 5–12: full parser, compiler, processor coverage, generation processors,
  invocation features, diagnostics, edge cases, release polish.
  Each phase has a concrete checkpoint.

- [x] **`qed-dev-workflow.md`** — day-to-day development guide.
  Build, run, test, lint commands; switching parser implementations;
  adding dependencies; reading Rust errors.

- [x] **`qed-rust-conventions.md`** — codebase-specific Rust conventions.
  Error handling (`?`, accumulator pattern), visibility rules, naming,
  the newtype pattern, `Box<dyn Trait>`, `Spanned<T>`, exhaustive matching,
  ownership and borrowing in the fragment model, module declarations, doc comments,
  feature-gated code.

- [x] **`CLAUDE.md`** — Claude Code orientation file.
  Read at startup by Claude Code.
  Covers: what `qed` is, key documents and what each covers, workspace layout,
  current phase, critical conventions, pipeline summary, test harness invocation,
  and open implementation notes.

---

## Documentation Updates Required

These updates are needed in the test scenario docs as a result of design decisions now made.

- [x] **Update `qed:number()` golden** — `number-single-result.txt` in
  `processors-edge-cases.md` and `foo-bar-baz-numbered.txt` in `processors.md`
  updated to `N: line` format (colon-space separator, stream line numbers).

- [x] **Update stderr placeholder goldens** — all `⚠️ Placeholder` goldens updated
  with confirmed diagnostic format `qed: <severity>: <location>: <source>: <message>`.
  Severity padded to `warning:` width; location padded to widest span per script.
  Affected files: `selectors.md`, `error-handling.md`, `error-handling-edge-cases.md`,
  `invocation.md`, `invocation-edge-cases.md`, `stream-control.md`,
  `external-processors-edge-cases.md`.
  Golden splits: `error-no-match.txt` split into `error-no-match-first-statement.txt` /
  `error-no-match-second-statement.txt` in `error-handling-edge-cases.md`;
  `processor-failed.txt` / `fallback-processor-failed.txt` split in `error-handling.md`.

- [x] **Update `dry-run-*` goldens** — `dry-run-delete-bar.txt` and
  `dry-run-multiple-hunks.txt` in `invocation.md` and `invocation-edge-cases.md`
  were already written correctly. Context line count confirmed as 3.

- [x] **Update stdout-on-failure golden files** — emit-as-done semantics applied.
  Five scenarios updated with correct stdout/output goldens; three new `foo.txt` goldens
  added (`error-handling-edge-cases.md`, `stream-control.md`).
  Affected: `on-error-fail` (selectors.md), `per-selector-on-error-overrides-global`
  (invocation-edge-cases.md), `multiple-statements-first-fails` and
  `multiple-statements-second-fails` (error-handling-edge-cases.md), `fail`
  (stream-control.md). All descriptions updated to reflect emit-as-done language.

- [x] **Document `set -o pipefail` recommendation** — added to the CLI Reference
  section of `README.md` under "Pipelines and `set -o pipefail`".

---

## Testing

### Incomplete scenarios

- [x] **`external-replace-empty-match`** — scenario removed.
  The lookbehind (`(?<=foo)`) is unsupported by RE2 and the behavior under test (empty-input
  external command) is already covered by `external-empty-input`.
  No replacement scenario was needed.

- [x] **`named-pattern-in-from-to`** in `patterns-edge-cases.md` — resolved.
  Added dedicated `inputs/range-source.txt` (`alpha / start / bravo / charlie / delta / end / echo`),
  new script `named-pattern-in-from-to.qed` using named literal patterns as `from > to` boundaries,
  new manifest entry, and `alpha-echo.txt` golden.

- [x] **`nth-negative-step` golden** in `selectors-edge-cases.md` — resolved.
  `nth:-2n` selects end-positions 2, 4, 6…; with three `x` matches only match 2 is selected.
  Golden renamed from `y-x-y.txt` to `x-y-y-x.txt`; manifest entry and script description updated.
  Golden content written as `x / y / y / x`.

- [x] **`timestamp-timezone-line.pattern`** in `generation-edge-cases.md` — resolved.
  `format:datetime` never embeds the UTC offset; output is bare `yyyy-MM-dd HH:mm:ss`.
  Pattern written as `^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$`, identical to the
  `timestamp-timezone-replace` golden in `tests/generation/`.
  Scenario description updated to note the shared structure and smoke-test intent.

### Remaining test work

- [x] **Regression scenarios** — written.
  After audit, most cases were already covered by existing scenarios:
  sequential statement semantics by `named-pattern-reused` (`patterns-edge-cases.md`),
  `qed:replace()` surrounding content by `replace-literal` (`processors.md`),
  `qed:substring()` no match by `substring-no-match-on-line` (`processors-edge-cases.md`),
  and passthrough preservation implicitly by every scenario.
  One genuinely missing case added: `insertion-point-no-output` in
  `external-processors-edge-cases.md` — verifies that an `after()` command writing
  nothing to stdout inserts nothing and the stream passes through unchanged.

- [x] **Use case scenarios** — written as `usecases.md`.
  Seven suites: `code-editing` (3 scenarios), `config-manipulation` (3),
  `log-processing` (3), `code-generation` (2), `template-rendering` (2),
  `document-processing` (3), `editor-integration` (2). 18 scenarios total.
  All inputs, scripts, manifests, and goldens written.
  `inject-uuid` uses `.pattern` golden for non-deterministic UUID output.
  `git-commit-cleanup` and `kubectl-enforce-limits` use `--in-place` with
  `cp "$INPUT" "$OUTPUT"` to capture the mutated file.

- [x] **AI-assisted transformation examples in README** — added as its own section
  in `README.md` with five examples: implement a TODO, add error handling, rewrite
  comments, generate a docstring, and translate comments.

- [x] **Warning scenarios** — written.
  Distributed into existing edge-case files.
  Covered: `+` on `at`/`after`/`before` (3 scenarios in `selectors-edge-cases.md`),
  `nth:0` (1 scenario in `selectors-edge-cases.md`),
  duplicate `nth` values in `b` and `...` forms (2 scenarios in `selectors-edge-cases.md`),
  `qed:file()` on an insertion point (1 scenario in `external-processors-edge-cases.md`),
  duplicate pattern name definitions (1 scenario in `patterns-edge-cases.md`).
  Unset env var warning already covered in `invocation-edge-cases.md`.
  All stderr goldens written using the confirmed diagnostic format.

- [x] **Duplicate pattern name behavior** — specified in `qed-design.md`.
  Added to Pattern Syntax section (forward references permitted; redefinition warns
  and last definition wins) and to the Constraints list.
  Changelog entries added to both `qed-design.md` and `qed-implementation-design.md`.

### Flagged for verification during implementation

- [ ] **UUID v5 exact golden** (`generation.md`) — the value `c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694`
  in `uuid-v5-line.txt` must be verified against actual Rust UUID library output and updated
  if incorrect. UUID v5 is deterministic but the value depends on the SHA-1 implementation
  and namespace byte encoding. Address during Phase 8 (Generation Processors).

- [ ] **`uuid-v7-after` script** (`generation.md`) — the script currently uses
  `after("header") | qed:replace("", qed:uuid())` as a workaround because it was
  unclear whether generation processors work directly in `after` pipelines.
  If `after("header") | qed:uuid()` is valid (it should be — resolved in design),
  update the script to use the simpler form. Confirm and fix during Phase 8.

### Harness design

- [x] **Harness specification** — written as `harness.md`.
  Covers: `libtest-mimic` integration with one `Trial` per scenario;
  Rust/bash split (Rust: manifest parsing, Trial registration, temp dir lifecycle,
  `scenario.sh` generation, exec, pass/fail; bash: file setup, mock generation,
  invocation, comparison, unconsumed mock detection);
  `scenario.sh` format including exact variable names for scalars, invocations,
  mock declarations, and env exports;
  per-invocation isolation via `invocation-N/` subdirectories with fresh input copies;
  `$SCRIPT` / `$INPUT` / `$STDOUT` / `$STDERR` / `$OUTPUT` variable injection;
  mock script generation with stateful call counter via `$MOCK_STATE_DIR/<command>.count`;
  `${QED_FILE}` in `expected_args` stored single-quoted and expanded at mock
  validation time, not generation time;
  `env` table values expanded from the Rust process environment at `scenario.sh`
  generation time;
  golden comparison rules (`.txt` exact, `.pattern` full-string regex, `.*` glob);
  unconsumed mock reporting after all invocations complete;
  exit code assertion per invocation;
  failure message format for `cargo test` output.
