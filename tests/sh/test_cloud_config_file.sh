#!/usr/bin/env bash
# Test G: Cloud backend configured entirely via ~/.config/dunno/config.toml.
# No environment variables or CLI flags needed — everything is in the config.
#
# Prerequisite: a valid config file at ~/.config/dunno/config.toml with
# backend = "cloud" and the [cloud] section filled out.
# Skips gracefully if the config file is missing or backend != cloud.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

CONFIG_FILE="${HOME}/.config/dunno/config.toml"
if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "SKIP: $CONFIG_FILE does not exist."
    exit 0
fi
if ! grep -q 'backend.*=.*"cloud"' "$CONFIG_FILE"; then
    echo "SKIP: $CONFIG_FILE does not have backend = \"cloud\"."
    exit 0
fi

build_binary

PREFIX="CfgCloud_${CLOUD_TS}"

print_header "Test G: Cloud Backend (config file)"

# ── config show ────────────────────────────────────────────────────
run_cmd "$BIN" config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is cloud"                   "$OUT" '"backend":"cloud"'
assert_contains   "namespace is dunno"                 "$OUT" '"namespace":"dunno"'
assert_contains   "username is dunno"                  "$OUT" '"username":"dunno"'
assert_contains   "password is redacted"               "$OUT" '***redacted***'

# ── 1. add project ─────────────────────────────────────────────
run_cmd "$BIN" project add "${PREFIX}_Project" "Cloud project via config file"
assert_exit_ok    "project add exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" "\"name\":\"${PREFIX}_Project\""
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. add module ──────────────────────────────────────────────
run_cmd "$BIN" module add --project-ids "$PROJECT_ID" "${PREFIX}_Module" "Auth module"
assert_exit_ok    "module add exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. add task ────────────────────────────────────────────────
run_cmd "$BIN" task add --module-ids "$MODULE_ID" --project-ids "$PROJECT_ID" "${PREFIX}_Task" "Login flow"
assert_exit_ok    "task add exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd "$BIN" task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 8. add todo ────────────────────────────────────────────────
run_cmd "$BIN" todo add --project-ids "$PROJECT_ID" "${PREFIX} Write tests"
assert_exit_ok    "todo add exits 0"                "$RC"
assert_contains   "todo status is pending"             "$OUT" '"status":"pending"'

# ── 9. ls todos ─────────────────────────────────────────────────
run_cmd "$BIN" todo ls --project-id "$PROJECT_ID"
assert_exit_ok    "todo ls exits 0"                  "$RC"
assert_contains   "todo ls has item"                 "$OUT" 'Write tests'

# ── 10. add knowledge ─────────────────────────────────────────────
run_cmd "$BIN" add --field category --value rust --field type --value mistake --field content --value "${PREFIX} Config file cloud mistake" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve ctx ──────────────────────────────────────────
run_cmd "$BIN" ctx --task-id "$TASK_ID"
assert_exit_ok    "ctx exits 0"                    "$RC"
assert_contains   "ctx contains mistake"           "$OUT" 'Config file cloud mistake'

# ── 12. structured error ──────────────────────────────────────────
run_cmd "$BIN" ctx --task-id "task:nonexistent_${CLOUD_TS}"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'

# ── persistence check ────────────────────────────────────────────
run_cmd "$BIN" project ls
assert_exit_ok    "persistence: project ls exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" "${PREFIX}_Project"

# ── done ──────────────────────────────────────────────────────────
print_summary
