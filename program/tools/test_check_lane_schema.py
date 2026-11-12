"""Self-tests for the lane schema check (WP-0701)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_lane_schema as cls  # noqa: E402


def schema():
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["format_version", "lane"],
        "properties": {
            "format_version": {"type": "string", "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$"},
            "lane": {
                "type": "object",
                "required": ["id", "name"],
                "properties": {
                    "id": {"type": "string", "pattern": "^lane-[a-z0-9-]+$"},
                    "name": {"type": "string", "minLength": 1},
                },
            },
        },
    }


def example(**overrides):
    lane = {"id": "lane-test", "name": "Test lane"}
    lane.update(overrides)
    return {"format_version": "1.0.0", "lane": lane}


class CheckLaneSchemaTests(unittest.TestCase):
    # Covers: REQ-LANE-001 (WP-0701 AC-2) — the schema validates the example.
    def test_valid_example_passes(self):
        self.assertEqual(cls.check(schema(), {"lane-test.json": example()}), [])

    # Covers: REQ-LANE-001 (WP-0701 AC-2) — an invalid example fails with its location.
    def test_invalid_example_reports_file_and_location(self):
        errors = cls.check(schema(), {"lane-test.json": example(id="no-prefix")})
        self.assertEqual(len(errors), 1)
        self.assertIn("lane-test.json", errors[0])
        self.assertIn("lane.id", errors[0])

    # Covers: REQ-LANE-001 (WP-0701 AC-2) — no examples to validate is an error.
    def test_missing_example_is_an_error(self):
        errors = cls.check(schema(), {})
        self.assertEqual(len(errors), 1)
        self.assertIn("no lane examples", errors[0])

    # Covers: REQ-LANE-001 (WP-0701 AC-2) — an invalid schema is an error.
    def test_invalid_schema_is_an_error(self):
        bad_schema = {"type": "nonsense"}
        errors = cls.check(bad_schema, {"lane-test.json": example()})
        self.assertEqual(len(errors), 1)
        self.assertIn("not a valid JSON Schema", errors[0])

    # Covers: REQ-LANE-001 (WP-0701 AC-2) — every example is checked, not just the first.
    def test_multiple_examples_all_checked(self):
        errors = cls.check(
            schema(),
            {
                "good.json": example(),
                "bad.json": example(id="bad"),
            },
        )
        self.assertEqual(len(errors), 1)
        self.assertIn("bad.json", errors[0])


if __name__ == "__main__":
    unittest.main()
