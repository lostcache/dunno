#!/usr/bin/env bash
# Test C: Local persistence using the --backend CLI flag.
# The --backend flag controls backend selection; the local path is
# supplied via DUNNO_LOCAL_PATH env var (CLI has no --path flag).
# No config file is present.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config
remove_config

DB="${TEST_DB_DIR}/cli-test.db"

print_header "Test C: CLI Flags (--backend local)"

# ── config show ────────────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is local"                   "$OUT" '"backend":"local"'
assert_contains   "path reflects env var"              "$OUT" 'cli-test.db'

# ── 1. add project ─────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local project add "CLIProject" "Created via --backend flag"
assert_exit_ok    "project add exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" '"name":"CLIProject"'
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. add module ──────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local module add --project-ids "$PROJECT_ID" "CLIModule" "Auth module"
assert_exit_ok    "module add exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. add task ────────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local task add --module-ids "$MODULE_ID" --project-ids "$PROJECT_ID" "CLITask" "Login flow"
assert_exit_ok    "task add exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local task update "$TASK_ID" --status finished
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is finished"            "$OUT" '"status":"finished"'

# ── 8-9. todos ────────────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local todo add --project-ids "$PROJECT_ID" "Deploy to staging"
assert_exit_ok    "todo add exits 0"                "$RC"

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local todo ls --project-id "$PROJECT_ID"
assert_exit_ok    "todo ls exits 0"                  "$RC"
assert_contains   "todo ls has item"                 "$OUT" 'Deploy to staging'

# ── 10-11. knowledge + ctx ────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local add --field category --value rust --field type --value mistake --field content --value "CLI linked mistake" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local ctx --task-id "$TASK_ID"
assert_exit_ok    "ctx exits 0"                    "$RC"
assert_contains   "ctx has linked mistake"         "$OUT" 'CLI linked mistake'

# ── 12. structured error ──────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local ctx --task-id "task:nonexistent"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'

# ── persistence ────────────────────────────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local project ls
assert_exit_ok    "persistence: project ls exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" 'CLIProject'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
