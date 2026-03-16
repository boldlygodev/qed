# TODO

Progress tracker for the qed implementation roadmap.
For phase details, deliverables, and checkpoints see `docs/qed-roadmap.md`.

---

- [x] **Phase 0 — Workspace Scaffold**
  `cargo build --workspace` clean with both feature flag configurations.

- [x] **Phase 1 — Test Harness Infrastructure**
  `cargo test --package qed-tests` registers all 396 trials (failing).

- [ ] **Phase 2 — Core Types and Fragmentation Algorithm**
  Buffer, fragment, and fragmentation unit tests pass.

- [ ] **Phase 3 — Parser POC Evaluation**
  One parser remains, feature flag routing removed, workspace builds clean.

- [ ] **Phase 4 — Walking Skeleton**
  `cargo test --package qed-tests selectors::at-literal-single-match::0` passes.

- [ ] **Phase 5 — Full Parser**
  All grammar productions parsed and unit-tested; `selectors` suite going green.

- [ ] **Phase 6 — Full Compiler**
  Compilation unit tests pass; `selectors` integration suite fully green.

- [ ] **Phase 7 — Processor Coverage**
  `processors` and `external-processors` integration suites green.

- [ ] **Phase 8 — Generation Processors**
  `generation` integration suite green.
  - [ ] Verify UUID v5 golden value (`uuid-v5-line.txt`) against actual Rust UUID
    library output and update if incorrect (`generation.md`).
  - [ ] Simplify `uuid-v7-after` script to `after("header") | qed:uuid()` if
    generation processors work directly in `after` pipelines (they should).

- [ ] **Phase 9 — Invocation Features**
  `invocation`, `stream-control`, and `script-files` integration suites green.

- [ ] **Phase 10 — Diagnostics**
  All warning scenarios and `error-handling` suites green.

- [ ] **Phase 11 — Edge Cases + Use Cases**
  `cargo test --workspace` fully green.

- [ ] **Phase 12 — Release Polish**
  Shell completions, README verified, `clippy` and `fmt` clean.
