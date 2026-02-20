#!/usr/bin/env bash
# Run all cloud-backend shell tests sequentially.
#
# Usage:
#   ./tests/sh/run_cloud.sh                 # run all cloud tests
#   ./tests/sh/run_cloud.sh env cli         # run only named suites
#
# Available suites: env, config, cli
#
# Required env vars:
#   DUNNO_CLOUD_URL   DUNNO_CLOUD_NS   DUNNO_CLOUD_DB
#   DUNNO_CLOUD_USER  DUNNO_CLOUD_PASS
#
# Tests skip gracefully if credentials are not set.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

declare -A SUITES=(
    [env]="test_cloud_env_vars.sh"
    [config]="test_cloud_config_file.sh"
    [cli]="test_cloud_cli_flags.sh"
)

ORDER=(env config cli)

if [[ $# -gt 0 ]]; then
    selected=("$@")
else
    selected=("${ORDER[@]}")
fi

total_pass=0
total_fail=0
failed_suites=()

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   Cloud Backend Test Suite               ║"
echo "╚══════════════════════════════════════════╝"

for suite in "${selected[@]}"; do
    file="${SUITES[$suite]:-}"
    if [[ -z "$file" ]]; then
        echo "Unknown suite: $suite (available: ${ORDER[*]})"
        exit 2
    fi

    echo ""
    echo "▶ Running: $suite ($file)"
    if bash "$SCRIPT_DIR/$file"; then
        total_pass=$((total_pass + 1))
    else
        total_fail=$((total_fail + 1))
        failed_suites+=("$suite")
    fi
done

echo ""
echo "╔══════════════════════════════════════════╗"
suites_run=$(( total_pass + total_fail ))
if [[ $total_fail -eq 0 ]]; then
    printf "║   ALL %d SUITES PASSED                    ║\n" "$suites_run"
else
    printf "║   %d/%d SUITES PASSED, %d FAILED            ║\n" \
        "$total_pass" "$suites_run" "$total_fail"
    printf "║   Failed: %-31s║\n" "${failed_suites[*]}"
fi
echo "╚══════════════════════════════════════════╝"

exit $total_fail
