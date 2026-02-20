#!/usr/bin/env bash
# Test H: Cloud backend selected via --backend cloud CLI flag.
# Credentials are supplied via env vars; no config file is present.
#
# Required env var: DUNNO_CLOUD_URL
# Namespace/database/credentials are hardcoded for the dunno cloud setup.
# Skips gracefully if DUNNO_CLOUD_URL is not set.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

require_cloud_env
build_binary
backup_config
remove_config

PREFIX="CliCloud_${CLOUD_TS}"

cloud_run() {
    run_cmd env \
        DUNNO_CLOUD_URL="$DUNNO_CLOUD_URL" \
        DUNNO_CLOUD_NS=dunno \
        DUNNO_CLOUD_DB=dunno \
        DUNNO_CLOUD_USER=dunno \
        DUNNO_CLOUD_PASS=dunnodev \
        DUNNO_CLOUD_AUTH_TYPE=namespace \
        "$BIN" --backend cloud "$@"
}

print_header "Test H: Cloud Backend (--backend cloud)"

# ── config show ────────────────────────────────────────────────────
cloud_run config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is cloud"                   "$OUT" '"backend":"cloud"'
assert_contains   "namespace is dunno"                 "$OUT" '"namespace":"dunno"'

# ── 1. create project ─────────────────────────────────────────────
cloud_run project create "${PREFIX}_Project" "Cloud project via CLI flag"
assert_exit_ok    "project create exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" "\"name\":\"${PREFIX}_Project\""
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. create module ──────────────────────────────────────────────
cloud_run module create "$PROJECT_ID" "${PREFIX}_Module" "Auth module"
assert_exit_ok    "module create exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. create task ────────────────────────────────────────────────
cloud_run task create "$MODULE_ID" "${PREFIX}_Task" "Login flow"
assert_exit_ok    "task create exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
cloud_run task update "$TASK_ID" --status finished
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is finished"            "$OUT" '"status":"finished"'

# ── 5-7. task updates ─────────────────────────────────────────────
cloud_run task append-update "$TASK_ID" "CLI cloud update note"
assert_exit_ok    "append-update exits 0"              "$RC"
UPDATE_ID=$(json_str "$OUT" "id")

cloud_run task update-entry "$UPDATE_ID" "CLI cloud update note (edited)"
assert_exit_ok    "update-entry exits 0"               "$RC"
assert_contains   "content is edited"                  "$OUT" '(edited)'

cloud_run task list-updates "$TASK_ID"
assert_exit_ok    "list-updates exits 0"               "$RC"
assert_contains   "list has edited update"             "$OUT" '(edited)'

# ── 8-9. todos ────────────────────────────────────────────────────
cloud_run todo create "$PROJECT_ID" "${PREFIX} Deploy to staging"
assert_exit_ok    "todo create exits 0"                "$RC"
assert_contains   "todo status is pending"             "$OUT" '"status":"pending"'

cloud_run todo list "$PROJECT_ID"
assert_exit_ok    "todo list exits 0"                  "$RC"
assert_contains   "todo list has item"                 "$OUT" 'Deploy to staging'

# ── 10-11. knowledge + context ────────────────────────────────────
cloud_run add --category rust --type mistake -C "${PREFIX} CLI linked cloud mistake" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"

cloud_run context --task-id "$TASK_ID"
assert_exit_ok    "context exits 0"                    "$RC"
assert_contains   "context has linked mistake"         "$OUT" 'CLI linked cloud mistake'

# ── 12. structured error ──────────────────────────────────────────
cloud_run context --task-id "task:nonexistent_${CLOUD_TS}"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'

# ── persistence ────────────────────────────────────────────────────
cloud_run project list
assert_exit_ok    "persistence: project list exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" "${PREFIX}_Project"

# ── teardown ───────────────────────────────────────────────────────
restore_config
print_summary
