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


def demo_count_definition():
    """Parse the LANE_WH_COUNT literal out of app.js (WP-0710 AC-5)."""
    text = APP_JS.read_text()
    assign = text.index("const LANE_WH_COUNT = ")
    brace = text.index("{", assign)
    obj, _ = json.JSONDecoder().raw_decode(text[brace:])
    return obj


def example_definition():
    return json.loads(EXAMPLE_PATH.read_text())


def example_count_definition():
    return json.loads(
        (REPO / "program/spec/schemas/examples/lane-wh-count.json").read_text()
    )


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
    # runner and the seeded run rely on: 5 stages, 13 steps (12 + the
    # WP-0716 child step 1.3 verify-supplier), the discrepancy branch and
    # the hold step.
    def test_demo_definition_has_the_worked_example_shape(self):
        lane = demo_definition()["lane"]
        self.assertEqual(lane["id"], "lane-wh-receiving")
        self.assertEqual([s["id"] for s in lane["stages"]],
                         ["verify", "record-receipt", "hold-review",
                          "stage-put-away", "confirm"])
        steps = [st["id"] for s in lane["stages"] for st in s["steps"]]
        self.assertEqual(len(steps), 13)
        for step_id in ("verify-supplier", "discrepancy-check",
                        "review-discrepancy", "record-hold",
                        "post-receipt-moves", "post-putaway-moves",
                        "close-po"):
            self.assertIn(step_id, steps)


class LaneCountDefinitionInDemoTests(unittest.TestCase):
    # Covers: REQ-LANE-017 (WP-0710 AC-5) — the demo runs the shared Count
    # definition, not a copy that drifted.
    def test_demo_count_definition_matches_example_file(self):
        self.assertEqual(demo_count_definition(), example_count_definition())

    # Covers: REQ-LANE-017 (WP-0710 AC-5) — the embedded Count definition
    # validates against the shared schema (the same check CI runs).
    def test_demo_count_definition_validates_against_schema(self):
        self.assertEqual(
            cls.check(schema(), {"lane-wh-count.json": demo_count_definition()}),
            [],
        )

    # Covers: REQ-LANE-017 (WP-0710 AC-5) — the worked example's shape the
    # runner relies on: 5 stages, 7 steps, the per-item assignment on the
    # count step (Q-33), the zero-variance skip on the adjustments step
    # (Q-34), the no-variance stage skip, and the PASS branch that routes
    # a clean count through Post adjustments so the skip can fire.
    def test_demo_count_definition_has_the_worked_example_shape(self):
        lane = demo_count_definition()["lane"]
        self.assertEqual(lane["id"], "lane-wh-count")
        self.assertEqual(
            [s["id"] for s in lane["stages"]],
            ["count", "variance-review", "hold-review",
             "post-adjustments", "confirm-close"],
        )
        steps = {st["id"]: st for s in lane["stages"] for st in s["steps"]}
        self.assertEqual(len(steps), 7)
        for step_id in ("count-lines", "verify-count", "check-variances",
                        "review-variances", "record-hold",
                        "post-count-adjustments", "close-count"):
            self.assertIn(step_id, steps)
        self.assertEqual(
            steps["count-lines"]["repeat_over"]["assign"],
            {"by": "child_field", "field": "counted_by"},
        )
        self.assertEqual(
            steps["post-count-adjustments"]["skip_when"],
            {"field": "$item.variance", "operator": "equals", "value": 0},
        )
        post_stage = [s for s in lane["stages"] if s["id"] == "post-adjustments"][0]
        self.assertEqual(
            post_stage["skip_condition"],
            {"field": "$record.status", "operator": "equals", "value": "COUNTING"},
        )
        check = steps["check-variances"]
        self.assertEqual(
            [b["target"] for b in check["branches"] if b["on"] == "pass"],
            [{"stage": "post-adjustments", "step": "post-count-adjustments"}],
        )


if __name__ == "__main__":
    unittest.main()
