#!/usr/bin/env bash
# Test: Cloud connection and authentication smoke test.
# Verifies binary can connect and auth to Cloud using config file.
# Required env vars: DUNNO_CLOUD_URL, DUNNO_CLOUD_NS, DUNNO_CLOUD_DB, DUNNO_CLOUD_USER, DUNNO_CLOUD_PASS

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

require_cloud_env
build_binary
backup_config

PREFIX="CloudSmoke_${CLOUD_TS}"

write_config "$(cat <<TOML
backend = "cloud"
url = "${DUNNO_CLOUD_URL}"
namespace = "${DUNNO_CLOUD_NS}"
database = "${DUNNO_CLOUD_DB}"
username = "${DUNNO_CLOUD_USER}"
password = "${DUNNO_CLOUD_PASS}"
auth_type = "${DUNNO_CLOUD_AUTH_TYPE:-namespace}"
TOML
)"

print_header "Test: Cloud Backend Smoke Test"

# ── 1. Config show reflects cloud settings ───────────────────────
echo "--- Testing config show ---"
run_cmd "$BIN" config show
assert_exit_ok    "config show exits 0"                    "$RC"
assert_contains   "backend is cloud"                       "$OUT" '"backend":"cloud"'

# ── 2. Create projects ────────────────────────────────────────────
echo "--- Testing project creation ---"
run_cmd "$BIN" project add "${PREFIX}_A" "Smoke test"
assert_exit_ok    "project add exits 0"                    "$RC"
assert_contains   "project created"                        "$OUT" "${PREFIX}_A"

run_cmd "$BIN" project add "${PREFIX}_B" "Smoke test"
assert_exit_ok    "second project add exits 0"             "$RC"
assert_contains   "second project created"                 "$OUT" "${PREFIX}_B"

# ── 3. Persistence check ──────────────────────────────────────────
echo "--- Testing persistence ---"
run_cmd "$BIN" project ls
assert_exit_ok    "project ls exits 0"                     "$RC"
assert_contains   "project A persisted"                    "$OUT" "${PREFIX}_A"
assert_contains   "project B persisted"                    "$OUT" "${PREFIX}_B"

# ── teardown ───────────────────────────────────────────────────────
restore_config
print_summary
