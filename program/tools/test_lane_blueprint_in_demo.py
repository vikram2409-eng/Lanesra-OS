"""Self-tests for the lane blueprint in the online demo (WP-0712).

Static checks that the demo wires the blueprint in: the renderer, the
route, the dashboard entry point and the print button in app.js, the
print rules in styles.css, and the shared example the renderer reads.
The rendered output itself is covered by the Playwright spec
(desktop/e2e/tests/warehouse-lane-blueprint.spec.ts) and the Vitest
snapshot of the same definition.
"""
import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


def app_js():
    return (REPO_ROOT / "app.js").read_text(encoding="utf-8")


def styles_css():
    return (REPO_ROOT / "styles.css").read_text(encoding="utf-8")


def example():
    return json.loads(
        (
            REPO_ROOT
            / "program/spec/schemas/examples/lane-wh-receiving.json"
        ).read_text(encoding="utf-8")
    )


class LaneBlueprintInDemoTests(unittest.TestCase):
    # Covers: REQ-LANE-020 (WP-0712 AC-5) - the renderer exists in app.js.
    def test_renderer_exists(self):
        source = app_js()
        self.assertIn("function renderLaneBlueprint(def)", source)
        self.assertIn("function laneBlueprintPage()", source)
        self.assertIn("function laneBlueprintDef(id)", source)

    # Covers: REQ-LANE-020 (WP-0712 AC-5) - the route is dispatched.
    def test_route_dispatched(self):
        self.assertIn("if(current==='laneBlueprint') return laneBlueprintPage();", app_js())

    # Covers: REQ-LANE-020 (WP-0712 AC-5) - the dashboard links to it.
    def test_dashboard_entry_point(self):
        source = app_js()
        self.assertIn('data-lane-blueprint="lane-wh-receiving"', source)
        self.assertIn("View blueprint", source)

    # Covers: REQ-LANE-020 (WP-0712 AC-5) - the demo reads the shared
    # example, the same definition the core imports.
    def test_demo_reads_the_shared_example(self):
        self.assertIn(
            "fetch('/program/spec/schemas/examples/lane-wh-receiving.json')",
            app_js(),
        )
        lane = example()["lane"]
        self.assertEqual(lane["id"], "lane-wh-receiving")
        self.assertEqual(len(lane["stages"]), 5)
        steps = [step for stage in lane["stages"] for step in stage["steps"]]
        self.assertEqual(len(steps), 12)

    # Covers: REQ-LANE-020 (WP-0712 AC-2) - the print button and the print
    # rules that leave only the blueprint on the sheet.
    def test_print_button_and_rules(self):
        self.assertIn("data-lane-blueprint-print", app_js())
        self.assertIn("window.print()", app_js())
        css = styles_css()
        self.assertIn("@media print", css)
        for selector in (".sidebar", ".topbar", ".demo-banner", ".bottom-nav"):
            self.assertIn(selector, css)


if __name__ == "__main__":
    unittest.main()
