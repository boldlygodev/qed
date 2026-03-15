# `qed` Rust Conventions

Codebase-specific Rust patterns and conventions for `qed`.
Covers the idioms that appear repeatedly across the codebase so they are
consistent from the first file.

---

## Error Handling

### The `?` operator

`?` is Rust's primary error propagation mechanism.
In a function that returns `Result<T, E>`, appending `?` to a fallible
call either unwraps the `Ok` value or returns the `Err` early.

```rust
// Without ?
fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = match lex(source) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    // ...
}

// With ?
fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = lex(source)?;   // returns early on error
    // ...
}
```

`?` also calls `.into()` on the error, so it automatically converts
between compatible error types.
This is why a function can use `?` on calls that return different error
types as long as those types both implement `Into<YourErrorType>`.

### Collecting errors before returning

The compilation pass collects all errors rather than failing on the first.
This pattern uses a `Vec<CompileError>` accumulator alongside sentinel
values that let compilation continue past a bad node.

```rust
fn compile(program: Program) -> Result<Script, Vec<CompileError>> {
    let mut errors = Vec::new();

    let selector = match compile_selector(&program.selector) {
        Ok(s) => s,
        Err(e) => {
            errors.push(e);
            CompiledSelector::invalid()   // sentinel — compilation continues
        }
    };

    if errors.is_empty() {
        Ok(Script { selector, /* ... */ })
    } else {
        Err(errors)
    }
}
```

Use `?` for single-error early-return paths (parser, executor).
Use the accumulator pattern for the compilation pass only.

### `unwrap()` and `expect()`

`unwrap()` panics if the value is `Err` or `None`.
Use it only in tests or in situations that are genuinely impossible at runtime.
Prefer `expect("reason")` over bare `unwrap()` — the message shows up in the panic output.

```rust
// Good — impossible case, reason documented
let offset = self.line_offsets.get(line).expect("line index validated at construction");

// Bad — should propagate the error instead
let tokens = lex(source).unwrap();
```

---

## Visibility

Rust defaults to private.
Use the minimum visibility needed.

| Keyword | Visible to |
|---|---|
| _(nothing)_ | Current module only |
| `pub(crate)` | Anywhere in `qed-core` |
| `pub` | Public API — accessible from `qed` binary and `qed-tests` |

Use `pub` only on types and functions that are part of `qed-core`'s
intentional public surface.
Use `pub(crate)` for things that cross module boundaries internally.
Leave everything else private.

```rust
// Public API — the binary calls this
pub fn parse(source: &str) -> Result<Program, Vec<ParseError>> { /* ... */ }

// Internal — used across modules inside qed-core, not exposed outside
pub(crate) fn compile_pattern(value: &PatternValue) -> Result<CompiledPattern, CompileError> { /* ... */ }

// Private — only used within this module
fn lex_string_literal(cursor: &mut Cursor) -> Result<String, ParseError> { /* ... */ }
```

---

## Naming

Follow standard Rust naming conventions throughout.

| Item | Convention | Example |
|---|---|---|
| Types, traits, enums, variants | `UpperCamelCase` | `CompiledSelector`, `SelectorOp` |
| Functions, methods, variables | `snake_case` | `collect_matches`, `line_offset` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_LINE_LENGTH` |
| Modules | `snake_case` | `mod compile;`, `mod exec;` |
| Lifetimes | Short lowercase | `'a`, `'src` |

Avoid abbreviations unless they are universally understood (`buf`, `idx`, `err`).
Prefer full words: `selector` not `sel`, `processor` not `proc`.

---

## The Newtype Pattern

`StatementId` and `SelectorId` are newtypes — wrapper structs around `usize`.
They exist to prevent accidentally passing a `StatementId` where a `SelectorId`
is expected, which would be a silent logic error if both were bare `usize`.

```rust
struct StatementId(usize);
struct SelectorId(usize);
```

Accessing the inner value uses `.0`:

```rust
let id = StatementId(3);
println!("id = {}", id.0);
```

When you see a single-field tuple struct in this codebase, it is a newtype —
read it as a typed wrapper, not a data container.

---

## Trait Objects (`Box<dyn Trait>`)

`Box<dyn Processor>` is how the codebase stores processors polymorphically.
`dyn Trait` is a trait object — a pointer to a value whose concrete type is
not known at compile time, plus a vtable for dynamic dispatch.
`Box` owns the allocation.

```rust
// The Processor trait
trait Processor {
    fn execute(&self, input: String) -> Result<String, ProcessorError>;
}

// Storing a mix of concrete types behind the trait
let processors: Vec<Box<dyn Processor>> = vec![
    Box::new(DeleteProcessor),
    Box::new(UpperProcessor),
    Box::new(ExternalProcessor { command: "tr".into(), args: vec![] }),
];
```

You rarely construct `Box<dyn Processor>` directly in calling code —
the compilation pass builds these from the AST and stores them in `Statement`.

When implementing a new processor, implement `Processor` on a plain struct.
The `Box` wrapping happens at the callsite in the compilation pass.

---

## `Spanned<T>`

Every AST node is wrapped in `Spanned<T>`, which pairs the node with its
source location for use in diagnostic messages.

```rust
struct Span {
    start: usize,   // byte offset into source, inclusive
    end: usize,     // byte offset into source, exclusive
}

struct Spanned<T> {
    node: T,
    span: Span,
}
```

Access the inner value via `.node` and the location via `.span`.

```rust
fn compile_selector(sel: &Spanned<Selector>) -> Result<CompiledSelector, CompileError> {
    let selector = &sel.node;
    let span = sel.span;
    // use span in any CompileError produced here
}
```

---

## Enums and Pattern Matching

Rust enums carry data per variant — use them instead of stringly-typed
discriminators or parallel boolean fields.

```rust
// Good — the variant IS the type information
enum PatternMatcher {
    Literal(String),
    Regex(regex::Regex),
}

// Bad — parallel fields that can be inconsistent
struct PatternMatcher {
    is_regex: bool,
    value: String,
    compiled: Option<regex::Regex>,
}
```

Pattern matching with `match` must be exhaustive — the compiler enforces
that every variant is handled.
Use `_` only when there is a genuine "everything else" case, never to
suppress exhaustiveness warnings.

```rust
match matcher {
    PatternMatcher::Literal(s) => s.contains(line),
    PatternMatcher::Regex(re) => re.is_match(line),
    // No _ — if a new variant is added, this will fail to compile,
    // which is exactly what we want.
}
```

---

## Ownership and Borrowing

The fragment model is designed to minimize allocation.
Understanding the core ownership rules helps keep that property intact.

- **Owned** (`String`, `Vec<T>`) — the value owns its memory; moving it
  transfers ownership.
- **Borrowed** (`&str`, `&[T]`) — a reference to memory owned elsewhere;
  the borrow cannot outlive the owner.

In the fragment model, `Borrowed(LineRange)` fragments hold a `LineRange`
(two `usize` indices) rather than a `&str` directly.
This sidesteps lifetime complexity — the `Buffer` owns the string, and
code that needs a `&str` slice calls into the buffer with the range.

```rust
// Getting a &str from a LineRange — no allocation
fn slice<'a>(&'a self, range: LineRange) -> &'a str {
    let start = self.line_offsets[range.start];
    let end = self.line_offsets[range.end];
    &self.content[start..end]
}
```

If you find yourself calling `.to_string()` or `.to_owned()` in a hot path
to resolve a lifetime error, stop and think about whether the ownership
model needs to be adjusted instead.

---

## Module Declarations

Modules are declared with `mod` in their parent's `mod.rs` or `lib.rs`.
The module's code lives in either `module_name.rs` or `module_name/mod.rs`.

```rust
// In qed-core/src/lib.rs
pub mod span;           // → src/span.rs
pub(crate) mod error;   // → src/error.rs
pub mod parse;          // → src/parse/mod.rs
pub(crate) mod compile; // → src/compile/mod.rs
```

Prefer `module_name.rs` for leaf modules (no submodules).
Use `module_name/mod.rs` when the module has its own submodules.

---

## Doc Comments

Public items must have doc comments.
`pub(crate)` items should have doc comments where the purpose is not
immediately obvious from the name and type signature.

```rust
/// A byte-offset range into a `Span` source string.
///
/// `start` is inclusive, `end` is exclusive — following Rust slice conventions.
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

Use `///` (triple-slash) for doc comments on items.
Use `//` for inline explanatory comments within function bodies.
Avoid restating what the type signature already says.

---

## Feature-Gated Code

Parser implementations are gated with `#[cfg(feature = "...")]`.
Always pair a `cfg` attribute with a comment explaining why it exists.

```rust
// Recursive descent parser — active when parser-rd feature is enabled (default)
#[cfg(feature = "parser-rd")]
mod rd;

// Chumsky combinator parser — active when parser-chumsky feature is enabled
#[cfg(feature = "parser-chumsky")]
mod chumsky;
```

Do not use `#[cfg(feature = "...")]` outside of `parse/mod.rs` without a
very good reason.
The feature flags exist solely to gate the two parser POC implementations.
