#!/usr/bin/env python3
"""Generate migration-upgrade fixture databases (WP-0007).

Creates SQLite files under desktop/core/tests/fixtures/db/ named
<version>_<label>.db, each at the named schema version with representative
data. Standard library only.

Usage:
    python3 desktop/core/tests/fixtures/gen_fixtures.py --out <dir>
"""
from __future__ import annotations

import argparse
import re
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
MIG_DIR = ROOT / "desktop" / "core" / "src" / "db" / "migrations"
FILE_RE = re.compile(r"^(\d{4})_[a-z0-9_]+\.sql$")


def migrations_up_to(max_version: int) -> list[tuple[int, str]]:
    out = []
    for p in sorted(MIG_DIR.glob("*.sql")):
        m = FILE_RE.match(p.name)
        if not m:
            continue
        n = int(m.group(1))
        if n <= max_version:
            out.append((n, p.read_text(encoding="utf-8")))
    return sorted(out)


def seed(conn: sqlite3.Connection) -> None:
    """Insert representative data; text columns act as canaries."""
    conn.execute(
        "INSERT INTO workspaces (id, business_name, created_at, updated_at)"
        " VALUES ('ws_fixture', 'Fixture Co', '2026-10-17T00:00:00Z', '2026-10-17T00:00:00Z')"
    )
    conn.execute(
        "INSERT INTO companies (id, workspace_id, customer_number, name, status, created_at, updated_at)"
        " VALUES ('c_fixture', 'ws_fixture', 'C-FIXTURE', 'Fixture Industries', 'Active Customer',"
        " '2026-10-17T00:00:00Z', '2026-10-17T00:00:00Z')"
    )
    conn.commit()


def seed_triggers(conn: sqlite3.Connection) -> None:
    """Users, an agent and unattended triggers (WP-0109 AC-2 upgrade fixture).

    One trigger was created by a Sales user, one by an Administrator, and
    one has no creator, so the upgrade must fall back to the first
    Administrator as the owner of its Agent Identity.
    """
    ts = "2026-10-17T00:00:00Z"
    conn.executemany(
        "INSERT OR IGNORE INTO roles (id, name) VALUES (?, ?)",
        [("role_admin", "Administrator"), ("role_sales", "Sales")],
    )
    for uid, name, created in [
        ("u_admin", "admin", "2026-10-01T00:00:00Z"),
        ("u_sales", "sales", "2026-10-02T00:00:00Z"),
    ]:
        conn.execute(
            "INSERT INTO users (id, workspace_id, username, display_name, password_hash, created_at, updated_at)"
            " VALUES (?, 'ws_fixture', ?, ?, 'x', ?, ?)",
            (uid, name, name.title(), created, created),
        )
    conn.execute(
        "INSERT INTO user_roles (user_id, role_id) SELECT 'u_admin', id FROM roles WHERE name = 'Administrator'"
    )
    conn.execute(
        "INSERT INTO user_roles (user_id, role_id) SELECT 'u_sales', id FROM roles WHERE name = 'Sales'"
    )
    conn.execute(
        "INSERT INTO ai_agents (id, workspace_id, name, system_prompt, action_names_json, created_at, updated_at)"
        " VALUES ('a_fixture', 'ws_fixture', 'Fixture agent', 'Summarise.', '[]', ?, ?)",
        (ts, ts),
    )
    conn.executemany(
        "INSERT INTO ai_agent_triggers (id, workspace_id, target_type, target_id, trigger_type,"
        " interval_minutes, created_at, created_by) VALUES (?, 'ws_fixture', 'agent', 'a_fixture', ?, ?, ?, ?)",
        [
            ("t_sched_sales", "schedule", 60, ts, "u_sales"),
            ("t_hook_admin", "webhook", None, ts, "u_admin"),
            ("t_sched_orphan", "schedule", 30, ts, None),
        ],
    )
    conn.commit()


def seed_runs(conn: sqlite3.Connection) -> None:
    """A published graph and runs in every state the 0062 schema allowed (WP-0110 AC-1, AC-3).

    Two runs sit in the retired waiting_tool and waiting_agent statuses, which
    the upgrade must turn into paused/interrupted. A waiting_scheduled run is
    due, so the upgraded runtime can finish it through the new guard.
    """
    ts = "2026-10-17T00:00:00Z"
    conn.execute(
        "INSERT INTO ai_execution_graphs (id, workspace_id, name, status, created_at, updated_at)"
        " VALUES ('g_fixture', 'ws_fixture', 'Fixture graph', 'published', ?, ?)",
        (ts, ts),
    )
    conn.executemany(
        "INSERT INTO ai_graph_nodes (id, graph_id, node_key, node_type, sort_order) VALUES (?, 'g_fixture', ?, ?, ?)",
        [("n_trigger", "trigger", "trigger", 0), ("n_end", "end", "end", 1)],
    )
    conn.execute(
        "INSERT INTO ai_graph_edges (id, graph_id, from_node_id, to_node_id) VALUES ('e_1', 'g_fixture', 'n_trigger', 'n_end')"
    )
    runs = [
        ("r_completed", "completed", "n_end", None, None, ts),
        ("r_failed", "failed", "n_trigger", None, "provider unavailable", ts),
        ("r_waiting_tool", "waiting_tool", "n_trigger", None, None, None),
        ("r_waiting_agent", "waiting_agent", "n_trigger", None, None, None),
        ("r_scheduled", "waiting_scheduled", "n_end", "2026-10-17T01:00:00Z", None, None),
    ]
    for rid, status, node, resume_at, error, finished in runs:
        conn.execute(
            "INSERT INTO ai_runs (id, workspace_id, graph_id, status, trigger_input, context_json,"
            " current_node_id, resume_at, error_message, steps_executed, started_at, finished_at)"
            " VALUES (?, 'ws_fixture', 'g_fixture', ?, '{\"topic\":\"fixture\"}', '{\"trigger\":{\"ok\":true}}',"
            " ?, ?, ?, 1, ?, ?)",
            (rid, status, node, resume_at, error, ts, finished),
        )
    conn.execute(
        "INSERT INTO ai_run_nodes (id, run_id, node_id, node_key, node_type, attempt, status,"
        " input_json, output_json, started_at, finished_at)"
        " VALUES ('rn_1', 'r_completed', 'n_trigger', 'trigger', 'trigger', 1, 'completed',"
        " '{}', '{\"ok\":true}', ?, ?)",
        (ts, ts),
    )
    conn.commit()


def gen(out_dir: Path, version: int, label: str, triggers: bool = False, runs: bool = False) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"{version:04d}_{label}.db"
    if path.exists():
        path.unlink()
    conn = sqlite3.connect(str(path))
    try:
        for n, sql in migrations_up_to(version):
            conn.executescript(sql)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
            )
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?, datetime('now'))",
                (n,),
            )
        seed(conn)
        if triggers:
            seed_triggers(conn)
        if runs:
            seed_runs(conn)
    finally:
        conn.close()
    return path


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", required=True, help="output directory for fixture .db files")
    ap.add_argument("--version", type=int, default=79, help="schema version to generate (default: 79)")
    ap.add_argument("--label", default="pre_0080", help="label suffix (default: pre_0080)")
    ap.add_argument(
        "--with-triggers",
        action="store_true",
        help="also seed users, an agent and unattended triggers (WP-0109)",
    )
    ap.add_argument(
        "--with-runs",
        action="store_true",
        help="also seed a published graph and runs in every 0062 status (WP-0110)",
    )
    args = ap.parse_args(argv)
    path = gen(Path(args.out), args.version, args.label, args.with_triggers, args.with_runs)
    print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
