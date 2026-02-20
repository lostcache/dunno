#!/usr/bin/env bash
# Run all local-persistence shell tests sequentially.
#
# Usage:
#   ./tests/sh/run_all.sh            # run all tests
#   ./tests/sh/run_all.sh env cli    # run only named suites
#
# Available suites: env, config, cli, precedence, cross

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

declare -A SUITES=(
    [env]="test_local_env_vars.sh"
    [config]="test_local_config_file.sh"
    [cli]="test_local_cli_flags.sh"
    [precedence]="test_local_precedence.sh"
    [cross]="test_local_cross_method.sh"
)

ORDER=(env config cli precedence cross)

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
echo "║   Local Persistence Test Suite           ║"
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
    echo "║   ALL $suites_run SUITES PASSED                    ║"
else
    printf "║   %d/%d SUITES PASSED, %d FAILED            ║\n" \
        "$total_pass" "$suites_run" "$total_fail"
    printf "║   Failed: %-31s║\n" "${failed_suites[*]}"
fi
echo "╚══════════════════════════════════════════╝"

exit $total_fail
