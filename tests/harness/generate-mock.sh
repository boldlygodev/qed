#!/usr/bin/env bash
# generate-mock.sh — create an executable mock script for a single command
# Args: $1=temp directory path, $2=command name
# @spec TINFRA-040, TINFRA-041, TINFRA-042, TINFRA-043, TINFRA-044, TINFRA-045, TINFRA-046
#
# Reads MOCK_* variables from the environment (already sourced from scenario.sh)
# and generates a self-contained script at $TMPDIR/bin/$COMMAND that:
#   - tracks a call counter per command name
#   - dispatches to the Nth declaration on each call
#   - validates stdin input against expected content
#   - validates positional arguments
#   - produces declared stdout/stderr and exits with declared exit code

set -euo pipefail

TMPDIR="$1"
COMMAND="$2"

# Source scenario variables so MOCK_* are available
# shellcheck disable=SC1091
source "$TMPDIR/scenario.sh"

# Collect declaration indices for this command name
INDICES=()
for ((i = 0; i < MOCK_COUNT; i++)); do
    eval "cmd=\$MOCK_${i}_COMMAND"
    if [[ "$cmd" == "$COMMAND" ]]; then
        INDICES+=("$i")
    fi
done

DECL_COUNT=${#INDICES[@]}

# Build the generated script
SCRIPT="$TMPDIR/bin/$COMMAND"

{
    cat <<'HEADER'
#!/usr/bin/env bash
set -euo pipefail
HEADER

    # Embed declaration count
    echo "DECL_COUNT=$DECL_COUNT"
    echo ""

    # Embed each declaration's variables
    for ((d = 0; d < DECL_COUNT; d++)); do
        src_idx="${INDICES[$d]}"

        eval "decl_input=\${MOCK_${src_idx}_INPUT:-}"
        eval "decl_stdout=\${MOCK_${src_idx}_STDOUT:-}"
        eval "decl_stderr=\${MOCK_${src_idx}_STDERR:-}"
        eval "decl_exit=\${MOCK_${src_idx}_EXIT_CODE:-0}"
        eval "decl_args_count=\${MOCK_${src_idx}_EXPECTED_ARGS_COUNT:-0}"

        echo "DECL_${d}_INPUT='${decl_input}'"
        echo "DECL_${d}_STDOUT='${decl_stdout}'"
        echo "DECL_${d}_STDERR='${decl_stderr}'"
        echo "DECL_${d}_EXIT_CODE=${decl_exit}"
        echo "DECL_${d}_EXPECTED_ARGS_COUNT=${decl_args_count}"

        for ((j = 0; j < decl_args_count; j++)); do
            eval "arg_val=\${MOCK_${src_idx}_EXPECTED_ARG_${j}:-}"
            # Single-quote to preserve literal ${QED_FILE} references
            echo "DECL_${d}_EXPECTED_ARG_${j}='${arg_val}'"
        done
        echo ""
    done

    # Embed the dispatch logic
    cat <<'DISPATCH'
# ── Call counter ─────────────────────────────────────────────────────
STATE_FILE="$MOCK_STATE_DIR/COMMAND_NAME.count"
if [[ -f "$STATE_FILE" ]]; then
    COUNT=$(cat "$STATE_FILE")
else
    COUNT=0
fi
COUNT=$((COUNT + 1))
printf '%d' "$COUNT" > "$STATE_FILE.tmp"
mv "$STATE_FILE.tmp" "$STATE_FILE"

CALL_INDEX=$((COUNT - 1))

if [[ $CALL_INDEX -ge $DECL_COUNT ]]; then
    echo "MOCK ERROR [COMMAND_NAME]: called $COUNT times but only $DECL_COUNT declaration(s)" >&2
    exit 127
fi

# ── Resolve current declaration ──────────────────────────────────────
eval "EXPECTED_INPUT=\$DECL_${CALL_INDEX}_INPUT"
eval "EXPECTED_STDOUT=\$DECL_${CALL_INDEX}_STDOUT"
eval "EXPECTED_STDERR=\$DECL_${CALL_INDEX}_STDERR"
eval "EXPECTED_EXIT=\$DECL_${CALL_INDEX}_EXIT_CODE"
eval "EXPECTED_ARGS_CT=\$DECL_${CALL_INDEX}_EXPECTED_ARGS_COUNT"

# ── Input validation ─────────────────────────────────────────────────
if [[ -n "$EXPECTED_INPUT" ]]; then
    ACTUAL_INPUT_FILE=$(mktemp)
    if [[ -n "${QED_FILE:-}" ]]; then
        cp "$QED_FILE" "$ACTUAL_INPUT_FILE"
    else
        cat > "$ACTUAL_INPUT_FILE"
    fi

    if ! diff "$EXPECTED_INPUT" "$ACTUAL_INPUT_FILE" > /dev/null 2>&1; then
        echo "MOCK ERROR [COMMAND_NAME] call $COUNT: input mismatch" >&2
        echo "--- expected ($EXPECTED_INPUT)" >&2
        echo "+++ actual" >&2
        diff "$EXPECTED_INPUT" "$ACTUAL_INPUT_FILE" >&2 || true
        rm -f "$ACTUAL_INPUT_FILE"
        exit 1
    fi
    rm -f "$ACTUAL_INPUT_FILE"
else
    # Drain stdin so the writing process does not get SIGPIPE
    cat > /dev/null 2>/dev/null || true
fi

# ── Argument validation ──────────────────────────────────────────────
ACTUAL_ARGS=("$@")
if [[ $EXPECTED_ARGS_CT -gt 0 ]]; then
    if [[ ${#ACTUAL_ARGS[@]} -ne $EXPECTED_ARGS_CT ]]; then
        echo "MOCK ERROR [COMMAND_NAME] call $COUNT: expected $EXPECTED_ARGS_CT args, got ${#ACTUAL_ARGS[@]}" >&2
        echo "  actual args: ${ACTUAL_ARGS[*]:-}" >&2
        exit 1
    fi
    for ((j = 0; j < EXPECTED_ARGS_CT; j++)); do
        eval "EXPECTED_ARG=\$DECL_${CALL_INDEX}_EXPECTED_ARG_${j}"
        # Expand ${QED_FILE} in the mock's own environment
        EXPECTED_EXPANDED=$(eval printf '%s' "\"$EXPECTED_ARG\"")
        if [[ "${ACTUAL_ARGS[$j]}" != "$EXPECTED_EXPANDED" ]]; then
            echo "MOCK ERROR [COMMAND_NAME] call $COUNT: arg $((j+1)) mismatch" >&2
            echo "  expected: $EXPECTED_EXPANDED" >&2
            echo "  actual:   ${ACTUAL_ARGS[$j]}" >&2
            exit 1
        fi
    done
fi

# ── Produce output ───────────────────────────────────────────────────
if [[ -n "$EXPECTED_STDERR" ]]; then
    cat "$EXPECTED_STDERR" >&2
fi
if [[ -n "$EXPECTED_STDOUT" ]]; then
    cat "$EXPECTED_STDOUT"
fi
exit "$EXPECTED_EXIT"
DISPATCH
} > "$SCRIPT"

# Replace COMMAND_NAME placeholder with actual command name
sed "s/COMMAND_NAME/${COMMAND}/g" "$SCRIPT" > "$SCRIPT.tmp"
mv "$SCRIPT.tmp" "$SCRIPT"

chmod +x "$SCRIPT"
