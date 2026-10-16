"""Self-test for the GitHub Pages wrapper of the dashboard (WP-0019)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dashboard_page as dp  # noqa: E402


class WrapTest(unittest.TestCase):
    def test_real_page_becomes_a_standalone_utf8_document(self):
        # Covers: REQ-PRE-017 (WP-0019 AC-3)
        src = (Path(__file__).resolve().parents[1] / "dashboard" / "index.html").read_text(encoding="utf-8")
        out = dp.wrap(src)
        self.assertTrue(out.startswith("<!doctype html>"))
        self.assertIn('<meta charset="utf-8">', out)
        self.assertLess(out.index("<title>"), out.index("</head>"))
        self.assertLess(out.index("</head>"), out.index('<nav class="tabs'))
        self.assertTrue(out.rstrip().endswith("</html>"))

    def test_page_without_style_block_is_rejected(self):
        with self.assertRaises(SystemExit):
            dp.wrap("<title>x</title><p>no style</p>")


if __name__ == "__main__":
    unittest.main()
