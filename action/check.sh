#!/bin/bash
#
# Spikes Feedback Gate — CI Gate Logic
#
# Counts blocking feedback and fails if threshold exceeded.
# Punk/zine energy: this script tells it like it is.
#
# Usage:
#   ./check.sh [threshold] [ignore-paths] [require-resolution]
#
# Environment:
#   SPIKES_BIN  — Path to spikes binary (default: ./spikes)
#

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

THRESHOLD="${1:-0}"
IGNORE_PATHS="${2:-}"
REQUIRE_RESOLUTION="${3:-false}"
SPIKES_BIN="${SPIKES_BIN:-./spikes}"

# ============================================================================
# Functions
# ============================================================================

# Punk/zine warning: something's off but we'll let it slide
warn() {
    echo "::warning::🔥 $1"
    echo "[SPIKES] $1" >&2
}

# Punk/zine error: the vibes are WRONG
error() {
    echo "::error::💀 $1"
    echo "[SPIKES] $1" >&2
}

# Info message
info() {
    echo "[SPIKES] $1"
}

# Set GitHub Actions output (handles missing GITHUB_OUTPUT gracefully)
set_output() {
    local name="$1"
    local value="$2"

    if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
        echo "$name=$value" >> "$GITHUB_OUTPUT"
    fi
}

# Check if jq is available
check_jq() {
    if ! command -v jq &> /dev/null; then
        error "jq is not installed. The vibes are off — can't parse JSON without it."
        exit 1
    fi
}

# Check if spikes binary exists
check_spikes() {
    if [[ ! -x "$SPIKES_BIN" ]]; then
        error "Spikes binary not found at $SPIKES_BIN. Did the download fail?"
        exit 1
    fi
}

# Check if .spikes directory exists
check_spikes_dir() {
    if [[ ! -d ".spikes" ]]; then
        warn "No .spikes/ directory found. Clean slate — passing by default."
        set_output "blocking_count" "0"
        set_output "status" "passed"
        exit 0
    fi
}

# Check if feedback file exists and has content
check_feedback_file() {
    local feedback_file=".spikes/feedback.jsonl"

    if [[ ! -f "$feedback_file" ]]; then
        warn "No feedback.jsonl found. Nothing to judge."
        set_output "blocking_count" "0"
        set_output "status" "passed"
        exit 0
    fi

    if [[ ! -s "$feedback_file" ]]; then
        warn "feedback.jsonl is empty. No feedback to analyze."
        set_output "blocking_count" "0"
        set_output "status" "passed"
        exit 0
    fi
}

# Export spikes as JSON
export_spikes() {
    "$SPIKES_BIN" export --format json 2>/dev/null
}

# Check if a page matches any ignore pattern
# Args: page, ignore_patterns (newline-separated)
should_ignore() {
    local page="$1"
    local patterns="$2"

    if [[ -z "$patterns" ]]; then
        return 1  # No patterns, don't ignore
    fi

    # Check each pattern
    while IFS= read -r pattern; do
        [[ -z "$pattern" ]] && continue

        # Use glob matching via case statement
        # shellcheck disable=SC2254
        case "$page" in
            $pattern)
                return 0  # Matched, should ignore
                ;;
        esac
    done <<< "$patterns"

    return 1  # No match, don't ignore
}

# Count blocking spikes
# Blocking = unresolved + negative rating (meh or no)
# When require-resolution=true: ALL unresolved spikes are blocking
count_blocking() {
    local json_data="$1"
    local ignore_patterns="$2"
    local require_res="$3"
    local count=0
    local blocking_list=""

    # Parse spikes array
    local num_spikes
    num_spikes=$(echo "$json_data" | jq 'length' 2>/dev/null || echo "0")

    if [[ "$num_spikes" -eq 0 ]]; then
        echo "0"
        return
    fi

    # Iterate through spikes
    for ((i=0; i<num_spikes; i++)); do
        local spike
        spike=$(echo "$json_data" | jq ".[$i]" 2>/dev/null)

        local page
        page=$(echo "$spike" | jq -r '.page // ""' 2>/dev/null)

        # Check ignore patterns
        if should_ignore "$page" "$ignore_patterns"; then
            echo "[SPIKES] Ignoring spike on page: $page" >&2
            continue
        fi

        local resolved
        resolved=$(echo "$spike" | jq -r '.resolved // false' 2>/dev/null)

        local rating
        rating=$(echo "$spike" | jq -r '.rating // ""' 2>/dev/null)

        local id
        id=$(echo "$spike" | jq -r '.id // "unknown"' 2>/dev/null)

        local comments
        comments=$(echo "$spike" | jq -r '.comments // ""' 2>/dev/null)

        # Check if unresolved
        if [[ "$resolved" != "true" ]]; then
            local is_blocking=false

            if [[ "$require_res" == "true" ]]; then
                # require-resolution: ANY unresolved spike is blocking
                is_blocking=true
            else
                # Default: only negative ratings (meh or no) are blocking
                if [[ "$rating" == "meh" || "$rating" == "no" ]]; then
                    is_blocking=true
                fi
            fi

            if [[ "$is_blocking" == "true" ]]; then
                ((count++))
                blocking_list+="  - [$id] $page (rating: $rating) \"$comments\"\n"
            fi
        fi
    done

    # Output count
    echo "$count"

    # Store blocking list for later
    BLOCKING_LIST="$blocking_list"
}

# ============================================================================
# Main
# ============================================================================

info "🔥 SPIKES FEEDBACK GATE — Checking the vibes..."
info "Threshold: $THRESHOLD"
info "Require resolution: $REQUIRE_RESOLUTION"

if [[ -n "$IGNORE_PATHS" ]]; then
    info "Ignore paths: $(echo "$IGNORE_PATHS" | tr '\n' ' ')"
fi

# Pre-flight checks
check_jq
check_spikes
check_spikes_dir
check_feedback_file

# Export spikes to JSON
info "Loading feedback data..."
JSON_DATA=$(export_spikes)

if [[ -z "$JSON_DATA" ]] || [[ "$JSON_DATA" == "[]" ]] || [[ "$JSON_DATA" == "null" ]]; then
    warn "No spikes data found. Clean slate."
    set_output "blocking_count" "0"
    set_output "status" "passed"
    exit 0
fi

# Count blocking spikes
BLOCKING_LIST=""
BLOCKING_COUNT=$(count_blocking "$JSON_DATA" "$IGNORE_PATHS" "$REQUIRE_RESOLUTION")

info "Found $BLOCKING_COUNT blocking spike(s)"

# Output to GitHub Actions
set_output "blocking_count" "$BLOCKING_COUNT"

# Compare to threshold
if [[ "$BLOCKING_COUNT" -gt "$THRESHOLD" ]]; then
    set_output "status" "failed"

    error "THE VIBES ARE OFF! $BLOCKING_COUNT blocking spike(s) exceed threshold of $THRESHOLD"
    echo ""
    echo "💀 BLOCKING FEEDBACK:"
    echo "─────────────────────────────────────────────────"
    echo -e "$BLOCKING_LIST"
    echo "─────────────────────────────────────────────────"
    echo ""
    echo "Fix the feedback above, or adjust your threshold."
    echo "Remember: feedback is a gift. But sometimes gifts are cursed."

    exit 1
else
    set_output "status" "passed"

    if [[ "$BLOCKING_COUNT" -eq 0 ]]; then
        info "✨ CLEAN SLATE! No blocking feedback. Ship it."
    else
        info "✅ PASSED: $BLOCKING_COUNT blocking spike(s) within threshold of $THRESHOLD"
        info "The vibes are acceptable. Proceed with confidence."
    fi

    exit 0
fi
