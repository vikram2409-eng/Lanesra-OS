"""Self-tests for the desktop / Team Workspace release workflow check (ADR-0019)."""
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_release_workflows as crw  # noqa: E402

REPO = Path(__file__).resolve().parents[2] / ".github" / "workflows"


def tagged(tag):
    return f'on:\n  push:\n    tags:\n      - "{tag}"\n'


class CheckReleaseWorkflowsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def write(self, **overrides):
        files = {
            "desktop-release.yml": tagged("desktop-v*"),
            "publish-release.yml": "name: Publish desktop release\n",
            "server-image-release.yml": tagged("team-workspace-server-v*") + "# desktop-v is only a comment\n",
            "publish-server-release.yml": "name: Publish server\n",
        }
        files.update(overrides)
        for name, text in files.items():
            (self.tmp / name).write_text(text)

    def test_the_real_workflows_are_separate(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-1, AC-2)
        self.assertEqual(crw.check(REPO), [])

    def test_a_clean_layout_passes(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-1)
        self.write()
        self.assertEqual(crw.check(self.tmp), [])

    def test_a_server_build_on_the_desktop_tag_is_reported(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-1)
        self.write(**{"server-image-release.yml": tagged("desktop-v*")})
        errors = crw.check(self.tmp)
        self.assertTrue(any("team-workspace-server-v*" in e for e in errors), errors)
        self.assertTrue(any("must not use desktop-v tags" in e for e in errors), errors)

    def test_a_desktop_build_on_the_server_tag_is_reported(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-1)
        self.write(**{"desktop-release.yml": tagged("team-workspace-server-v*")})
        self.assertTrue(any("desktop-v*" in e for e in crw.check(self.tmp)))

    def test_the_desktop_publisher_touching_the_image_is_reported(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-2)
        self.write(**{"publish-release.yml": "run: docker push ghcr.io/x/lanesra-os-server:latest\n"})
        errors = crw.check(self.tmp)
        self.assertTrue(any("ghcr.io" in e for e in errors), errors)

    def test_comments_do_not_count(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-2)
        self.write(**{"publish-release.yml": "# this used to move ghcr.io latest\nname: x\n"})
        self.assertEqual(crw.check(self.tmp), [])

    def test_a_missing_workflow_is_reported(self):
        # Covers: REQ-PRE-024 (WP-0200 AC-1)
        self.write()
        (self.tmp / "publish-server-release.yml").unlink()
        self.assertTrue(any("missing" in e for e in crw.check(self.tmp)))


if __name__ == "__main__":
    unittest.main()
