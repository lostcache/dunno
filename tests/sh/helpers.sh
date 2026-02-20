#!/usr/bin/env bash
# Shared test helpers for local persistence tests.
# Source this file; do not execute directly.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DB_DIR="${PROJECT_ROOT}/db_data/_test_persistence"
CONFIG_FILE="${HOME}/.config/dunno/config.toml"
CONFIG_BAK="${CONFIG_FILE}.test-bak"

_PASS=0
_FAIL=0
_TOTAL=0
_TEST_NAME=""

# Last captured output and exit code from run_cmd.
OUT=""
RC=0

# ── Build ──────────────────────────────────────────────────────────

build_binary() {
    echo "Building..."
    cargo build --quiet --manifest-path "$PROJECT_ROOT/Cargo.toml" 2>&1

    _BIN_WRAPPER="$(mktemp "${TMPDIR:-/tmp}/lazydev-test-bin.XXXXXX")"
    cat > "$_BIN_WRAPPER" <<WRAPPER
#!/usr/bin/env bash
exec cargo run --quiet --manifest-path "$PROJECT_ROOT/Cargo.toml" -- "\$@"
WRAPPER
    chmod +x "$_BIN_WRAPPER"
    BIN="$_BIN_WRAPPER"
    export BIN
}

# ── Safe command runner ────────────────────────────────────────────
# Captures stdout+stderr into $OUT and exit code into $RC.
# Usage: run_cmd "$BIN" project list
#        run_cmd env DUNNO_BACKEND=local "$BIN" project list

run_cmd() {
    OUT="$("$@" 2>&1)" && RC=0 || RC=$?
}

# ── Config-file management ─────────────────────────────────────────

backup_config() {
    if [[ -f "$CONFIG_FILE" ]]; then
        cp "$CONFIG_FILE" "$CONFIG_BAK"
    fi
}

restore_config() {
    if [[ -f "$CONFIG_BAK" ]]; then
        mv "$CONFIG_BAK" "$CONFIG_FILE"
    else
        rm -f "$CONFIG_FILE"
    fi
}

write_config() {
    mkdir -p "$(dirname "$CONFIG_FILE")"
    printf '%s\n' "$1" > "$CONFIG_FILE"
}

remove_config() {
    rm -f "$CONFIG_FILE"
}

# ── Test-DB management ─────────────────────────────────────────────

setup_test_db_dir() {
    rm -rf "$TEST_DB_DIR"
    mkdir -p "$TEST_DB_DIR"
}

cleanup_test_db_dir() {
    rm -rf "$TEST_DB_DIR"
}

# ── Assertions ─────────────────────────────────────────────────────

assert_contains() {
    local label="$1"
    local haystack="$2"
    local needle="$3"
    _TOTAL=$((_TOTAL + 1))
    if printf '%s' "$haystack" | grep -qF "$needle"; then
        echo "  PASS  $label"
        _PASS=$((_PASS + 1))
    else
        echo "  FAIL  $label"
        echo "        expected to find: $needle"
        echo "        in: ${haystack:0:300}"
        _FAIL=$((_FAIL + 1))
    fi
}

assert_not_contains() {
    local label="$1"
    local haystack="$2"
    local needle="$3"
    _TOTAL=$((_TOTAL + 1))
    if printf '%s' "$haystack" | grep -qF "$needle"; then
        echo "  FAIL  $label"
        echo "        expected NOT to find: $needle"
        _FAIL=$((_FAIL + 1))
    else
        echo "  PASS  $label"
        _PASS=$((_PASS + 1))
    fi
}

assert_exit_ok() {
    local label="$1"
    local code="$2"
    _TOTAL=$((_TOTAL + 1))
    if [[ "$code" -eq 0 ]]; then
        echo "  PASS  $label"
        _PASS=$((_PASS + 1))
    else
        echo "  FAIL  $label (exit code $code)"
        _FAIL=$((_FAIL + 1))
    fi
}

assert_exit_nonzero() {
    local label="$1"
    local code="$2"
    _TOTAL=$((_TOTAL + 1))
    if [[ "$code" -ne 0 ]]; then
        echo "  PASS  $label"
        _PASS=$((_PASS + 1))
    else
        echo "  FAIL  $label (expected nonzero exit, got 0)"
        _FAIL=$((_FAIL + 1))
    fi
}

# ── JSON helpers (no jq dependency) ────────────────────────────────

# Extract a string value for a key from flat JSON.
# Usage: json_str "$json" "id"
json_str() {
    printf '%s' "$1" | grep -o "\"$2\":\"[^\"]*\"" | head -1 | sed "s/\"$2\":\"//;s/\"$//"
}

# ── Reporting ──────────────────────────────────────────────────────

print_header() {
    _TEST_NAME="$1"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  $_TEST_NAME"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

print_summary() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if [[ $_FAIL -eq 0 ]]; then
        echo "  ALL PASSED  $_PASS/$_TOTAL"
    else
        echo "  FAILED  $_PASS passed, $_FAIL failed (of $_TOTAL)"
    fi
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    return $_FAIL
}
