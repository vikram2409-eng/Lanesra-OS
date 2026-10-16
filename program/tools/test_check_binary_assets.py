"""Self-tests for the binary asset check (WP-0003)."""
import base64
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_binary_assets as cba  # noqa: E402

PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 16


class CheckBinaryAssetsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def write(self, name, data):
        p = self.tmp / name
        p.write_bytes(data)
        return p

    def test_real_png_passes(self):
        # Covers: REQ-PRE-005 (WP-0003)
        self.assertEqual(cba.check([self.write("a.png", PNG)]), [])

    def test_base64_png_is_reported_with_hint(self):
        errors = cba.check([self.write("a.png", base64.b64encode(PNG))])
        self.assertEqual(len(errors), 1)
        self.assertIn("base64", errors[0])

    def test_wrong_format_for_extension_is_reported(self):
        self.assertEqual(len(cba.check([self.write("a.ico", PNG)])), 1)

    def test_other_extensions_are_ignored(self):
        self.assertEqual(cba.check([self.write("a.txt", b"hello")]), [])

    def test_repository_assets_are_valid(self):
        root = Path(__file__).resolve().parents[2]
        self.assertEqual(cba.check(cba.tracked_files(root)), [])


if __name__ == "__main__":
    unittest.main()
