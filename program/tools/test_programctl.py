"""Self-tests for programctl. Run: python3 -m unittest discover -s program/tools -p 'test_*.py'"""
import shutil
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import programctl as pc  # noqa: E402

REQS = textwrap.dedent("""\
    # Requirements
    - **REQ-AAA-001** [P0] First requirement
    - **REQ-AAA-002** [P1] Second requirement
    """)


def wp_text(wid="WP-0001", phase="0", status="ready", deps="", reqs="REQ-AAA-001, REQ-AAA-002",
            owner="unassigned", pr="", acs="- [ ] AC-1: something", extra=""):
    return textwrap.dedent(f"""\
        ---
        id: {wid}
        title: Test package
        phase: {phase}
        epic: E0.1 Test
        status: {status}
        owner: {owner}
        depends_on: [{deps}]
        requirements: [{reqs}]
        estimate: M
        editions: [desktop, workspace]
        branch:
        pr: {pr}
        updated: 2026-10-16
        ---

        # {wid}

        ## Goal

        Goal.

        ## Acceptance criteria

        {acs}

        ## Test plan

        - a test
        """) + extra


class Base(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.saved = (pc.ROOT, pc.PROGRAM, pc.WORK, pc.REQS, pc.STATUS, pc.LOGS)
        pc.ROOT = self.tmp
        pc.PROGRAM = self.tmp / "program"
        pc.WORK = pc.PROGRAM / "work"
        pc.REQS = pc.PROGRAM / "spec" / "REQUIREMENTS.md"
        pc.STATUS = pc.PROGRAM / "STATUS.md"
        pc.LOGS = pc.PROGRAM / "log"
        pc.REQS.parent.mkdir(parents=True)
        pc.REQS.write_text(REQS)

    def tearDown(self):
        pc.ROOT, pc.PROGRAM, pc.WORK, pc.REQS, pc.STATUS, pc.LOGS = self.saved
        shutil.rmtree(self.tmp)

    def add(self, text, wid="WP-0001", phase="0"):
        d = pc.WORK / f"P{phase}"
        d.mkdir(parents=True, exist_ok=True)
        (d / f"{wid}-test.md").write_text(text)

    def check(self, strict=True):
        reqs, _ = pc.load_requirements()
        return pc.validate(pc.load_work_packages(), reqs, strict_coverage=strict)


class ProgramctlTest(Base):
    def test_valid_package_passes(self):
        self.add(wp_text())
        errors, _ = self.check()
        self.assertEqual(errors, [])

    def test_parses_lists_and_scalars(self):
        self.add(wp_text())
        w = pc.load_work_packages()["WP-0001"]
        self.assertEqual(w["requirements"], ["REQ-AAA-001", "REQ-AAA-002"])
        self.assertEqual(w["depends_on"], [])
        self.assertEqual(w["status"], "ready")

    def test_uncovered_requirement_is_error_or_warning(self):
        self.add(wp_text(reqs="REQ-AAA-001"))
        errors, _ = self.check()
        self.assertTrue(any("REQ-AAA-002" in e for e in errors))
        errors, warnings = self.check(strict=False)
        self.assertEqual(errors, [])
        self.assertTrue(any("REQ-AAA-002" in w for w in warnings))

    def test_unknown_requirement_and_dependency(self):
        self.add(wp_text(deps="WP-0099", reqs="REQ-AAA-001, REQ-AAA-002, REQ-ZZZ-009"))
        errors, _ = self.check()
        self.assertTrue(any("WP-0099" in e for e in errors))
        self.assertTrue(any("REQ-ZZZ-009" in e for e in errors))

    def test_dependency_cycle_detected(self):
        self.add(wp_text(wid="WP-0001", deps="WP-0002"), wid="WP-0001")
        self.add(wp_text(wid="WP-0002", deps="WP-0001"), wid="WP-0002")
        errors, _ = self.check()
        self.assertTrue(any("cycle" in e for e in errors))

    def test_wrong_phase_directory(self):
        self.add(wp_text(phase="1"), phase="0")
        errors, _ = self.check()
        self.assertTrue(any("live in work/P1/" in e for e in errors))

    def test_in_progress_needs_owner_and_done_needs_pr_and_ticked_acs(self):
        self.add(wp_text(status="in_progress"))
        self.assertTrue(any("needs an owner" in e for e in self.check()[0]))
        self.add(wp_text(status="done", owner="codex"))
        errors = self.check()[0]
        self.assertTrue(any("needs a pr link" in e for e in errors))
        self.assertTrue(any("unticked acceptance criteria" in e for e in errors))
        self.add(wp_text(status="done", owner="codex", pr="https://example/pr/1", acs="- [x] AC-1: ok")
                 .replace("branch:", "delivered: [desktop, workspace]\nbranch:"))
        self.assertEqual(self.check()[0], [])

    def test_done_requires_dependencies_done(self):
        self.add(wp_text(wid="WP-0001", status="ready", reqs="REQ-AAA-001"), wid="WP-0001")
        self.add(wp_text(wid="WP-0002", status="done", owner="x", pr="p", acs="- [x] AC-1: ok",
                         deps="WP-0001", reqs="REQ-AAA-002").replace("branch:", "delivered: [desktop, workspace]\nbranch:"), wid="WP-0002")
        self.assertTrue(any("dependency WP-0001 is ready" in e for e in self.check()[0]))

    def test_blocked_needs_blocker_section(self):
        self.add(wp_text(status="blocked", owner="human-x"))
        self.assertTrue(any("Blocker" in e for e in self.check()[0]))
        self.add(wp_text(status="blocked", owner="human-x", extra="\n## Blocker\n\nWaiting."))
        self.assertEqual(self.check()[0], [])

    def test_ready_needs_required_sections(self):
        self.add(wp_text().replace("## Test plan", "## Tests"))
        self.assertTrue(any("'## Test plan'" in e for e in self.check()[0]))

    def test_malformed_requirement_line_rejected(self):
        pc.REQS.write_text(REQS + "- **REQ-AAA-003** P1 missing brackets\n")
        with self.assertRaises(pc.Problem):
            pc.load_requirements()

    def test_ready_to_start_respects_dependencies(self):
        self.add(wp_text(wid="WP-0001", status="ready", reqs="REQ-AAA-001"), wid="WP-0001")
        self.add(wp_text(wid="WP-0002", status="ready", deps="WP-0001", reqs="REQ-AAA-002"), wid="WP-0002")
        self.assertEqual(pc.ready_to_start(pc.load_work_packages()), ["WP-0001"])

    def test_status_render_and_check(self):
        self.add(wp_text())
        self.assertEqual(pc.main(["status"]), 0)
        self.assertIn("WP-0001", pc.STATUS.read_text())
        self.assertEqual(pc.main(["status", "--check"]), 0)
        pc.STATUS.write_text("stale")
        self.assertEqual(pc.main(["status", "--check"]), 1)


class EditionTest(Base):
    def test_done_needs_every_edition_delivered(self):
        txt = wp_text(status="done", owner="x", pr="p", acs="- [x] AC-1: ok")
        self.add(txt)
        self.assertTrue(any("not delivered in ['desktop', 'workspace']" in e for e in self.check()[0]))
        self.add(txt.replace("branch:", "delivered: [desktop, workspace]\nbranch:"))
        self.assertEqual(self.check()[0], [])

    def test_bad_edition_rejected(self):
        self.add(wp_text().replace("editions: [desktop, workspace]", "editions: [mobile]"))
        self.assertTrue(any("editions" in e for e in self.check()[0]))


class ExportTest(Base):
    def test_export_sequences_and_attributes_builders(self):
        self.add(wp_text(wid="WP-0001", reqs="REQ-AAA-001"), wid="WP-0001")
        log = "\n## Progress log\n\n### 2026-10-16 muse\nDid the first half.\n\n### 2026-10-17 claude\nFinished it.\n"
        self.add(wp_text(wid="WP-0002", deps="WP-0001", reqs="REQ-AAA-002") + log, wid="WP-0002")
        reqs, _ = pc.load_requirements()
        data = pc.export_data(pc.load_work_packages(), reqs)
        by = {w["id"]: w for w in data["work_packages"]}
        self.assertEqual([w["id"] for w in data["work_packages"]], ["WP-0001", "WP-0002"])
        self.assertEqual((by["WP-0001"]["wave"], by["WP-0002"]["wave"]), (1, 2))
        self.assertEqual(by["WP-0002"]["built_by"], ["muse", "claude"])
        self.assertEqual(by["WP-0001"]["unblocks"], ["WP-0002"])
        self.assertEqual(data["critical_path"], ["WP-0001", "WP-0002"])
        self.assertEqual({r["id"]: r["state"] for r in data["requirements"]}["REQ-AAA-001"], "planned")


class BriefTest(Base):
    def test_brief_is_self_contained(self):
        self.add(wp_text(reqs="REQ-AAA-001, REQ-AAA-002"))
        wps = pc.load_work_packages()
        reqs, _ = pc.load_requirements()
        card = pc.render_brief(wps["WP-0001"], wps, reqs)
        self.assertIn("REQ-AAA-001: First requirement", card)
        self.assertIn("## What you must deliver", card)
        self.assertIn("No previous entry", card)

    def test_agent_field_is_rejected(self):
        self.add(wp_text().replace("owner: unassigned\n", "owner: unassigned\nagent: muse\n"))
        self.assertTrue(any("not pre-assigned" in e for e in self.check()[0]))


if __name__ == "__main__":
    unittest.main()
