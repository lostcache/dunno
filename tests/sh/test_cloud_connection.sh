#!/usr/bin/env bash
# Test: Cloud connection and authentication smoke test.
# Verifies binary can connect and auth to Cloud using config/env/flags.
# Required env vars: DUNNO_CLOUD_URL, DUNNO_CLOUD_NS, DUNNO_CLOUD_DB, DUNNO_CLOUD_USER, DUNNO_CLOUD_PASS

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

require_cloud_env
build_binary
backup_config
remove_config

PREFIX="CloudSmoke_${CLOUD_TS}"

# Helper for standard cloud env vars
cloud_env() {
    env \
        DUNNO_CLOUD_URL="${DUNNO_CLOUD_URL}" \
        DUNNO_CLOUD_NS="${DUNNO_CLOUD_NS}" \
        DUNNO_CLOUD_DB="${DUNNO_CLOUD_DB}" \
        DUNNO_CLOUD_USER="${DUNNO_CLOUD_USER}" \
        DUNNO_CLOUD_PASS="${DUNNO_CLOUD_PASS}" \
        DUNNO_CLOUD_AUTH_TYPE="${DUNNO_CLOUD_AUTH_TYPE:-namespace}" \
        "$@"
}

print_header "Test: Cloud Backend Smoke Test"

# ── 1. Connection via env vars ────────────────────────────────────
echo "--- Testing connection via env vars ---"
cloud_env DUNNO_BACKEND=cloud "$BIN" config show
assert_exit_ok    "env: config show exits 0"                "$RC"
assert_contains   "env: backend is cloud"                   "$OUT" '"backend":"cloud"'

cloud_env DUNNO_BACKEND=cloud "$BIN" project add "${PREFIX}_Env" "Smoke test"
assert_exit_ok    "env: project add exits 0"                "$RC"
assert_contains   "env: project created"                    "$OUT" "${PREFIX}_Env"

# ── 2. Connection via CLI flags ───────────────────────────────────
echo "--- Testing connection via CLI flags ---"
cloud_env "$BIN" --backend cloud project add "${PREFIX}_Flag" "Smoke test"
assert_exit_ok    "flag: project add exits 0"               "$RC"
assert_contains   "flag: project created"                   "$OUT" "${PREFIX}_Flag"

# ── 3. Persistence Check ──────────────────────────────────────────
echo "--- Testing persistence check ---"
cloud_env DUNNO_BACKEND=cloud "$BIN" project ls
assert_exit_ok    "cloud persistence: project ls exits 0"   "$RC"
assert_contains   "cloud persistence: env project exists"   "$OUT" "${PREFIX}_Env"
assert_contains   "cloud persistence: flag project exists"  "$OUT" "${PREFIX}_Flag"

# ── teardown ───────────────────────────────────────────────────────
restore_config
print_summary
