#!/usr/bin/env bash
# run-scenario.sh — main bash runner, invoked once per Trial
# Args: $1=temp directory path
# @spec TINFRA-030, TINFRA-031, TINFRA-033, TINFRA-034, TINFRA-047

set -euo pipefail

TMPDIR="$1"
HARNESS_DIR="$(cd "$(dirname "$0")" && pwd)"

# Source scenario variables
# shellcheck disable=SC1091
source "$TMPDIR/scenario.sh"

# Copy input to temp directory
cp "$INPUT_SRC" "$TMPDIR/input"

# Set up invocation variables
INPUT="$TMPDIR/input"
STDOUT="$TMPDIR/stdout"
STDERR="$TMPDIR/stderr"
OUTPUT="$TMPDIR/output"

# Ensure output files exist for comparison
touch "$STDOUT" "$STDERR" "$OUTPUT"

# Environment setup
export MOCK_STATE_DIR="$TMPDIR/mock-state"
export PATH="$TMPDIR/bin:$PATH"

# Generate mock scripts
for ((i = 0; i < MOCK_COUNT; i++)); do
    eval "cmd=\$MOCK_${i}_COMMAND"
    if [[ ! -f "$TMPDIR/bin/$cmd" ]]; then
        "$HARNESS_DIR/generate-mock.sh" "$TMPDIR" "$cmd"
    fi
done

# Execute the invocation in a subshell so `exit` in the invocation
# does not kill this script (some invocations use `exit $CODE`).
set +e
(eval "$INVOCATION")
ACTUAL_EXIT=$?
set -e

# Exit code assertion
if [[ "$ACTUAL_EXIT" -ne "$EXPECTED_EXIT_CODE" ]]; then
    echo "FAIL [$SCENARIO_ID]: exit code $ACTUAL_EXIT, expected $EXPECTED_EXIT_CODE — $SCENARIO_DESC" >&2
    exit 1
fi

# Golden comparisons
"$HARNESS_DIR/compare-golden.sh" "$STDOUT" "$STDOUT_GOLDEN" "$SCENARIO_ID" "stdout"
"$HARNESS_DIR/compare-golden.sh" "$STDERR" "$STDERR_GOLDEN" "$SCENARIO_ID" "stderr"
"$HARNESS_DIR/compare-golden.sh" "$OUTPUT" "$OUTPUT_GOLDEN" "$SCENARIO_ID" "output"

# Verify all declared mock calls were consumed (TINFRA-047)
# Uses per-command files instead of associative arrays for bash 3.2 compatibility.
if [[ $MOCK_COUNT -gt 0 ]]; then
    _expected_dir="$TMPDIR/mock-expected"
    mkdir -p "$_expected_dir"
    for ((i = 0; i < MOCK_COUNT; i++)); do
        eval "cmd=\$MOCK_${i}_COMMAND"
        _n=0
        [[ -f "$_expected_dir/$cmd" ]] && _n=$(cat "$_expected_dir/$cmd")
        echo $(( _n + 1 )) > "$_expected_dir/$cmd"
    done
    for _f in "$_expected_dir"/*; do
        cmd="$(basename "$_f")"
        expected=$(cat "$_f")
        actual=0
        [[ -f "$MOCK_STATE_DIR/$cmd.count" ]] && actual=$(cat "$MOCK_STATE_DIR/$cmd.count")
        if [[ $actual -lt $expected ]]; then
            echo "FAIL [$SCENARIO_ID]: mock '$cmd' called $actual time(s), expected $expected — $SCENARIO_DESC" >&2
            exit 1
        fi
    done
fi
