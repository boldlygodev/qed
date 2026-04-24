# Arrow: generation

Processors that produce new content, ignoring their input — UUID, timestamp, and random string generation.

## Status

**MAPPED** — last audited 2026-04-24 (git SHA `null`).
Brownfield mapping pass; no code annotations yet.

## References

### HLD
- `docs/high-level-design.md` — Approach section (processor primitive)

### LLD
- `docs/llds/generation.md`

### EARS
- `docs/specs/generation-specs.md`

### Tests
- `tests/generation/` — 16 scenarios (uuid v4/v5/v7, timestamp formats/timezones, random alphabets)
- `tests/generation-edge-cases/` — 9 scenarios (multiple placeholders, before() composition, boundary lengths)
- All goldens use `.pattern` (regex) files; uuid-v5 uses `.*` glob

### Code
- `qed-core/src/processor/uuid.rs`
- `qed-core/src/processor/timestamp.rs`
- `qed-core/src/processor/random.rs`

## Architecture

**Purpose:** Implements the `Processor` trait for non-deterministic, generative operations. All three processors ignore their input string and produce fresh content. All append `'\n'` unconditionally — a distinct pattern from transformative processors.

**Key Components:**
1. `UuidProcessor` — generates UUID v4 (random), v5 (name-based SHA-1, namespace + name pre-parsed at compile time), or v7 (time-ordered + random); all via the `uuid` crate
2. `TimestampProcessor` — generates formatted timestamps using `chrono`/`chrono_tz`; supports ISO 8601, Unix, Unix-ms, date-only, time-only, datetime, and Custom (LDML subset via `ldml_to_strftime`)
3. `RandomProcessor` — generates a random string of configurable length from a configurable alphabet; uses `rand` crate

## Spec Coverage

| Category | Spec IDs | Implemented | Deferred | Gaps |
|---|---|---|---|---|
| UUID generation | GEN-001–GEN-010 | *(to be filled)* | 0 | *(to be filled)* |
| Timestamp generation | GEN-011–GEN-025 | *(to be filled)* | 0 | *(to be filled)* |
| Random string generation | GEN-026–GEN-035 | *(to be filled)* | 0 | *(to be filled)* |

**Summary:** Spec coverage to be populated during EARS authoring session.

## Key Findings

1. **Unconditional `'\n'` append** — All three processors append `'\n'` to output (`uuid.rs:33`, `timestamp.rs:53`, `random.rs:31`), making them generative replacements rather than in-place transforms. This is a distinct contract from all transformative processors.
2. **`ldml_to_strftime` covers limited LDML subset** — Only handles `yyyy`, `MM`, `dd`, `HH`, `mm`, `ss`; other LDML tokens pass through verbatim (`timestamp.rs`). Custom format strings with unsupported tokens silently produce unexpected output.
3. **UUID v5 exact value not pinned in tests** — The deterministic v5 UUID (`c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694` for `url` namespace + `https://example.com`) is tested with a `.*` glob golden rather than an exact value. The computed value was flagged for verification in `.claude/tests/generation.md`.
4. **ISO 8601 UTC vs offset format divergence** — UTC uses `Z` suffix (`%Y-%m-%dT%H:%M:%SZ`); fixed-offset uses `%:z`. These produce different format representations for the same wall time (`timestamp.rs:61, 74`).
5. **`RandomProcessor` alphabet `Vec<char>` rebuilt per call** — `random.rs` collects alphabet chars into a `Vec<char>` on every `execute()` call rather than caching at construction time.
6. **`parse_fixed_offset` accepts limited forms** — Only `UTC+H:MM` and `UTC-H` forms; returns `None` for anything else.

## Work Required

### Must Fix
*(none — generation is functionally complete)*

### Should Fix
1. Document `ldml_to_strftime` supported subset explicitly (GEN specs TBD) so users know which LDML tokens are safe to use.
2. Pin uuid-v5 exact value in test or document why glob golden is preferred.

### Nice to Have
1. Cache `Vec<char>` alphabet in `RandomProcessor::new` rather than rebuilding per call.
2. Expand `ldml_to_strftime` to cover more LDML tokens or document the limitation as a non-goal.
