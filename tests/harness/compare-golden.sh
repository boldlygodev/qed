#!/usr/bin/env bash
# compare-golden.sh — golden comparison logic
# Args: $1=actual file, $2=golden reference, $3=scenario ID, $4=channel name
# @spec TINFRA-050, TINFRA-051, TINFRA-052, TINFRA-053

set -euo pipefail

ACTUAL="$1"
GOLDEN_REF="$2"
SCENARIO_ID="$3"
CHANNEL="$4"

GOLDEN_DIR="$(dirname "$GOLDEN_REF")"
GOLDEN_BASE="$(basename "$GOLDEN_REF")"

compare_single() {
    local actual="$1"
    local golden="$2"
    local scenario_id="$3"
    local channel="$4"
    local ext="${golden##*.}"

    case "$ext" in
        pattern)
            local actual_content
            actual_content=$(cat "$actual")
            local golden_pattern
            golden_pattern=$(cat "$golden")
            # Resolve literal \n sequences to actual newlines for multiline matching
            local nl=$'\n'
            local resolved="${golden_pattern//\\n/${nl}}"
            local re="^(${resolved})$"
            if ! [[ "$actual_content" =~ $re ]]; then
                echo "FAIL [$scenario_id]: $channel does not match pattern" >&2
                echo "--- pattern ($(basename "$golden"))" >&2
                cat "$golden" >&2
                echo "+++ actual" >&2
                cat "$actual" >&2
                exit 1
            fi
            ;;
        *)
            # All other extensions (.txt, .go, .yaml, .md, .toml, etc.)
            # are compared as exact text diffs.
            if ! diff_output=$(diff "$golden" "$actual" 2>&1); then
                echo "FAIL [$scenario_id]: $channel does not match golden" >&2
                echo "--- expected ($(basename "$golden"))" >&2
                echo "+++ actual" >&2
                echo "$diff_output" >&2
                exit 1
            fi
            ;;
    esac
}

# Glob reference (ends in .*)
if [[ "$GOLDEN_BASE" == *'.*' ]]; then
    PREFIX="${GOLDEN_BASE%'.*'}"
    shopt -s nullglob
    matches=("$GOLDEN_DIR"/"$PREFIX".*)
    shopt -u nullglob

    if [[ ${#matches[@]} -eq 0 ]]; then
        echo "FAIL [$SCENARIO_ID]: $CHANNEL golden glob '$GOLDEN_BASE' matched no files in $GOLDEN_DIR" >&2
        exit 1
    fi

    for golden_file in "${matches[@]}"; do
        compare_single "$ACTUAL" "$golden_file" "$SCENARIO_ID" "$CHANNEL"
    done
    exit 0
fi

# Direct file (not a glob)
compare_single "$ACTUAL" "$GOLDEN_REF" "$SCENARIO_ID" "$CHANNEL"
