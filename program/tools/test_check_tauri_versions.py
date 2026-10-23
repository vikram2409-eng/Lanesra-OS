"""Self-tests for the Tauri crate and JavaScript API version check."""
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_tauri_versions as ctv  # noqa: E402


class CheckTauriVersionsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def write(self, crate="2.12.5", api="2.12.4"):
        (self.tmp / "Cargo.lock").write_text(
            '[[package]]\nname = "tauri-build"\nversion = "2.7.4"\n\n'
            f'[[package]]\nname = "tauri"\nversion = "{crate}"\n'
        )
        (self.tmp / "package-lock.json").write_text(
            json.dumps({"packages": {"node_modules/@tauri-apps/api": {"version": api}}})
        )

    def test_same_minor_release_passes(self):
        self.write()
        self.assertEqual(ctv.check(self.tmp), [])

    def test_a_different_minor_release_is_reported(self):
        # The desktop-v0.16.0 release failure: crate 2.11, API 2.12.
        self.write(crate="2.11.5", api="2.12.4")
        errors = ctv.check(self.tmp)
        self.assertEqual(len(errors), 1)
        self.assertIn("2.11.5", errors[0])

    def test_the_tauri_crate_is_not_confused_with_tauri_build(self):
        self.write(crate="2.12.0", api="2.12.9")
        self.assertEqual(ctv.check(self.tmp), [])

    def test_repository_versions_match(self):
        desktop = Path(__file__).resolve().parents[2] / "desktop"
        self.assertEqual(ctv.check(desktop), [])


if __name__ == "__main__":
    unittest.main()
