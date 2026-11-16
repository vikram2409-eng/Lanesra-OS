"""Self-tests for styles.css brace balance (warehouse mobile fix, 2026-11-15).

A missing closing brace silently killed ~194 lines of CSS: on 2026-10-13
commit fb2f8fc opened `@media(min-width:641px){` for the og-compact-toolbar
block and never closed it, so every rule after it — the whole lane-runner
styling, the mobile topbar fix, the layout collapse — was nested inside that
media query and inert below 641px. The demo looked fine on desktop and PR
#780's polish never showed up on phones. The live site shipped that way for
a month because nothing checked it.

These tests fail if the braces ever go unbalanced again, and pin the exact
regression: the og-compact-toolbar media block must close before the
og-mockup-parity rules that follow it.
"""

import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


def styles_css():
    return (REPO_ROOT / "styles.css").read_text(encoding="utf-8")


class StylesCssBracesTests(unittest.TestCase):
    # The whole file must balance: a missing brace nests every later rule
    # inside the unclosed block and silently disables it on some widths.
    def test_braces_balance(self):
        source = styles_css()
        self.assertEqual(
            source.count("{"),
            source.count("}"),
            "styles.css braces are unbalanced — a missing } nests every "
            "later rule inside the unclosed block",
        )

    # The 2026-11-15 regression, pinned: the og-compact-toolbar
    # @media(min-width:641px) block must close before the og-mockup-parity
    # comment, so the parity rules (and everything after them) are top-level.
    def test_compact_toolbar_media_block_closes(self):
        source = styles_css()
        media_start = source.index("@media(min-width:641px){")
        parity_start = source.index("/* og-mockup-parity")
        self.assertGreater(parity_start, media_start)
        between = source[media_start:parity_start]
        self.assertEqual(
            between.count("{"),
            between.count("}"),
            "the @media(min-width:641px) block is not closed before "
            "/* og-mockup-parity */ — everything after it is nested inside "
            "the media query",
        )
