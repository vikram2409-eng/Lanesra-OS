#!/usr/bin/env python3
"""Migration order check (ADR-0003, WP-0003).

The migration runner applies only versions above MAX(version) already
applied, so a migration merged out of order is silently skipped forever.
This check fails when:

- migration files are not numbered 0001..N without gaps or duplicates,
- desktop/core/src/db/migrate.rs does not list every file, in order,
  with the matching version number,
- (with --base REF) a migration added on this branch is not numbered
  above the highest migration on the base branch.
- a migration numbered above 0079 lacks the ADR-0010 header block.

    python3 program/tools/check_migrations.py
    python3 program/tools/check_migrations.py --base origin/main

Standard library only.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MIG_DIR = ROOT / "desktop" / "core" / "src" / "db" / "migrations"
MIGRATE_RS = ROOT / "desktop" / "core" / "src" / "db" / "migrate.rs"
FILE_RE = re.compile(r"^(\d{4})_[a-z0-9_]+\.sql$")
LIST_RE = re.compile(r'\(\s*(\d+),\s*include_str!\(\s*"migrations/(\d{4})_([a-z0-9_]+)\.sql"\s*\),?\s*\)')


def files_on_disk(mig_dir: Path) -> list[tuple[int, str]]:
    out = []
    for p in sorted(mig_dir.glob("*.sql")):
        m = FILE_RE.match(p.name)
        if not m:
            raise SystemExit(f"error: migration file name {p.name!r} must look like 0079_short_name.sql")
        out.append((int(m.group(1)), p.name))
    return out


def listed(migrate_rs: Path) -> list[tuple[int, str]]:
    text = migrate_rs.read_text(encoding="utf-8")
    return [(int(v), f"{num}_{name}.sql", int(num)) for v, num, name in LIST_RE.findall(text)]


HEADER_RE = re.compile(
    r"^-- Migration (\d{4}): .+\n"
    r"-- Work package: WP-\d+\n"
    r"-- Change: (additive|data|destructive)( \(.+\))?\n"
    r"-- Data touched: .+\n"
    r"-- Re-run safe: (yes|no)( \(.+\))?\n",
    re.MULTILINE,
)


def check_header(path: Path, num: int) -> str | None:
    """ADR-0010 header check for migrations numbered above 0079 (WP-0007)."""
    if num <= 79:
        return None
    text = path.read_text(encoding="utf-8")
    m = HEADER_RE.match(text)
    if not m:
        return (f"{path.name}: missing or malformed ADR-0010 header; "
                "migrations above 0079 must start with the 5-line header block")
    if int(m.group(1)) != num:
        return f"{path.name}: header says migration {m.group(1)}, file is {num:04d}"
    return None


def check(mig_dir: Path, migrate_rs: Path, base_max: int | None = None, added: list[int] | None = None) -> list[str]:
    errors = []
    disk = files_on_disk(mig_dir)
    numbers = [n for n, _ in disk]
    if len(set(numbers)) != len(numbers):
        dupes = sorted({n for n in numbers if numbers.count(n) > 1})
        errors.append(f"duplicate migration numbers: {dupes}")
    expected = list(range(1, len(set(numbers)) + 1))
    if sorted(set(numbers)) != expected:
        missing = sorted(set(expected) - set(numbers))
        errors.append(f"migration numbers must run 1..N without gaps; missing {missing}")
    for n, name in disk:
        if err := check_header(mig_dir / name, n):
            errors.append(err)
    lst = listed(migrate_rs)
    if [(v, f) for v, f, _ in lst] != [(n, f) for n, f in disk]:
        disk_names = {f for _, f in disk}
        listed_names = {f for _, f, _ in lst}
        for f in sorted(disk_names - listed_names):
            errors.append(f"{f} is not listed in migrate.rs MIGRATIONS")
        for f in sorted(listed_names - disk_names):
            errors.append(f"migrate.rs lists {f}, which does not exist")
        for v, f, num in lst:
            if v != num:
                errors.append(f"migrate.rs lists {f} as version {v}; the version must equal the file number {num}")
        if not errors:
            errors.append("migrate.rs MIGRATIONS must list the migration files in numeric order")
    if base_max is not None and added:
        for n in added:
            if n <= base_max:
                errors.append(f"new migration {n:04d} is not above the base branch's highest migration "
                              f"{base_max:04d}; renumber it to {base_max + 1:04d} or higher (ADR-0003)")
    return errors


def base_state(ref: str) -> tuple[int, list[int]]:
    rel = MIG_DIR.relative_to(ROOT).as_posix()
    base_files = subprocess.run(["git", "ls-tree", "--name-only", f"{ref}:{rel}"], cwd=ROOT,
                                capture_output=True, text=True, check=True).stdout.split()
    base_nums = {int(m.group(1)) for f in base_files if (m := FILE_RE.match(f))}
    head_nums = {n for n, _ in files_on_disk(MIG_DIR)}
    return (max(base_nums) if base_nums else 0), sorted(head_nums - base_nums)


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--base", help="git ref of the base branch, e.g. origin/main")
    args = ap.parse_args(argv)
    base_max, added = (None, None)
    if args.base:
        base_max, added = base_state(args.base)
    errors = check(MIG_DIR, MIGRATE_RS, base_max, added)
    for e in errors:
        print(f"error: {e}", file=sys.stderr)
    total = len(files_on_disk(MIG_DIR))
    extra = f", {len(added)} new on this branch (base max {base_max:04d})" if args.base else ""
    print(f"{total} migrations{extra}, {len(errors)} problem(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
