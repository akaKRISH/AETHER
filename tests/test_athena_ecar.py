"""ATH-02B CLI regressions with public synthetic inputs only."""

from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = ROOT / "docs/research/ATH_02B_ECAR_PROTOCOL.md"
ANALYSIS = ROOT / "scripts/athena_ecar_summary.py"
DEFAULT_EXE = ROOT / "target/debug/examples" / ("ecar_diagnostics.exe" if os.name == "nt" else "ecar_diagnostics")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def public_config(phase):
    return {
        "version": "aether.ecar-config/2", "phase": phase,
        "first_seed": 10, "seed_count": 30 if phase == "heldout" else 2, "seed_key": "ab" * 32,
        "caps": [24, 48],
        "scenarios": [{
            "id": "public_fixture", "parents": [None, None, None], "weights": [7, 9, 3],
            "truth": [60, 80, 99], "replay_cost": 3, "challenge_cost": 5, "noise": 6, "failure": 4,
        }],
    }


class EcarCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        workspace = tempfile.TemporaryDirectory(prefix="athena-public-test-")
        cls.addClassCleanup(workspace.cleanup)
        cls.root = Path(workspace.name)
        cls.exe = Path(os.environ.get("AETHER_ECAR_TEST_RUNNER", DEFAULT_EXE)).resolve()
        cls.config = cls.public_file("config.json", public_config("heldout"))
        cls.pins = {
            "version": "aether.ecar-approval/1",
            "protocol_sha256": sha(PROTOCOL.read_bytes()),
            "runner_sha256": sha(cls.exe.read_bytes()),
            "analysis_sha256": sha(ANALYSIS.read_bytes()),
            "config_sha256": sha(cls.config.read_bytes()),
        }
        cls.manifest = cls.public_file("approval.json", cls.pins)
        cls.approval = sha(cls.manifest.read_bytes())
        cls.arguments = ["--heldout", cls.config, cls.manifest, cls.approval]
        result = cls.invoke(cls.exe, *cls.arguments)
        if result.returncode:
            raise RuntimeError(result.stderr)
        cls.data = json.loads(result.stdout)

    @classmethod
    def public_file(cls, name, data):
        path = cls.root / name
        path.write_bytes(json.dumps(data).encode())
        return path

    @staticmethod
    def invoke(*args):
        return subprocess.run(args, capture_output=True, text=True, timeout=30)

    def reject(self, result, error):
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn(error, result.stderr)

    def analyze(self, data, manifest=None, approval=None):
        report = self.public_file("report.json", data)
        return self.invoke(sys.executable, ANALYSIS, "--heldout", report, self.config,
                           manifest or self.manifest, approval or self.approval)

    def test_public_approval_runs_and_reexports_the_same_cases(self):
        repeat = self.invoke(self.exe, *self.arguments)
        self.assertEqual(repeat.returncode, 0, repeat.stderr)
        self.assertEqual(json.loads(repeat.stdout)["outcomes"], self.data["outcomes"])
        summary = self.analyze(self.data)
        self.assertEqual(summary.returncode, 0, summary.stderr)
        summary = json.loads(summary.stdout)
        self.assertEqual(summary["cases"], 240)
        self.assertEqual(summary["primary_ecar_vs_adaptive_flat"]["mean_points"], 0)
        self.assertEqual(summary["primary_ecar_vs_adaptive_flat"]["seed_blocks"], 30)

    def test_development_v2_runs_and_analyzes(self):
        config = self.public_file("development.json", public_config("development"))
        result = self.invoke(self.exe, "--development-v2", config)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = self.public_file("development-report.json", json.loads(result.stdout))
        result = self.invoke(sys.executable, ANALYSIS, "--development-v2", report, config)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["cases"], 16)
        self.reject(self.invoke(sys.executable, ANALYSIS, report), "version 1 development")

    def test_runner_rejects_implicit_or_missing_approval(self):
        self.reject(self.invoke(self.exe, self.config), "phase")
        self.reject(self.invoke(self.exe, "--development-v2", self.config), "phase")
        self.reject(self.invoke(self.exe, *self.arguments[:-1]), "usage")
        self.reject(self.invoke(self.exe, *self.arguments[:-1], "0" * 64), "approval manifest digest mismatch")

    def test_legacy_runner_is_limited_to_the_archived_pilot_configuration(self):
        config = public_config("development")
        del config["version"], config["seed_key"]
        path = self.public_file("legacy.json", config)
        self.reject(self.invoke(self.exe, path), "exact archived pilot configuration")

    def test_runner_rejects_protocol_runner_and_config_pin_mismatch(self):
        for field, message in (("protocol", "protocol digest"), ("runner", "runner digest"),
                               ("config", "configuration digest")):
            with self.subTest(field=field):
                manifest = self.public_file("unmatched.json", self.pins | {f"{field}_sha256": "0" * 64})
                self.reject(self.invoke(self.exe, "--heldout", self.config, manifest, sha(manifest.read_bytes())),
                            message)

    def test_runner_rejects_repeated_case_dimensions(self):
        for field in ("caps", "scenarios"):
            with self.subTest(field=field):
                config = public_config("heldout")
                config[field].append(config[field][0])
                path = self.public_file("duplicate-grid.json", config)
                manifest = self.public_file("duplicate-grid-approval.json",
                                            self.pins | {"config_sha256": sha(path.read_bytes())})
                self.reject(self.invoke(self.exe, "--heldout", path, manifest, sha(manifest.read_bytes())),
                            "invalid diagnostic specification")

    def test_analyzer_rejects_missing_case(self):
        self.reject(self.analyze(self.data | {"outcomes": self.data["outcomes"][4:]}), "missing cases")

    def test_analyzer_rejects_duplicate_case_arm(self):
        rows = self.data["outcomes"]
        self.reject(self.analyze(self.data | {"outcomes": rows + rows[:1]}), "duplicate case and arm")

    def test_analyzer_rejects_unexpected_case_or_arm(self):
        for changed in ({"seed": 1000}, {"family": "unknown"}, {"cap": 100}, {"mode": "Other"}):
            with self.subTest(changed=changed):
                data = deepcopy(self.data)
                data["outcomes"][0].update(changed)
                self.reject(self.analyze(data), "unexpected case or arm")

    def test_analyzer_rejects_unmatched_arm(self):
        self.reject(self.analyze(self.data | {"outcomes": self.data["outcomes"][1:]}), "unmatched case arms")

    def test_analyzer_rejects_invalid_budget_or_denominator(self):
        for field, value in (("attempts", 49), ("wall_ms", 241), ("total_weight", 0), ("total_claims", 0)):
            with self.subTest(field=field):
                data = deepcopy(self.data)
                row = data["outcomes"][0]
                target = row["used"] if field in {"attempts", "wall_ms"} else row
                target[field] = value
                self.reject(self.analyze(data), "invalid metric denominator, count or budget")

    def test_analyzer_rejects_unmatched_report_pins_and_phase(self):
        for field in ("protocol_sha256", "runner_sha256", "config_sha256", "approval_sha256", "phase"):
            with self.subTest(field=field):
                self.reject(self.analyze(self.data | {field: "0" * 64}), "mismatch")

    def test_analyzer_rejects_unapproved_manifest_or_analysis(self):
        self.reject(self.analyze(self.data, approval="0" * 64), "approval manifest digest mismatch")
        manifest = self.public_file("unapproved-analysis.json", self.pins | {"analysis_sha256": "0" * 64})
        self.reject(self.analyze(self.data, manifest, sha(manifest.read_bytes())), "approval analysis_sha256 mismatch")


if __name__ == "__main__":
    unittest.main()
