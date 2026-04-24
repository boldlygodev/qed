# Generation

## Context and Design Philosophy

Implements the `Processor` trait for operations that produce new content, ignoring their input entirely. These three processors share a distinct behavioral contract: they are **generative**, not transformative. They replace selected content with freshly generated output. All append `'\n'` unconditionally — the selected region is treated as a placeholder to be replaced, not text to be shaped.

Segregated from `text-transformation` because the behavioral contract, testing strategy, and external dependencies are fundamentally different: generators read OS state (clock, RNG), produce non-deterministic output, and require regex-based golden files rather than exact-match comparisons.

## Generative Processor Contract

All three processors:
- Ignore the `input: &str` argument to `execute()`
- Return `Ok(generated_content + "\n")`
- Read OS state (system clock or OS randomness source)
- Have no side effects beyond reading OS state

This `'\n'` append is unconditional — unlike transformative processors, which preserve the input's trailing newline semantics. A generator always occupies a full line in the output.

## UUID Generation

`UuidProcessor { version: UuidVersion }` via the `uuid` crate:

- **V4** — random; reads OS randomness (`uuid::Uuid::new_v4()`)
- **V5** — name-based SHA-1; deterministic given `namespace: uuid::Uuid` and `name: String`. Both are pre-parsed at compile time and stored in the struct. The computed value for `url` namespace + `"https://example.com"` is `c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694` — this value was flagged for verification in `.claude/tests/generation.md` and is currently tested with a `.*` glob rather than an exact match.
- **V7** — time-ordered + random; suitable for sortable IDs (`uuid::Uuid::now_v7()`)

All three produce lowercase hyphenated UUID strings (e.g. `550e8400-e29b-41d4-a716-446655440000\n`).

## Timestamp Generation

`TimestampProcessor { format: TimestampFormat, timezone: Timezone }` via `chrono` and `chrono_tz`:

**Formats** (`TimestampFormat`):
- `Iso8601` — UTC uses `Z` suffix (`%Y-%m-%dT%H:%M:%SZ`); fixed-offset and IANA use `%:z` — these produce divergent representations
- `Unix` — seconds since epoch as integer string
- `UnixMs` — milliseconds since epoch (13 digits)
- `Date` — `yyyy-MM-dd`
- `Time` — `HH:mm:ss`
- `DateTime` — `yyyy-MM-dd HH:mm:ss`
- `Custom(String)` — LDML format string, converted to strftime via `ldml_to_strftime`

**Timezones** (`Timezone`):
- `Utc` — `chrono::Utc`
- `Iana(chrono_tz::Tz)` — IANA timezone database via `chrono_tz`
- `Fixed(FixedOffset)` — parsed at compile time via `parse_fixed_offset`; accepts `UTC+H:MM` and `UTC-H` forms only

**`ldml_to_strftime`** — a compile-time helper exposed `pub(crate)` (used by `compile/mod.rs`). Handles `yyyy→%Y`, `MM→%m`, `dd→%d`, `HH→%H`, `mm→%M`, `ss→%S`; other LDML tokens pass through verbatim. Unsupported tokens produce unexpected output silently.

## Random String Generation

`RandomProcessor { length: usize, alphabet: String }`:

- Generates `length` characters sampled uniformly from `alphabet` using `rand::rng()` (rand 0.9+ API)
- Empty `alphabet` returns `ProcessorFailed`
- `alphabet` chars are collected into a `Vec<char>` on every `execute()` call — not cached at construction
- Built-in alphabet shortcuts are resolved at compile time: `alpha`, `alnum`, `numeric`, `hex` → expanded to their character sets

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Unconditional `'\n'` append | Always append regardless of input | Preserve input's newline semantics | Generators are replacements, not transforms; the selected placeholder line is consumed entirely. [inferred] |
| UUID v5 namespace pre-parsed at compile time | `uuid::Uuid` stored in struct | Parse at execution time | Fail fast on invalid namespace values; no repeated parsing overhead per call. [inferred] |
| `ldml_to_strftime` covering limited LDML subset | Partial LDML (6 tokens) | Full LDML implementation | Limited subset covers the common formatting tokens; a full LDML implementation is significant scope. [inferred] |
| `parse_fixed_offset` limited forms | `UTC+H:MM` and `UTC-H` only | Full RFC 3339 offset parsing | Covers the expected user-facing forms; full RFC 3339 parsing would be broader than needed. [inferred] |
| Alphabet as `String` (not `Vec<char>`) | Store as `String`; convert per call | Store pre-computed `Vec<char>` | Simpler struct definition; conversion cost is low for typical alphabet sizes. [inferred — possibly an oversight] |

## Open Questions & Future Decisions

### Resolved
*(none yet)*

### Deferred
1. **UUID v5 exact value** — Pin the v5 UUID value in tests or document explicitly why glob golden is preferred.
2. **`ldml_to_strftime` LDML coverage** — Document the supported subset in user-facing documentation. Consider whether unsupported tokens should produce an error at compile time rather than passing through silently.
3. **`ISO8601` UTC vs offset format divergence** — Should UTC and fixed-offset produce the same canonical form (e.g. always `%:z`)? Current divergence (`Z` vs offset) may surprise users converting between timezone forms.
4. **`RandomProcessor` alphabet caching** — Cache `Vec<char>` at construction time rather than rebuilding per call?

## References

- `qed-core/src/processor/uuid.rs`
- `qed-core/src/processor/timestamp.rs`
- `qed-core/src/processor/random.rs`
- `docs/qed-design.md` — generation processor specifications
- `docs/arrows/generation.md`
- `docs/specs/generation-specs.md`
