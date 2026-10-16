"""Self-tests for the image toolchain pin check (WP-0020)."""
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_toolchain_pins as ctp  # noqa: E402


class CheckToolchainPinsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def write(self, rust="1.97.0", node="22", d_rust="1.97.0", d_node="22"):
        (self.tmp / "rust-toolchain.toml").write_text(f'[toolchain]\nchannel = "{rust}"\n')
        (self.tmp / ".nvmrc").write_text(node + "\n")
        (self.tmp / "Dockerfile").write_text(f"ARG NODE_VERSION={d_node}\nARG RUST_VERSION={d_rust}\nFROM scratch\n")

    def test_matching_pins_pass(self):
        # Covers: REQ-PRE-018 (WP-0020 AC-2)
        self.write()
        self.assertEqual(ctp.check(self.tmp), [])

    def test_rust_drift_is_reported(self):
        self.write(d_rust="1.98.0")
        self.assertTrue(any("RUST_VERSION" in e for e in ctp.check(self.tmp)))

    def test_node_drift_is_reported(self):
        self.write(d_node="20")
        self.assertTrue(any("NODE_VERSION" in e for e in ctp.check(self.tmp)))

    def test_node_minor_versions_are_accepted(self):
        self.write(node="22.22.0", d_node="22")
        self.assertEqual(ctp.check(self.tmp), [])

    def test_repository_pins_match(self):
        desktop = Path(__file__).resolve().parents[2] / "desktop"
        self.assertEqual(ctp.check(desktop), [])


if __name__ == "__main__":
    unittest.main()
