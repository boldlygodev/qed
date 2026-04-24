# EARS Specs — Generation

ID prefix: `GEN`

## Generative Processor Contract

- [x] GEN-001: All generative processors SHALL ignore the `input: &str` argument to `execute()`.
- [x] GEN-002: All generative processors SHALL return `Ok(generated_content + "\n")` — the `'\n'` append is unconditional.
- [x] GEN-003: Generative processors SHALL read OS state (system clock or OS randomness) and SHALL have no other side effects.
- [x] GEN-004: Generative processors SHALL require regex-based (`.pattern`) or glob (`.*`) golden files in tests, not exact-match goldens, because their output is non-deterministic.

## UUID Generation

- [x] GEN-010: `UuidProcessor` with `version: V4` SHALL generate a random UUID using `uuid::Uuid::new_v4()`.
- [x] GEN-011: `UuidProcessor` with `version: V5` SHALL generate a deterministic, name-based SHA-1 UUID given a pre-parsed `namespace: uuid::Uuid` and `name: String`.
- [x] GEN-012: `UuidProcessor` with `version: V7` SHALL generate a time-ordered, random-suffix UUID using `uuid::Uuid::now_v7()` suitable for sortable IDs.
- [x] GEN-013: All UUID variants SHALL produce lowercase hyphenated strings (e.g. `550e8400-e29b-41d4-a716-446655440000`).
- [x] GEN-014: UUID v5 namespace and name SHALL be pre-parsed at compile time and stored in the struct; invalid namespace values SHALL fail at compile time.
- [ ] GEN-015: The computed UUID v5 value for `url` namespace + `"https://example.com"` (`c5c17c18-a4a4-5a46-bcd1-b7d8e9c05694`) SHOULD be pinned in an exact-match golden file rather than matched by a `.*` glob.

## Timestamp Generation

- [x] GEN-020: `TimestampProcessor` with `format: Iso8601` and `timezone: Utc` SHALL produce a timestamp using the `Z` suffix (e.g. `2024-01-01T12:00:00Z`).
- [x] GEN-021: `TimestampProcessor` with `format: Iso8601` and a fixed-offset or IANA timezone SHALL produce a timestamp using `%:z` offset notation (e.g. `2024-01-01T12:00:00+05:30`).
- [x] GEN-022: `TimestampProcessor` with `format: Unix` SHALL produce seconds since epoch as an integer string.
- [x] GEN-023: `TimestampProcessor` with `format: UnixMs` SHALL produce milliseconds since epoch as a 13-digit integer string.
- [x] GEN-024: `TimestampProcessor` with `format: Date` SHALL produce `yyyy-MM-dd`.
- [x] GEN-025: `TimestampProcessor` with `format: Time` SHALL produce `HH:mm:ss`.
- [x] GEN-026: `TimestampProcessor` with `format: DateTime` SHALL produce `yyyy-MM-dd HH:mm:ss`.
- [x] GEN-027: `TimestampProcessor` with `format: Custom(String)` SHALL convert the LDML format string to strftime via `ldml_to_strftime` before formatting.
- [x] GEN-028: `ldml_to_strftime` SHALL convert: `yyyy→%Y`, `MM→%m`, `dd→%d`, `HH→%H`, `mm→%M`, `ss→%S`; all other LDML tokens SHALL pass through verbatim.
- [x] GEN-029: `Timezone::Iana` SHALL use the `chrono_tz` IANA timezone database.
- [x] GEN-030: `Timezone::Fixed` SHALL accept only `UTC+H:MM` and `UTC-H` forms, parsed at compile time.
- [ ] GEN-031: Unsupported LDML tokens in a `Custom` format string SHOULD produce a compile-time warning rather than passing through silently.

## Random String Generation

- [x] GEN-040: `RandomProcessor.execute()` SHALL generate `length` characters sampled uniformly from `alphabet` using `rand::rng()`.
- [x] GEN-041: WHEN `alphabet` is empty, `RandomProcessor.execute()` SHALL return `ProcessorFailed`.
- [x] GEN-042: Built-in alphabet shortcuts (`alpha`, `alnum`, `numeric`, `hex`) SHALL be expanded to their character sets at compile time.

## Non-Features

- [D] GEN-050: Generative processors SHALL NOT attempt to preserve the input's trailing-newline semantics; the `'\n'` append is always unconditional.
- [D] GEN-051: `ldml_to_strftime` SHALL NOT implement the full LDML specification; only the 6 core formatting tokens are intentionally supported.

## References

- `qed-core/src/processor/uuid.rs`
- `qed-core/src/processor/timestamp.rs`
- `qed-core/src/processor/random.rs`
- `docs/llds/generation.md`
