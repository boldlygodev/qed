# `qed` Project Structure

This document captures the workspace layout, crate responsibilities, and module organization for the `qed` implementation.
It is the authoritative reference for project structure decisions made before implementation began.

---

## Workspace Layout

`qed` uses a Cargo workspace with three crates.

```
qed/
  Cargo.toml              # workspace root
  qed-core/               # library crate — all domain logic
    Cargo.toml
    src/
      lib.rs
      span.rs
      error.rs
      diagnostic.rs
      parse/
      compile/
      selector/
      processor/
      exec/
  qed/                    # binary crate — CLI entry point
    Cargo.toml
    src/
      main.rs
  qed-tests/              # integration test harness (libtest-mimic)
    Cargo.toml
    src/
      main.rs
    tests/
```

**Rationale:**
Separating the library from the binary gives `qed-core` a clean public API surface,
keeps the binary thin,
and allows the integration test harness to live as its own crate with its own dependency set.

---

## Crate Responsibilities

### `qed-core`

The library crate.
Contains all parsing, compilation, and execution logic.
Exposes a public API for use by the binary and the test harness.
Has no knowledge of the terminal, process arguments, or file I/O beyond what the domain requires.

### `qed`

The binary crate.
Depends on `qed-core`.
Responsible for: reading CLI arguments via `clap`, opening input files, writing output, and wiring the `qed-core` pipeline together.
Contains no domain logic.

### `qed-tests`

The integration test harness crate.
Uses `libtest-mimic` to register one `Trial` per test scenario.
Parses `manifest.toml` files under `tests/`,
generates `scenario.sh` in a temp directory,
invokes the bash layer,
and reports pass/fail.
See `harness.md` for full specification.

---

## Module Structure (`qed-core`)

The internal module layout follows a hybrid approach:
foundational cross-cutting concerns get their own top-level modules,
phase logic owns both its types and its transformation code,
and domain areas complex enough to stand alone get dedicated top-level modules.

### Foundational Modules

These modules are referenced across multiple phases and do not belong to any single one.

| Module | Contents |
|---|---|
| `span` | `Span`, `Spanned<T>` — source location tracking for diagnostics |
| `error` | `CompileError`, `ExecError`, error collection types |
| `diagnostic` | Diagnostic formatting, severity levels, stderr emission |

### Phase Modules

Each phase module owns the types it produces and the logic that produces them.

| Module | Types owned | Logic owned |
|---|---|---|
| `parse` | AST — `Program`, `Statement`, `Selector`, `PatternRef`, `ProcessorChain`, `NthExpr`, … | Parser (see [Parser](#parser)) |
| `compile` | IR — `Script`, `CompiledSelector`, `CompiledPattern`, `SymbolTable`, … | Compilation pass, symbol resolution, regex compilation |
| `exec` | `FragmentList`, `Fragment`, `LineRange` | Fragmentation algorithm, execution engine, output emission |

### Domain Modules

These are independently complex areas that are referenced across multiple phases.

| Module | Contents |
|---|---|
| `selector` | `SelectorOp`, `CompiledSelector`, `CompoundSelector`, match collection logic |
| `processor` | `Processor` trait, all built-in `qed:*` processor implementations |

---

## Parser

The parser uses hand-written recursive descent, implemented under `parse/rd/`.

A chumsky 0.9 combinator parser was evaluated as an alternative during Phase 3.
Recursive descent won on every criterion — compile time (1.5s vs 2.9s clean build),
error quality (natural zero-offset detection vs source re-inspection workaround),
debuggability (straightforward control flow vs fighting type inference), and
dependency count (0 new deps vs 16). The chumsky spike was removed.

### Directory Layout

```
parse/
  mod.rs          # public entry point — delegates to rd/
  ast.rs          # AST type definitions
  error.rs        # ParseError enum, ParseResult struct
  rd/             # hand-written recursive descent
    mod.rs
    cursor.rs
    parser.rs
```

---

## Key Dependencies

| Crate | Used by | Purpose |
|---|---|---|
| `clap` | `qed` | CLI argument parsing |
| `regex` | `qed-core` | Pattern compilation and matching |
| `rayon` | `qed-core` | Parallel selector match collection |
| `libtest-mimic` | `qed-tests` | Test harness registration |
| `toml` | `qed-tests` | Manifest parsing |
