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

The parser is being evaluated as a proof-of-concept with two competing implementations.
Both live under `parse/` and are gated by mutually exclusive Cargo feature flags.
One will be pruned once evaluation is complete.

### Feature Flags

Declared in `qed-core/Cargo.toml`:

```toml
[features]
default = ["parser-rd"]
parser-rd = []
parser-chumsky = ["dep:chumsky"]
```

`parser-rd` and `parser-chumsky` are mutually exclusive.
Using both simultaneously is a compile error.

### Directory Layout

```
parse/
  mod.rs          # public parse() entry point — routes to active implementation
  ast.rs          # AST type definitions (shared by both implementations)
  rd/             # hand-written recursive descent
    mod.rs
    lexer.rs
    parser.rs
  chumsky/        # chumsky combinator parser
    mod.rs
    lexer.rs
    parser.rs
```

`ast.rs` is shared — both implementations produce the same `Program` type.

### Routing

`parse/mod.rs` exposes a single public entry point and delegates to whichever implementation is active:

```rust
#[cfg(feature = "parser-rd")]
mod rd;
#[cfg(feature = "parser-chumsky")]
mod chumsky;

pub fn parse(source: &str) -> Result<Program, Vec<ParseError>> {
    #[cfg(feature = "parser-rd")]
    return rd::parse(source);
    #[cfg(feature = "parser-chumsky")]
    return chumsky::parse(source);
}
```

### Switching Implementations

```sh
# Default (recursive descent)
cargo build

# Chumsky
cargo build --no-default-features --features parser-chumsky
```

### Evaluation Criteria

| Criterion | Description |
|---|---|
| Error quality | Clarity and accuracy of parse error messages |
| Span accuracy | Source location fidelity for diagnostics |
| Grammar coverage | Completeness against the formal grammar |
| Debuggability | Ease of tracing failures during development |
| Compile time | Impact on incremental build times |

### Pruning

Once evaluation is complete:

1. Delete the losing implementation's subdirectory (`rd/` or `chumsky/`)
2. Remove the corresponding feature flag from `Cargo.toml`
3. Simplify `parse/mod.rs` to call the winning implementation directly

No other files require changes.

---

## Key Dependencies

| Crate | Used by | Purpose |
|---|---|---|
| `clap` | `qed` | CLI argument parsing |
| `regex` | `qed-core` | Pattern compilation and matching |
| `rayon` | `qed-core` | Parallel selector match collection |
| `chumsky` | `qed-core` (optional) | Combinator parser (feature-gated) |
| `libtest-mimic` | `qed-tests` | Test harness registration |
| `toml` | `qed-tests` | Manifest parsing |
