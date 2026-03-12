#!/usr/bin/env bash
# Test B: Local persistence configured entirely via ~/.config/dunno/config.toml.
# No environment variables or CLI flags are used for backend/path selection.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config

DB="${TEST_DB_DIR}/config-test.db"

write_config "$(cat <<TOML
backend = "local"

[local]
path = "$DB"
TOML
)"

print_header "Test B: Config File"

# ── config show ────────────────────────────────────────────────────
run_cmd "$BIN" config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is local"                   "$OUT" '"backend":"local"'
assert_contains   "local path from config file"        "$OUT" 'config-test.db'

# ── 1. add project ─────────────────────────────────────────────
run_cmd "$BIN" project add "ConfigProject" "Created via config file"
assert_exit_ok    "project add exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" '"name":"ConfigProject"'
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. add module ──────────────────────────────────────────────
run_cmd "$BIN" module add --project-ids "$PROJECT_ID" "ConfigModule" "Auth module"
assert_exit_ok    "module add exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. add task ────────────────────────────────────────────────
run_cmd "$BIN" task add --module-ids "$MODULE_ID" --project-ids "$PROJECT_ID" "ConfigTask" "Login flow"
assert_exit_ok    "task add exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd "$BIN" task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 8. add todo ────────────────────────────────────────────────
run_cmd "$BIN" todo add --project-ids "$PROJECT_ID" "Write tests"
assert_exit_ok    "todo add exits 0"                "$RC"

# ── 9. ls todos ─────────────────────────────────────────────────
run_cmd "$BIN" todo ls --project-id "$PROJECT_ID"
assert_exit_ok    "todo ls exits 0"                  "$RC"
assert_contains   "todo ls has item"                 "$OUT" 'Write tests'

# ── 10. add knowledge ─────────────────────────────────────────────
run_cmd "$BIN" add --field category --value rust --field type --value mistake --field content --value "Config file mistake" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve ctx ──────────────────────────────────────────
run_cmd "$BIN" ctx --task-id "$TASK_ID"
assert_exit_ok    "ctx exits 0"                    "$RC"
assert_contains   "ctx contains mistake"           "$OUT" 'Config file mistake'

# ── 12. structured error ──────────────────────────────────────────
run_cmd "$BIN" ctx --task-id "task:nonexistent"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'

# ── persistence check ─────────────────────────────────────────────
run_cmd "$BIN" project ls
assert_exit_ok    "persistence: project ls exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" 'ConfigProject'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
