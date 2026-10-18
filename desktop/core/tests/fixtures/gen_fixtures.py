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


def gen(out_dir: Path, version: int, label: str) -> Path:
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
    finally:
        conn.close()
    return path


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", required=True, help="output directory for fixture .db files")
    ap.add_argument("--version", type=int, default=79, help="schema version to generate (default: 79)")
    ap.add_argument("--label", default="pre_0080", help="label suffix (default: pre_0080)")
    args = ap.parse_args(argv)
    path = gen(Path(args.out), args.version, args.label)
    print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
