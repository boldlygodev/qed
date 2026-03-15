# Use Case Scenarios

End-to-end scenarios grounded in realistic use cases from `qed-examples.md`.
Each suite lives in its own directory under `tests/usecases/` and has its own
`manifest.toml`.
Scenarios test realistic feature combinations that the unit-style feature tests
do not exercise together.

---

## Code Editing

```
tests/usecases/code-editing/
  manifest.toml
  inputs/
    main-with-todos.go
    main-with-function.go
    main-with-old-funcs.go
  scripts/
    delete-todo-comments.qed
    delete-function.qed
    add-deprecation-notice.qed
  goldens/
    stdout/
      no-todos.go
      no-function.go
      with-deprecation.go
      empty.txt
    stderr/
      empty.txt
    output/
      no-todos.go
      no-function.go
      with-deprecation.go
      empty.txt
```

### Inputs

#### `inputs/main-with-todos.go`

Used by: `delete-todo-comments`

```go
package main

// TODO: add error handling
func main() {
	run()
}

// TODO: implement retries
func run() {
}
```

#### `inputs/main-with-function.go`

Used by: `delete-function`

```go
package main

func keep() {
	println("keep")
}

func handleRequest() {
	println("handle")
}

func alsoKeep() {
	println("also keep")
}
```

#### `inputs/main-with-old-funcs.go`

Used by: `add-deprecation-notice`

```go
package main

func OldSend() {
}

func OldReceive() {
}
```

### Scripts

#### `scripts/delete-todo-comments.qed`

```
at(/^\s*\/\/\s*TODO:/) | qed:delete()
```

#### `scripts/delete-function.qed`

```
func_start=/^func handleRequest\(/
func_end=/^\}/
from(func_start+) > to(func_end+) | qed:delete()
```

#### `scripts/add-deprecation-notice.qed`

```
target=/^func Old/
before(target) | echo "// Deprecated: use New* equivalent instead."
```

### Manifest

```toml
# tests/usecases/code-editing/manifest.toml

[[scenario]]
id = "delete-todo-comments"
description = "delete all TODO comment lines from a Go source file"
script = "delete-todo-comments.qed"
input = "main-with-todos.go"
stdout = "no-todos.go"
stderr = "empty.txt"
output = "no-todos.go"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "delete-function"
description = "delete a named function and its body using from > to inclusive boundaries"
script = "delete-function.qed"
input = "main-with-function.go"
stdout = "no-function.go"
stderr = "empty.txt"
output = "no-function.go"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "add-deprecation-notice"
description = "insert a deprecation comment before every function matching a naming convention"
script = "add-deprecation-notice.qed"
input = "main-with-old-funcs.go"
stdout = "with-deprecation.go"
stderr = "empty.txt"
output = "with-deprecation.go"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/no-todos.go` and `goldens/output/no-todos.go`

Used by: `delete-todo-comments`

Both TODO lines deleted; `package main`, `func main()`, and `func run()` remain.

```go
package main

func main() {
	run()
}

func run() {
}
```

#### `goldens/stdout/no-function.go` and `goldens/output/no-function.go`

Used by: `delete-function`

`handleRequest` and its body deleted; `keep` and `alsoKeep` remain.
The blank line between `keep` and `handleRequest` is part of the passthrough —
the blank line between the deleted function and `alsoKeep` is also deleted as part
of the inclusive `to(func_end+)` region.

```go
package main

func keep() {
	println("keep")
}

func alsoKeep() {
	println("also keep")
}
```

#### `goldens/stdout/with-deprecation.go` and `goldens/output/with-deprecation.go`

Used by: `add-deprecation-notice`

Deprecation comment inserted before each `Old*` function by `before()` + `echo`.
`echo` appends a newline, so each inserted line is complete.

```go
package main

// Deprecated: use New* equivalent instead.
func OldSend() {
}

// Deprecated: use New* equivalent instead.
func OldReceive() {
}
```

---

## Config Manipulation

```
tests/usecases/config-manipulation/
  manifest.toml
  inputs/
    Cargo.toml
    config.ini
    deployment.yaml
  scripts/
    update-toml-version.qed
    comment-out-ini-section.qed
    delete-yaml-block.qed
  goldens/
    stdout/
      Cargo-updated.toml
      config-commented.ini
      deployment-no-annotations.yaml
      empty.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

### Inputs

#### `inputs/Cargo.toml`

Used by: `update-toml-version`

```toml
[package]
name = "myapp"
version = "1.0.0"
edition = "2021"
```

#### `inputs/config.ini`

Used by: `comment-out-ini-section`

```ini
[server]
host = localhost
port = 8080

[database]
host = db.internal
port = 5432
name = mydb

[cache]
host = cache.internal
```

#### `inputs/deployment.yaml`

Used by: `delete-yaml-block`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  annotations:
    deploy-time: "2026-01-01"
    owner: "team-a"
  labels:
    app: myapp
```

### Scripts

#### `scripts/update-toml-version.qed`

```
at(/^version = /) | qed:replace(/=.*/, "= \"2.0.0\"")
```

#### `scripts/comment-out-ini-section.qed`

```
section=/^\[database\]/
section_end=/^\[/
from(section+) > to(section_end) | qed:prefix(text:"# ")
```

#### `scripts/delete-yaml-block.qed`

```
key=/^  annotations:/
next_key=/^  [a-z]/
from(key+) > to(next_key) | qed:delete()
```

### Manifest

```toml
# tests/usecases/config-manipulation/manifest.toml

[[scenario]]
id = "update-toml-version"
description = "update the version field in a TOML file using regex replacement"
script = "update-toml-version.qed"
input = "Cargo.toml"
stdout = "Cargo-updated.toml"
stderr = "empty.txt"
output = "Cargo-updated.toml"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "comment-out-ini-section"
description = "comment out an INI section from its header to the start of the next section using from > to with prefix"
script = "comment-out-ini-section.qed"
input = "config.ini"
stdout = "config-commented.ini"
stderr = "empty.txt"
output = "config-commented.ini"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "delete-yaml-block"
description = "delete a YAML key block from its key line to the start of the next sibling key"
script = "delete-yaml-block.qed"
input = "deployment.yaml"
stdout = "deployment-no-annotations.yaml"
stderr = "empty.txt"
output = "deployment-no-annotations.yaml"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/Cargo-updated.toml` and `goldens/output/Cargo-updated.toml`

Used by: `update-toml-version`

```toml
[package]
name = "myapp"
version = "2.0.0"
edition = "2021"
```

#### `goldens/stdout/config-commented.ini` and `goldens/output/config-commented.ini`

Used by: `comment-out-ini-section`

`from(section+) > to(section_end)` — `section` is included, `section_end` (`[cache]`) is
excluded.
The `[database]` header through the last database key are prefixed; `[cache]` passes through.

```ini
[server]
host = localhost
port = 8080

# [database]
# host = db.internal
# port = 5432
# name = mydb
#
[cache]
host = cache.internal
```

#### `goldens/stdout/deployment-no-annotations.yaml` and `goldens/output/deployment-no-annotations.yaml`

Used by: `delete-yaml-block`

`from(key+) > to(next_key)` — `annotations:` included, `labels:` excluded.
The `annotations` block and its child keys are deleted; `labels` and everything
after it passes through.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  labels:
    app: myapp
```

---

## Log Processing

```
tests/usecases/log-processing/
  manifest.toml
  inputs/
    app.log
    time-range.log
  scripts/
    extract-errors.qed
    extract-time-range.qed
    reformat-timestamp.qed
  goldens/
    stdout/
      errors-only.log
      time-range-extracted.log
      reformatted.log
      empty.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

### Inputs

#### `inputs/app.log`

Used by: `extract-errors`, `reformat-timestamp`

```
2026-02-26 09:00:01 INFO  server started
2026-02-26 09:00:02 DEBUG loading config
2026-02-26 09:00:03 ERROR connection refused: db.internal:5432
2026-02-26 09:00:04 INFO  retrying connection
2026-02-26 09:00:05 DEBUG retry attempt 1
2026-02-26 09:00:06 ERROR connection refused: db.internal:5432
2026-02-26 09:00:07 INFO  giving up
```

#### `inputs/time-range.log`

Used by: `extract-time-range`

```
2026-02-26 08:59:00 INFO  pre-window event
2026-02-26 09:00:00 INFO  window start
2026-02-26 09:00:30 INFO  mid-window event
2026-02-26 09:01:00 INFO  window end
2026-02-26 09:01:30 INFO  post-window event
```

### Scripts

#### `scripts/extract-errors.qed`

```
at(/\bERROR\b/) | qed:skip()
```

#### `scripts/extract-time-range.qed`

```
start=/2026-02-26 09:00:00/
end=/2026-02-26 09:01:00/
from(start+) > to(end+) | qed:skip()
```

#### `scripts/reformat-timestamp.qed`

Reformats `yyyy-mm-dd` dates to `dd/mm/yyyy` in every log line.

```
at(/^\d{4}-\d{2}-\d{2}/) | qed:replace(/(\d{4})-(\d{2})-(\d{2})/, /$3\/$2\/$1/)
```

### Manifest

```toml
# tests/usecases/log-processing/manifest.toml

[[scenario]]
id = "extract-errors"
description = "extract only ERROR lines from a log file using --extract"
script = "extract-errors.qed"
input = "app.log"
stdout = "errors-only.log"
stderr = "empty.txt"
output = "errors-only.log"
exit_code = 0
invoke = [
  """qed --extract "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed --extract -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "extract-time-range"
description = "extract a time window from a log file using --extract with from > to inclusive boundaries"
script = "extract-time-range.qed"
input = "time-range.log"
stdout = "time-range-extracted.log"
stderr = "empty.txt"
output = "time-range-extracted.log"
exit_code = 0
invoke = [
  """qed --extract -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "reformat-timestamp"
description = "reformat yyyy-mm-dd timestamps to dd/mm/yyyy in every log line using capture group replacement"
script = "reformat-timestamp.qed"
input = "app.log"
stdout = "reformatted.log"
stderr = "empty.txt"
output = "reformatted.log"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/errors-only.log` and `goldens/output/errors-only.log`

Used by: `extract-errors`

Only the two ERROR lines; INFO and DEBUG lines suppressed by `--extract`.

```
2026-02-26 09:00:03 ERROR connection refused: db.internal:5432
2026-02-26 09:00:06 ERROR connection refused: db.internal:5432
```

#### `goldens/stdout/time-range-extracted.log` and `goldens/output/time-range-extracted.log`

Used by: `extract-time-range`

`from(start+) > to(end+)` inclusive on both ends; pre- and post-window lines suppressed.

```
2026-02-26 09:00:00 INFO  window start
2026-02-26 09:00:30 INFO  mid-window event
2026-02-26 09:01:00 INFO  window end
```

#### `goldens/stdout/reformatted.log` and `goldens/output/reformatted.log`

Used by: `reformat-timestamp`

Every `yyyy-mm-dd` date rewritten as `dd/mm/yyyy`; rest of each line unchanged.

```
26/02/2026 09:00:01 INFO  server started
26/02/2026 09:00:02 DEBUG loading config
26/02/2026 09:00:03 ERROR connection refused: db.internal:5432
26/02/2026 09:00:04 INFO  retrying connection
26/02/2026 09:00:05 DEBUG retry attempt 1
26/02/2026 09:00:06 ERROR connection refused: db.internal:5432
26/02/2026 09:00:07 INFO  giving up
```

---

## Code Generation

```
tests/usecases/code-generation/
  manifest.toml
  inputs/
    routes.go
    version.go
  scripts/
    inject-between-markers.qed
    stamp-build-version.qed
  mocks/
    input/
      routes-region.txt
    stdout/
      generated-routes.txt
  goldens/
    stdout/
      routes-generated.go
      version-stamped.go
      empty.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

### Inputs

#### `inputs/routes.go`

Used by: `inject-between-markers`

```go
package main

// CODE GENERATED START
func oldRoute() {}
// CODE GENERATED END

func main() {}
```

#### `inputs/version.go`

Used by: `stamp-build-version`

```go
package main

var Version = "dev"
```

### Scripts

#### `scripts/inject-between-markers.qed`

```
marker_start=/\/\/ CODE GENERATED START/
marker_end=/\/\/ CODE GENERATED END/
from(marker_start+) > to(marker_end) | generate-routes
```

#### `scripts/stamp-build-version.qed`

```
at(/^var Version = /) | qed:replace(/=.*/, "= \"${BUILD_VERSION}\"")
```

### Mock Files

#### `mocks/input/routes-region.txt`

The content delivered to `generate-routes` — the region from `marker_start+` to
`marker_end` (exclusive), which is the existing generated content between the markers.

```
// CODE GENERATED START
func oldRoute() {}
```

#### `mocks/stdout/generated-routes.txt`

Fresh output from `generate-routes`, replacing the old content.

```
// CODE GENERATED START
func newRoute() {}
func anotherRoute() {}
```

### Manifest

```toml
# tests/usecases/code-generation/manifest.toml

[[scenario]]
id = "inject-between-markers"
description = "replace the region between generation markers with fresh output from an external generator"
script = "inject-between-markers.qed"
input = "routes.go"
stdout = "routes-generated.go"
stderr = "empty.txt"
output = "routes-generated.go"
exit_code = 0
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario.mock]]
command = "generate-routes"
input = "routes-region.txt"
stdout = "generated-routes.txt"

[[scenario]]
id = "stamp-build-version"
description = "replace the version sentinel with a build version injected via environment variable"
script = "stamp-build-version.qed"
input = "version.go"
stdout = "version-stamped.go"
stderr = "empty.txt"
output = "version-stamped.go"
exit_code = 0
env = { BUILD_VERSION = "1.4.2" }
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/routes-generated.go` and `goldens/output/routes-generated.go`

Used by: `inject-between-markers`

Old generated content replaced by mock output; `marker_end` line and `func main()`
pass through unchanged.

```go
package main

// CODE GENERATED START
func newRoute() {}
func anotherRoute() {}
// CODE GENERATED END

func main() {}
```

#### `goldens/stdout/version-stamped.go` and `goldens/output/version-stamped.go`

Used by: `stamp-build-version`

`BUILD_VERSION` env var expanded at compile time; `= "dev"` replaced.

```go
package main

var Version = "1.4.2"
```

---

## Template Rendering

```
tests/usecases/template-rendering/
  manifest.toml
  inputs/
    template.yaml
    template.sql
  scripts/
    replace-env-placeholders.qed
    inject-uuid.qed
  goldens/
    stdout/
      rendered.yaml
      uuid-injected.sql
      uuid-injected.sql.pattern
      empty.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

### Inputs

#### `inputs/template.yaml`

Used by: `replace-env-placeholders`

```yaml
name: {{APP_NAME}}
version: {{VERSION}}
image: myrepo/{{APP_NAME}}:{{VERSION}}
```

#### `inputs/template.sql`

Used by: `inject-uuid`

```sql
INSERT INTO jobs (id, name) VALUES ('{{uuid}}', 'batch-job');
```

### Scripts

#### `scripts/replace-env-placeholders.qed`

```
at(/\{\{APP_NAME\}\}/) | qed:replace("{{APP_NAME}}", "${APP_NAME}")
at(/\{\{VERSION\}\}/)  | qed:replace("{{VERSION}}", "${VERSION}")
```

#### `scripts/inject-uuid.qed`

```
at(/\{\{uuid\}\}/) | qed:replace("{{uuid}}", qed:uuid())
```

### Manifest

```toml
# tests/usecases/template-rendering/manifest.toml

[[scenario]]
id = "replace-env-placeholders"
description = "replace multiple {{PLACEHOLDER}} tokens with environment variable values using sequential statements"
script = "replace-env-placeholders.qed"
input = "template.yaml"
stdout = "rendered.yaml"
stderr = "empty.txt"
output = "rendered.yaml"
exit_code = 0
env = { APP_NAME = "myservice", VERSION = "3.1.0" }
invoke = [
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "inject-uuid"
description = "replace a {{uuid}} placeholder with a generated UUID v7; output matches UUID v7 format"
script = "inject-uuid.qed"
input = "template.sql"
stdout = "uuid-injected.sql.*"
stderr = "empty.txt"
output = "uuid-injected.sql.*"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/rendered.yaml` and `goldens/output/rendered.yaml`

Used by: `replace-env-placeholders`

Both placeholders replaced in all three lines where they appear.
Statement 1 replaces `{{APP_NAME}}`; statement 2 sees the output of statement 1
and replaces `{{VERSION}}`.

```yaml
name: myservice
version: 3.1.0
image: myrepo/myservice:3.1.0
```

#### `goldens/stdout/uuid-injected.sql.pattern` and `goldens/output/uuid-injected.sql.pattern`

Used by: `inject-uuid`

`qed:uuid()` produces a non-deterministic UUID v7; exact comparison is not possible.
The pattern matches the full output line with a UUID v7 in the values clause.

```
^INSERT INTO jobs \(id, name\) VALUES \('[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}', 'batch-job'\);$
```

> No `uuid-injected.sql.txt` is provided — the glob `uuid-injected.sql.*` resolves
> to only the `.pattern` file and runs one comparison.

---

## Document Processing

```
tests/usecases/document-processing/
  manifest.toml
  inputs/
    README.md
  scripts/
    promote-headings.qed
    extract-code-blocks.qed
    wrap-long-lines.qed
  goldens/
    stdout/
      README-promoted.md
      code-blocks-extracted.md
      README-wrapped.md
      empty.txt
    stderr/
      empty.txt
    output/
      (same filenames and content as goldens/stdout/)
```

### Inputs

#### `inputs/README.md`

Used by: `promote-headings`, `extract-code-blocks`, `wrap-long-lines`

```markdown
# Project

## Installation

Install with:

```sh
cargo install myapp
```

## Usage

This is a very long line that explains everything about the usage of this tool in excessive detail and should be wrapped.

### Options

```sh
myapp --help
```
```

### Scripts

#### `scripts/promote-headings.qed`

Promotes every `##` heading to `#` (one level up).

```
at(/^##/) | qed:replace(/^##/, "#")
```

#### `scripts/extract-code-blocks.qed`

Extracts all fenced code blocks including their fence lines.

```
fence=/^```/
from(fence+) > to(fence+) | qed:skip()
```

#### `scripts/wrap-long-lines.qed`

Wraps lines longer than 80 characters that are not table rows.

```
at(/^[^|].{80,}/) | qed:wrap(width:80)
```

### Manifest

```toml
# tests/usecases/document-processing/manifest.toml

[[scenario]]
id = "promote-headings"
description = "promote all ## headings to # by replacing the leading ## prefix"
script = "promote-headings.qed"
input = "README.md"
stdout = "README-promoted.md"
stderr = "empty.txt"
output = "README-promoted.md"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "extract-code-blocks"
description = "extract all fenced code blocks including their fence lines using --extract with from > to inclusive"
script = "extract-code-blocks.qed"
input = "README.md"
stdout = "code-blocks-extracted.md"
stderr = "empty.txt"
output = "code-blocks-extracted.md"
exit_code = 0
invoke = [
  """qed --extract -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]

[[scenario]]
id = "wrap-long-lines"
description = "wrap lines longer than 80 characters that are not table rows"
script = "wrap-long-lines.qed"
input = "README.md"
stdout = "README-wrapped.md"
stderr = "empty.txt"
output = "README-wrapped.md"
exit_code = 0
invoke = [
  """qed "$(cat "$SCRIPT")" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
  """qed -f "$SCRIPT" < "$INPUT" 2> "$STDERR" | tee "$OUTPUT" > "$STDOUT" """,
]
```

### Goldens

#### `goldens/stdout/README-promoted.md` and `goldens/output/README-promoted.md`

Used by: `promote-headings`

`## Installation`, `## Usage`, and `### Options` promoted by one level each.
`# Project` has no `##` prefix and passes through unchanged.

```markdown
# Project

# Installation

Install with:

```sh
cargo install myapp
```

# Usage

This is a very long line that explains everything about the usage of this tool in excessive detail and should be wrapped.

## Options

```sh
myapp --help
```
```

#### `goldens/stdout/code-blocks-extracted.md` and `goldens/output/code-blocks-extracted.md`

Used by: `extract-code-blocks`

Both fenced blocks extracted with their fence lines; prose lines suppressed by `--extract`.

````markdown
```sh
cargo install myapp
```
```sh
myapp --help
```
````

#### `goldens/stdout/README-wrapped.md` and `goldens/output/README-wrapped.md`

Used by: `wrap-long-lines`

The long prose line wrapped at 80 characters; all other lines pass through unchanged.

```markdown
# Project

## Installation

Install with:

```sh
cargo install myapp
```

## Usage

This is a very long line that explains everything about the usage of this tool
in excessive detail and should be wrapped.

### Options

```sh
myapp --help
```
```

---

## Editor Integration

```
tests/usecases/editor-integration/
  manifest.toml
  inputs/
    commit-msg.txt
    deployment.yaml
  scripts/
    git-commit-cleanup.qed
    kubectl-enforce-limits.qed
  goldens/
    stdout/
      empty.txt
    stderr/
      empty.txt
    output/
      commit-msg-clean.txt
      deployment-with-limits.yaml
```

### Inputs

#### `inputs/commit-msg.txt`

Used by: `git-commit-cleanup`

A git commit message file as git would open it in `$EDITOR` — user text plus
git's comment lines and trailing whitespace.

```
Fix connection retry logic   

Increase max retries from 3 to 5 and add exponential backoff.   

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch main
# Changes to be committed:
#	modified:   main.go
```

#### `inputs/deployment.yaml`

Used by: `kubectl-enforce-limits`

A Kubernetes deployment spec with a container section but no resource limits.

```yaml
spec:
  containers:
    - name: myapp
      image: myrepo/myapp:latest
      ports:
        - containerPort: 8080
```

### Scripts

#### `scripts/git-commit-cleanup.qed`

Trims trailing whitespace and removes git comment lines — two sequential statements.

```
at(/\s+$/) | qed:replace(/\s+$/, "")
at(/^#/) | qed:delete()
```

#### `scripts/kubectl-enforce-limits.qed`

Inserts resource limit lines after the `limits:` key.

```
limits=/^\s*limits:/
after(limits) |
    printf "          cpu: \"500m\"\n          memory: \"256Mi\"\n"
```

### Manifest

```toml
# tests/usecases/editor-integration/manifest.toml

[[scenario]]
id = "git-commit-cleanup"
description = "trim trailing whitespace and remove git comment lines from a commit message file using --in-place"
script = "git-commit-cleanup.qed"
input = "commit-msg.txt"
stdout = "empty.txt"
stderr = "empty.txt"
output = "commit-msg-clean.txt"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --in-place "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp "$INPUT" "$OUTPUT"
  """,
]

[[scenario]]
id = "kubectl-enforce-limits"
description = "insert resource limit lines after the limits: key using --in-place as an EDITOR substitute"
script = "kubectl-enforce-limits.qed"
input = "deployment.yaml"
stdout = "empty.txt"
stderr = "empty.txt"
output = "deployment-with-limits.yaml"
exit_code = 0
invoke = [
  """
  qed -f "$SCRIPT" --in-place "$INPUT" > "$STDOUT" 2> "$STDERR"
  cp "$INPUT" "$OUTPUT"
  """,
]
```

### Goldens

#### `goldens/output/commit-msg-clean.txt`

Used by: `git-commit-cleanup`

Trailing whitespace trimmed from the first two non-empty lines; all `#` comment
lines deleted.
The blank lines between sections remain — they have no trailing whitespace and
no `#` prefix.

```
Fix connection retry logic

Increase max retries from 3 to 5 and add exponential backoff.

```

#### `goldens/output/deployment-with-limits.yaml`

Used by: `kubectl-enforce-limits`

`printf` inserts two indented resource limit lines after `limits:`.

```yaml
spec:
  containers:
    - name: myapp
      image: myrepo/myapp:latest
      ports:
        - containerPort: 8080
          cpu: "500m"
          memory: "256Mi"
```

---

## Notes

### `inject-between-markers` and mock region content

The `generate-routes` mock receives the content from `marker_start+` to `marker_end`
(exclusive) — that is the `// CODE GENERATED START` line and `func oldRoute() {}`
but not the `// CODE GENERATED END` line.
The mock's `input` declaration (`routes-region.txt`) reflects this precisely.
The mock returns its own `// CODE GENERATED START` line as part of its output —
this replaces the original start line as well as the old content, keeping the
marker intact in the final output.

### `replace-env-placeholders` and sequential semantics

The two statements in `replace-env-placeholders.qed` demonstrate sequential
semantics on the `image:` line.
Statement 1 replaces `{{APP_NAME}}` throughout, transforming
`myrepo/{{APP_NAME}}:{{VERSION}}` to `myrepo/myservice:{{VERSION}}`.
Statement 2 then sees `myrepo/myservice:{{VERSION}}` and replaces `{{VERSION}}`,
producing `myrepo/myservice:3.1.0`.
If the implementation incorrectly re-selected from the original input for
statement 2, the `image:` line would not be fully rendered —
`{{APP_NAME}}` would survive. The golden catches this immediately.

### `git-commit-cleanup` and trailing blank line

The `commit-msg-clean.txt` golden ends with a blank line.
The original input has trailing whitespace on the user-text lines but the blank
lines between sections have no trailing whitespace.
After the `at(/\s+$/)` statement removes trailing whitespace from the prose lines
and `at(/^#/)` deletes the comment block, the final blank line between the prose
and the comment block remains — it matched neither statement.

### `extract-code-blocks` and nested fence matching

`from(fence+) > to(fence+)` with `fence=/^```/` matches opening and closing fence
lines inclusively.
The two code blocks in `README.md` are matched as two independent ranges.
With `--extract`, only the selected ranges are emitted — prose lines between and
around the blocks are suppressed.
The two extracted blocks appear in source order with no separator between them in
the output.

### `kubectl-enforce-limits` golden indentation

The `printf` in `kubectl-enforce-limits.qed` uses hardcoded ten-space indentation
(`          `), matching the indentation level of keys inside the `ports:` block.
The golden reflects this exactly.
In a real `kubectl edit` workflow the indentation would need to match the actual
file structure — this scenario uses a controlled input where the indentation is known.
