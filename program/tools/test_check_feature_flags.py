"""Self-tests for the feature flag check (WP-0010)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_feature_flags as cff  # noqa: E402


def rust(stage="Building", desktop="false", key="playground.shell", registered=True, wp="WP-0203"):
    reg = "&[&PLAYGROUND_SHELL]" if registered else "&[]"
    return f'''
pub const PLAYGROUND_SHELL: Flag = Flag {{
    key: "{key}",
    description: "x",
    work_package: "{wp}",
    remove_by: "WP-0223",
    default: Defaults {{
        desktop: {desktop},
        workspace: false,
    }},
    stage: Stage::{stage},
}};

pub const REGISTRY: &[&Flag] = {reg};
'''


def demo(*keys):
    inner = ",".join(f"'{k}'" for k in keys)
    return f"const DEMO_RELEASED_FEATURES=[{inner}];\n"


class CheckFeatureFlagsTest(unittest.TestCase):
    def test_valid_registry_and_empty_demo_pass(self):
        # Covers: REQ-PRE-021 (WP-0010)
        self.assertEqual(cff.check(rust(), demo()), [])

    def test_demo_may_not_show_an_unreleased_flag(self):
        # Covers: REQ-PRE-021 (WP-0010)
        errors = cff.check(rust(stage="Preview"), demo("playground.shell"))
        self.assertTrue(any("not yet released" in e for e in errors))

    def test_demo_may_show_a_released_flag(self):
        self.assertEqual(cff.check(rust(stage="Ga"), demo("playground.shell")), [])

    def test_unknown_demo_key_is_reported(self):
        self.assertTrue(any("unknown flag" in e for e in cff.check(rust(), demo("nope.flag"))))

    def test_building_flag_defaulting_on_is_reported(self):
        self.assertTrue(any("default off" in e for e in cff.check(rust(desktop="true"), demo())))

    def test_unregistered_flag_and_bad_key_are_reported(self):
        errors = cff.check(rust(registered=False, key="Bad-Key", wp="none"), demo())
        self.assertTrue(any("not listed in REGISTRY" in e for e in errors))
        self.assertTrue(any("lower snake case" in e for e in errors))
        self.assertTrue(any("work_package" in e for e in errors))

    def test_repository_flags_pass(self):
        root = Path(__file__).resolve().parents[2]
        rs = (root / "desktop/core/src/services/feature_flag_service.rs").read_text()
        js = (root / "app.js").read_text()
        self.assertEqual(cff.check(rs, js), [])


if __name__ == "__main__":
    unittest.main()
