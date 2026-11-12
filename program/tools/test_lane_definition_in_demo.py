"""Self-tests: the lane definition embedded in the demo (WP-0702).

The demo runs the Receive Shipment lane from a JSON literal in app.js
(LANE_WH_RECEIVING). That literal must stay identical to the shared example
file and must validate against the shared schema — otherwise the demo and
the editions it previews would drift apart silently.
"""
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_lane_schema as cls  # noqa: E402

REPO = Path(__file__).resolve().parent.parent.parent
APP_JS = REPO / "app.js"
SCHEMA_PATH = REPO / "program/spec/schemas/process-lane.schema.json"
EXAMPLE_PATH = REPO / "program/spec/schemas/examples/lane-wh-receiving.json"


def demo_definition():
    """Parse the LANE_WH_RECEIVING literal out of app.js (raw_decode respects
    string contents, so braces inside descriptions cannot fool it)."""
    text = APP_JS.read_text()
    assign = text.index("const LANE_WH_RECEIVING = ")
    brace = text.index("{", assign)
    obj, _ = json.JSONDecoder().raw_decode(text[brace:])
    return obj


def example_definition():
    return json.loads(EXAMPLE_PATH.read_text())


def schema():
    return json.loads(SCHEMA_PATH.read_text())


class LaneDefinitionInDemoTests(unittest.TestCase):
    # Covers: REQ-LANE-025 (WP-0702 AC-5) — the demo runs the shared
    # definition, not a copy that drifted.
    def test_demo_definition_matches_example_file(self):
        self.assertEqual(demo_definition(), example_definition())

    # Covers: REQ-LANE-025 (WP-0702 AC-5) — the embedded definition
    # validates against the shared schema (the same check CI runs).
    def test_demo_definition_validates_against_schema(self):
        self.assertEqual(
            cls.check(schema(), {"lane-wh-receiving.json": demo_definition()}), []
        )

    # Covers: REQ-LANE-025 (WP-0702 AC-5) — the worked example's shape the
    # runner and the seeded run rely on: 5 stages, 12 steps, the
    # discrepancy branch and the hold step.
    def test_demo_definition_has_the_worked_example_shape(self):
        lane = demo_definition()["lane"]
        self.assertEqual(lane["id"], "lane-wh-receiving")
        self.assertEqual([s["id"] for s in lane["stages"]],
                         ["verify", "record-receipt", "hold-review",
                          "stage-put-away", "confirm"])
        steps = [st["id"] for s in lane["stages"] for st in s["steps"]]
        self.assertEqual(len(steps), 12)
        for step_id in ("discrepancy-check", "review-discrepancy",
                        "record-hold", "post-receipt-moves",
                        "post-putaway-moves", "close-po"):
            self.assertIn(step_id, steps)


if __name__ == "__main__":
    unittest.main()
