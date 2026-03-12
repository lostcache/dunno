#!/usr/bin/env bash
# Runs all configuration and persistence tests.
# These tests focus on binary-level behavior that cannot be fully captured in Rust unit tests.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

# 1. Configuration Hierarchy: defaults -> config file -> env vars -> CLI flags
bash "$SCRIPT_DIR/test_config_hierarchy.sh"

# Cloud tests require all DUNNO_CLOUD_* env vars
missing=()
[[ -z "${DUNNO_CLOUD_URL:-}" ]] && missing+=("DUNNO_CLOUD_URL")
[[ -z "${DUNNO_CLOUD_NS:-}" ]] && missing+=("DUNNO_CLOUD_NS")
[[ -z "${DUNNO_CLOUD_DB:-}" ]] && missing+=("DUNNO_CLOUD_DB")
[[ -z "${DUNNO_CLOUD_USER:-}" ]] && missing+=("DUNNO_CLOUD_USER")
[[ -z "${DUNNO_CLOUD_PASS:-}" ]] && missing+=("DUNNO_CLOUD_PASS")

if [[ ${#missing[@]} -eq 0 ]]; then
    bash "$SCRIPT_DIR/test_cloud_connection.sh"
else
    echo ""
    echo "NOTICE: Cloud smoke tests were skipped because the following env vars are not set:"
    echo "        ${missing[*]}"
    echo "        To enable them, export all the variables above (e.g., export DUNNO_CLOUD_URL='wss://...')"
fi

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   ALL SUITES COMPLETED                   ║"
echo "╚══════════════════════════════════════════╝"
