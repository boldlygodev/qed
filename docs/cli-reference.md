# qed CLI Reference

```
Usage: qed [OPTIONS] [SCRIPT] [FILE]
```

| Flag                | Short | Description                                                 |
| ------------------- | ----- | ----------------------------------------------------------- |
| `--file <FILE>`     | `-f`  | Load script from a file instead of inline                   |
| `--in-place`        | `-i`  | Modify the input file directly (atomic write)               |
| `--output <FILE>`   | `-o`  | Write output to a file instead of stdout                    |
| `--extract`         | `-x`  | Suppress passthrough — emit only selected regions           |
| `--dry-run`         | `-d`  | Show proposed changes as a unified diff; do not modify      |
| `--on-error <MODE>` |       | Global no-match behaviour: `fail` (default), `warn`, `skip` |
| `--no-env`          |       | Disable `${VAR}` expansion in patterns                      |

---

## `--in-place`

Writes atomically —
the original file is replaced only after the full transformation succeeds.
A failing run leaves the original file unchanged.

## `--extract`

Inverts the output:
only selected regions are emitted, passthrough lines are suppressed.
Use `qed:skip()` as the processor to select without transforming:

```sh
qed --extract 'at(/\bERROR\b/) | qed:skip()' app.log
# shorthand — qed:skip() is implied when --extract is used with no processor
qed --extract 'at(/\bERROR\b/)' app.log
```

## `--dry-run`

Produces a unified diff on stdout;
the input file is never modified.
Composable with diff viewers:

```sh
qed --dry-run --file transform.qed main.go | delta
```

## `--on-error`

Sets the global no-match behaviour,
overridable per-selector with the `on_error` parameter.
See [`on_error`](language-reference.md#on_error--no-match-behaviour) in the language reference.

## Pipelines and `set -o pipefail`

`qed` emits lines as soon as they are processed —
it does not buffer the entire output before writing.
If a downstream command exits early (e.g. `head`), `qed` may receive `SIGPIPE`.
Use `set -o pipefail` in scripts that pipe `qed` output to catch failures in any stage:

```sh
set -o pipefail
qed 'at(/\bERROR\b/)' app.log | wc -l
```
