"""Self-tests for the GitHub Issues mirror (no network)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import issues_mirror as im  # noqa: E402
import programctl as pc  # noqa: E402


class IssuesMirrorTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.wps = pc.load_work_packages()

    def test_every_work_package_gets_a_create_with_labels_and_milestone(self):
        changes = im.plan_changes(self.wps, {}, "o/r")
        self.assertEqual(len(changes), len(self.wps))
        c = next(c for c in changes if c["wid"] == "WP-0001")
        self.assertEqual(c["action"], "create")
        self.assertTrue(c["title"].startswith("[WP-0001] "))
        self.assertIn("program", c["labels"])
        self.assertIn("phase:0", c["labels"])
        self.assertEqual(c["milestone_title"], "Phase 0 · Precursors")
        self.assertTrue(c["body"].startswith("<!-- programctl:WP-0001 -->"))

    def test_unchanged_issue_produces_no_update_and_changed_one_does(self):
        first = {c["wid"]: c for c in im.plan_changes(self.wps, {}, "o/r")}
        c = first["WP-0002"]
        issue = {"number": 7, "title": c["title"], "body": c["body"], "state": c["state"],
                 "labels": [{"name": n} for n in c["labels"]], "milestone": {"title": c["milestone_title"]}}
        self.assertEqual([x for x in im.plan_changes(self.wps, {"WP-0002": issue}, "o/r") if x["wid"] == "WP-0002"
                          and x["action"] == "update"], [])
        issue["state"] = "closed" if c["state"] == "open" else "open"
        upd = [x for x in im.plan_changes(self.wps, {"WP-0002": issue}, "o/r") if x["wid"] == "WP-0002"]
        self.assertEqual(upd[0]["action"], "update")
        self.assertEqual(upd[0]["number"], 7)

    def test_dependencies_link_to_issue_numbers_once_known(self):
        wp = self.wps["WP-0003"]
        body = im.render_body(wp, "o/r", {"WP-0002": 42})
        self.assertIn("#42 (WP-0002)", body)

    def test_closed_states(self):
        wp = dict(self.wps["WP-0002"], status="dropped")
        c = im.plan_changes({"WP-0002": wp}, {}, "o/r")[0]
        self.assertEqual((c["state"], c["state_reason"]), ("closed", "not_planned"))
        wp = dict(self.wps["WP-0002"], status="done")
        c = im.plan_changes({"WP-0002": wp}, {}, "o/r")[0]
        self.assertEqual((c["state"], c["state_reason"]), ("closed", "completed"))


if __name__ == "__main__":
    unittest.main()
