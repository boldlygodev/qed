#!/usr/bin/env bash
# run-scenario.sh — main bash runner, invoked once per Trial
# Args: $1=temp directory path

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

# Execute the invocation
set +e
eval "$INVOCATION"
ACTUAL_EXIT=$?
set -e

# Exit code assertion
if [[ "$ACTUAL_EXIT" -ne "$EXPECTED_EXIT_CODE" ]]; then
    echo "FAIL [$SCENARIO_ID]: exit code $ACTUAL_EXIT, expected $EXPECTED_EXIT_CODE" >&2
    exit 1
fi

# Golden comparisons
"$HARNESS_DIR/compare-golden.sh" "$STDOUT" "$STDOUT_GOLDEN" "$SCENARIO_ID" "stdout"
"$HARNESS_DIR/compare-golden.sh" "$STDERR" "$STDERR_GOLDEN" "$SCENARIO_ID" "stderr"
"$HARNESS_DIR/compare-golden.sh" "$OUTPUT" "$OUTPUT_GOLDEN" "$SCENARIO_ID" "output"

# Mock unconsumed check placeholder (Phase 7)
