"""Self-tests for the migration order check (ADR-0003, WP-0003)."""
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_migrations as cm  # noqa: E402

RS_HEAD = "const MIGRATIONS: &[(i64, &str)] = &[\n"


class CheckMigrationsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.mig = self.tmp / "migrations"
        self.mig.mkdir()
        self.rs = self.tmp / "migrate.rs"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def setup(self, files, listed, multiline=False):
        for f in files:
            (self.mig / f).write_text("-- sql")
        rows = []
        for v, f in listed:
            if multiline:
                rows.append(f'    (\n        {v},\n        include_str!("migrations/{f}"),\n    ),\n')
            else:
                rows.append(f'    ({v}, include_str!("migrations/{f}")),\n')
        self.rs.write_text(RS_HEAD + "".join(rows) + "];\n")

    def test_contiguous_and_listed_passes(self):
        # Covers: REQ-PRE-005 (WP-0003)
        self.setup(["0001_a.sql", "0002_b.sql"], [(1, "0001_a.sql"), (2, "0002_b.sql")])
        self.assertEqual(cm.check(self.mig, self.rs), [])

    def test_rustfmt_multiline_entries_are_understood(self):
        self.setup(["0001_a.sql", "0002_b.sql"], [(1, "0001_a.sql"), (2, "0002_b.sql")], multiline=True)
        self.assertEqual(cm.check(self.mig, self.rs), [])

    def test_gap_is_reported(self):
        self.setup(["0001_a.sql", "0003_c.sql"], [(1, "0001_a.sql"), (3, "0003_c.sql")])
        self.assertTrue(any("without gaps" in e for e in cm.check(self.mig, self.rs)))

    def test_unlisted_file_is_reported(self):
        self.setup(["0001_a.sql", "0002_b.sql"], [(1, "0001_a.sql")])
        self.assertTrue(any("0002_b.sql is not listed" in e for e in cm.check(self.mig, self.rs)))

    def test_version_must_match_file_number(self):
        self.setup(["0001_a.sql", "0002_b.sql"], [(1, "0001_a.sql"), (3, "0002_b.sql")])
        self.assertTrue(any("must equal the file number" in e for e in cm.check(self.mig, self.rs)))

    def test_new_migration_not_above_base_is_reported(self):
        self.setup(["0001_a.sql", "0002_b.sql"], [(1, "0001_a.sql"), (2, "0002_b.sql")])
        errors = cm.check(self.mig, self.rs, base_max=2, added=[2])
        self.assertTrue(any("renumber it to 0003" in e for e in errors))
        self.assertEqual(cm.check(self.mig, self.rs, base_max=1, added=[2]), [])

    def test_bad_file_name_is_rejected(self):
        self.setup(["0001_A-B.sql"], [])
        with self.assertRaises(SystemExit):
            cm.check(self.mig, self.rs)


if __name__ == "__main__":
    unittest.main()
