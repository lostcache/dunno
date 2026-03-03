#!/usr/bin/env python3
"""
Seed script to populate the database with dummy data for visualization in Surrealist.

Graph model (current):
- Structure: project -> module -> submodule -> file; project -> task -> subtask (via has_task,
  belongs_to_project, belongs_to_module, has_subtask, belongs_to_task).
- Knowledge: mistakes, style rules, and security details are linked with `dunno add --link-to <id>`.
  The backend creates forward edges (has_mistake, has_style, has_security_detail) and reverse
  edges (belongs_to_project, belongs_to_module, belongs_to_task) automatically, so each
  knowledge node points back to the relevant project/module/task in the hierarchy.
- You can link to any structural node: project, module, submodule, task, or subtask.
"""

import subprocess
import json
import sys
import argparse
from pathlib import Path

BINARY_PATH = None


def run_cmd(cmd, check=True):
    """Run a command and return parsed JSON output."""
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error running: {' '.join(cmd)}", file=sys.stderr)
        print(f"stderr: {result.stderr}", file=sys.stderr)
        print(f"stdout: {result.stdout}", file=sys.stderr)
        if check:
            sys.exit(1)
        return None
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        print(f"Failed to parse JSON from: {result.stdout}", file=sys.stderr)
        if check:
            sys.exit(1)
        return None


def get_dunno_binary():
    """Find the dunno binary."""
    global BINARY_PATH

    if BINARY_PATH:
        return BINARY_PATH

    # Check if binary exists in target/release or target/debug
    for path in ["target/release/dunno", "target/debug/dunno"]:
        if Path(path).exists():
            BINARY_PATH = path
            return BINARY_PATH

    # Try to find in PATH
    result = subprocess.run(["which", "dunno"], capture_output=True, text=True)
    if result.returncode == 0:
        BINARY_PATH = result.stdout.strip()
        return BINARY_PATH

    print(
        "Error: dunno binary not found. Please build it first: cargo build --release",
        file=sys.stderr,
    )
    sys.exit(1)


def clear_database():
    """Clear existing data using the purge command."""
    dunno = get_dunno_binary()
    print("Clearing database via 'dunno purge'...")
    run_cmd([dunno, "purge"], check=True)

    # Optional: Still try to clean up local files if backend is local, just to be thorough?
    # No, 'reset' is sufficient for the database content.
    # If the user switches backend later, the file might remain but it won't be used.
    # The previous file deletion logic was actually risky if config pointed elsewhere.
    # Relying on the binary to handle its own storage is much safer.


def seed():
    """Seed the database with dummy data."""
    dunno = get_dunno_binary()

    print("Seeding database...")

    # 1. Create 2 projects
    print("Creating projects...")
    p1 = run_cmd(
        [
            dunno,
            "project",
            "create",
            "E-commerce API",
            "Online store backend with user auth, product catalog, and order management",
        ]
    )
    p2 = run_cmd(
        [
            dunno,
            "project",
            "create",
            "Task Manager",
            "Personal productivity app with boards, lists, and calendar view",
        ]
    )

    p1_id = p1["id"]
    p2_id = p2["id"]
    print(f"  Created project: {p1['name']} ({p1_id})")
    print(f"  Created project: {p2['name']} ({p2_id})")

    # 2. Create modules for Project 1
    print("Creating modules for E-commerce API...")
    m1_1 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p1_id,
            "auth",
            "User authentication and authorization",
        ]
    )
    m1_2 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p1_id,
            "products",
            "Product catalog and inventory management",
        ]
    )
    m1_3 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p1_id,
            "orders",
            "Order processing and payment handling",
        ]
    )

    m1_1_id = m1_1["id"]
    m1_2_id = m1_2["id"]
    m1_3_id = m1_3["id"]

    print(f"  Created module: {m1_1['name']}")
    print(f"  Created module: {m1_2['name']}")
    print(f"  Created module: {m1_3['name']}")

    # 3. Create modules for Project 2
    print("Creating modules for Task Manager...")
    m2_1 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p2_id,
            "boards",
            "Kanban board management",
        ]
    )
    m2_2 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p2_id,
            "tasks",
            "Task CRUD operations",
        ]
    )
    m2_3 = run_cmd(
        [
            dunno,
            "module",
            "create",
            "--project-ids",
            p2_id,
            "calendar",
            "Calendar view integration",
        ]
    )

    m2_1_id = m2_1["id"]
    m2_2_id = m2_2["id"]
    m2_3_id = m2_3["id"]

    print(f"  Created module: {m2_1['name']}")
    print(f"  Created module: {m2_2['name']}")
    print(f"  Created module: {m2_3['name']}")

    # 4. Create submodules for some modules
    print("Creating submodules...")
    sm1 = run_cmd(
        [
            dunno,
            "submodule",
            "create",
            "--module-ids",
            m1_1_id,
            "jwt",
            "JWT token handling",
        ]
    )
    sm2 = run_cmd(
        [
            dunno,
            "submodule",
            "create",
            "--module-ids",
            m1_1_id,
            "oauth",
            "OAuth2 provider integration",
        ]
    )
    sm3 = run_cmd(
        [
            dunno,
            "submodule",
            "create",
            "--module-ids",
            m1_2_id,
            "search",
            "Product search with filters",
        ]
    )

    sm1_id = sm1["id"]
    sm2_id = sm2["id"]
    sm3_id = sm3["id"]

    print(f"  Created submodule: {sm1['name']}")
    print(f"  Created submodule: {sm2['name']}")
    print(f"  Created submodule: {sm3['name']}")

    # 5. Create files
    print("Creating files...")
    f1 = run_cmd(
        [dunno, "file", "create", "--parent-ids", sm1_id, "jwt.rs", "src/auth/jwt.rs"]
    )
    f2 = run_cmd(
        [
            dunno,
            "file",
            "create",
            "--parent-ids",
            sm2_id,
            "oauth.rs",
            "src/auth/oauth.rs",
        ]
    )
    f3 = run_cmd(
        [dunno, "file", "create", "--parent-ids", sm1_id, "mod.rs", "src/auth/mod.rs"]
    )
    f4 = run_cmd(
        [
            dunno,
            "file",
            "create",
            "--parent-ids",
            m1_2_id,
            "product.rs",
            "src/products/product.rs",
        ]
    )
    f5 = run_cmd(
        [
            dunno,
            "file",
            "create",
            "--parent-ids",
            m1_2_id,
            "inventory.rs",
            "src/products/inventory.rs",
        ]
    )
    f6 = run_cmd(
        [
            dunno,
            "file",
            "create",
            "--parent-ids",
            sm3_id,
            "search.rs",
            "src/products/search.rs",
        ]
    )

    print(f"  Created file: {f1['name']}")
    print(f"  Created file: {f2['name']}")
    print(f"  Created file: {f3['name']}")
    print(f"  Created file: {f4['name']}")
    print(f"  Created file: {f5['name']}")
    print(f"  Created file: {f6['name']}")

    # 6. Create tasks
    print("Creating tasks...")
    t1 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m1_1_id,
            "--project-ids",
            p1_id,
            "Implement JWT refresh tokens",
            "Add refresh token rotation for better security",
        ]
    )
    t2 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m1_1_id,
            "--project-ids",
            p1_id,
            "Add OAuth2 Google login",
            "Integrate Google as an OAuth2 provider",
        ]
    )
    t3 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m1_2_id,
            "--project-ids",
            p1_id,
            "Product search with filters",
            "Implement advanced search with category and price filters",
        ]
    )
    t4 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m1_3_id,
            "--project-ids",
            p1_id,
            "Stripe payment integration",
            "Add Stripe for payment processing",
        ]
    )
    t5 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m2_1_id,
            "--project-ids",
            p2_id,
            "Drag and drop cards",
            "Implement drag and drop for kanban cards",
        ]
    )
    t6 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m2_2_id,
            "--project-ids",
            p2_id,
            "Task due dates",
            "Add due date functionality to tasks",
        ]
    )
    t7 = run_cmd(
        [
            dunno,
            "task",
            "create",
            "--module-ids",
            m2_3_id,
            "--project-ids",
            p2_id,
            "Calendar sync",
            "Sync tasks with Google Calendar",
        ]
    )

    t1_id = t1["id"]
    t2_id = t2["id"]
    t3_id = t3["id"]
    t4_id = t4["id"]
    t5_id = t5["id"]
    t6_id = t6["id"]
    t7_id = t7["id"]

    # Update some task statuses
    run_cmd([dunno, "task", "update", t1_id, "--status", "started"], check=False)
    run_cmd([dunno, "task", "update", t3_id, "--status", "finished"], check=False)
    run_cmd([dunno, "task", "update", t5_id, "--status", "started"], check=False)

    print(f"  Created task: {t1['name']}")
    print(f"  Created task: {t2['name']}")
    print(f"  Created task: {t3['name']}")
    print(f"  Created task: {t4['name']}")
    print(f"  Created task: {t5['name']}")
    print(f"  Created task: {t6['name']}")
    print(f"  Created task: {t7['name']}")

    # 7. Create subtasks
    print("Creating subtasks...")
    st1 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t1_id,
            "Add refresh token table",
            "Create database table for refresh tokens",
        ]
    )
    st2 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t1_id,
            "Implement token rotation",
            "Rotate refresh tokens on each use",
        ]
    )
    st3 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t2_id,
            "Register Google OAuth app",
            "Set up Google Cloud project",
        ]
    )
    st4 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t3_id,
            "Add Elasticsearch",
            "Set up Elasticsearch for search",
        ]
    )
    st5 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t4_id,
            "Stripe API integration",
            "Connect to Stripe API",
        ]
    )
    st6 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t5_id,
            "Frontend drag-drop",
            "Implement React DnD",
        ]
    )
    st7 = run_cmd(
        [
            dunno,
            "subtask",
            "create",
            "--task-ids",
            t6_id,
            "Date picker component",
            "Add date picker UI",
        ]
    )

    print(f"  Created subtask: {st1['name']}")
    print(f"  Created subtask: {st2['name']}")
    print(f"  Created subtask: {st3['name']}")
    print(f"  Created subtask: {st4['name']}")
    print(f"  Created subtask: {st5['name']}")
    print(f"  Created subtask: {st6['name']}")
    print(f"  Created subtask: {st7['name']}")

    # 8. Create todo items
    print("Creating todo items...")
    todo1 = run_cmd(
        [
            dunno,
            "todo",
            "create",
            "--project-ids",
            p1_id,
            "Add unit tests for auth module",
        ]
    )
    todo2 = run_cmd(
        [dunno, "todo", "create", "--project-ids", p1_id, "Set up CI/CD pipeline"]
    )
    todo3 = run_cmd(
        [dunno, "todo", "create", "--project-ids", p1_id, "Add API documentation"]
    )
    todo4 = run_cmd(
        [dunno, "todo", "create", "--project-ids", p2_id, "Design mobile app mockups"]
    )
    todo5 = run_cmd(
        [dunno, "todo", "create", "--project-ids", p2_id, "User testing sessions"]
    )
    todo6 = run_cmd(
        [dunno, "todo", "create", "--project-ids", p2_id, "Performance optimization"]
    )

    print(f"  Created 6 todo items")

    # 10. Create knowledge entries (mistakes, style rules, security)
    # Link to various levels: project, module, submodule, task, subtask.
    # Reverse edges (belongs_to_project, belongs_to_module, belongs_to_task) are created by the CLI.
    print("Creating knowledge entries...")

    # Linked to modules
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Using unwrap() on user input - should use match or if let",
            "--link-to",
            m1_1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Project-wide: avoid global mutable state",
            "--link-to",
            p1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "style",
            "--field",
            "content",
            "--value",
            "Submodule-level: use same error type in this submodule",
            "--link-to",
            sm1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Task-level: do not log tokens in production",
            "--link-to",
            t1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Subtask-level: refresh token table must be migrated",
            "--link-to",
            st1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Forgetting to index database columns used in WHERE clauses",
            "--link-to",
            m1_2_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "mistake",
            "--field",
            "content",
            "--value",
            "Not validating CSRF tokens on form submissions",
            "--link-to",
            m1_3_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "style",
            "--field",
            "content",
            "--value",
            "Prefer functional style for iterators: vec.iter().map(...).collect()",
            "--link-to",
            m2_2_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "style",
            "--field",
            "content",
            "--value",
            "Use consistent naming: snake_case for variables, CamelCase for types",
            "--link-to",
            m2_1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "security",
            "--field",
            "content",
            "--value",
            "SQL injection risk - use parameterized queries instead of string concatenation",
            "--link-to",
            m1_2_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "security",
            "--field",
            "content",
            "--value",
            "XSS vulnerability - sanitize user input before rendering HTML",
            "--link-to",
            m2_1_id,
        ],
        check=False,
    )
    run_cmd(
        [
            dunno,
            "add",
            "--field",
            "type",
            "--value",
            "security",
            "--field",
            "content",
            "--value",
            "Rate limiting not implemented on login endpoint",
            "--link-to",
            m1_1_id,
        ],
        check=False,
    )

    print(f"  Created 12 knowledge entries")

    print("\n✅ Database seeded successfully!")
    print(f"\nProjects: 2")
    print(f"Modules: 6")
    print(f"Submodules: 3")
    print(f"Files: 6")
    print(f"Tasks: 7")
    print(f"Subtasks: 7")
    print(f"Todo items: 6")
    print(
        f"Knowledge entries: 12 (linked at project, module, submodule, task, subtask)"
    )


def main():
    parser = argparse.ArgumentParser(description="Seed the database with dummy data")
    parser.add_argument(
        "--force", action="store_true", help="Clear existing data before seeding"
    )
    args = parser.parse_args()

    if args.force:
        print("Force flag set - clearing database first...")
        clear_database()

    seed()


if __name__ == "__main__":
    main()
