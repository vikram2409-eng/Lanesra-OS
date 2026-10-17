"""Self-tests for the program-bot PR comment renderer (WP-0018, REQ-PRE-014)."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pr_report  # noqa: E402

WP_SAMPLE = """---
id: WP-0018
title: Tracker upkeep and test feedback for AIs without a shell
phase: 0
status: in_progress
owner: muse
requirements: [REQ-PRE-014]
editions: []
---

# WP-0018

## Acceptance criteria

- [ ] AC-1: Bot commit fixes STATUS.md
- [x] AC-2: Comment updates in place
"""

VALIDATE_OK = "137 work packages, 238 requirements, 0 error(s), 0 warning(s)\n"
TRACE_SAMPLE = (
    "| Requirement | Tests | Work packages | Coverage |\n"
    "|---|---|---|---|\n"
    "| REQ-PRE-014 | 0 | WP-0018(in_progress) | - |\n"
    "| REQ-PRE-015 | 0 | WP-0010(ready) | - |\n"
)
JOBS_SAMPLE = [
    {"name": "rust (format and lint)", "conclusion": "success", "html_url": "https://example.com/1"},
    {"name": "web (frontend)", "conclusion": "failure", "html_url": "https://example.com/2"},
    {"name": "demo (website and online demo)", "conclusion": "in_progress", "html_url": ""},
]


class Args:
    def __init__(self, d):
        self.validate_output = str(d / "validate.txt")
        self.wp_file = str(d / "WP-0018.md")
        self.trace_output = str(d / "trace.txt")
        self.jobs_json = str(d / "jobs.json")
        self.run_url = "https://example.com/run/1"


class PrReportTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        (self.tmp / "validate.txt").write_text(VALIDATE_OK)
        (self.tmp / "WP-0018.md").write_text(WP_SAMPLE)
        (self.tmp / "trace.txt").write_text(TRACE_SAMPLE)
        (self.tmp / "jobs.json").write_text(json.dumps(JOBS_SAMPLE))
        self.args = Args(self.tmp)

    def test_marker_first_line(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertTrue(out.startswith("<!-- program-bot:report -->"))

    def test_validate_result_shown(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertIn("137 work packages, 238 requirements", out)
        self.assertIn("\u2705 `programctl validate`", out)

    def test_validate_failure_shown(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        (self.tmp / "validate.txt").write_text("2 error(s) in work packages\n")
        out = pr_report.render(self.args)
        self.assertIn("\u274c `programctl validate` failed:", out)

    def test_acceptance_criteria_ticked(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertIn("- [ ] AC-1: Bot commit fixes STATUS.md", out)
        self.assertIn("- [x] AC-2: Comment updates in place", out)

    def test_trace_filtered_to_wp_requirements(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertIn("REQ-PRE-014", out)
        self.assertNotIn("REQ-PRE-015", out)

    def test_job_results_with_failure_link(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertIn("rust (format and lint)", out)
        self.assertIn("\u2705", out)
        self.assertIn("[logs](https://example.com/2)", out)

    def test_untested_locally_note(self):
        # Covers: REQ-PRE-014 (WP-0018 AC-2)
        out = pr_report.render(self.args)
        self.assertIn("Untested locally - see CI", out)


if __name__ == "__main__":
    unittest.main()
