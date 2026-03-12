#!/usr/bin/env bash
# Runs cloud connection smoke test.
# Required env var: DUNNO_CLOUD_URL

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

if [[ -z "${DUNNO_CLOUD_URL:-}" ]]; then
  echo "SKIP: DUNNO_CLOUD_URL is not set."
  exit 0
fi

bash "$SCRIPT_DIR/test_cloud_connection.sh"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   CLOUD SUITE COMPLETED                  ║"
echo "╚══════════════════════════════════════════╝"
