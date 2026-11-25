import copy
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "contracts/schemas/event.schema.json").read_text())
EXAMPLE = json.loads((ROOT / "contracts/examples/event.example.json").read_text())
VALIDATOR = Draft202012Validator(SCHEMA, format_checker=FormatChecker())


class EventTimestampFormatTests(unittest.TestCase):
    def test_format_checker_includes_date_time(self):
        self.assertIn("date-time", VALIDATOR.FORMAT_CHECKER.checkers)

    def test_valid_rfc3339_timestamp_is_accepted(self):
        self.assertTrue(VALIDATOR.is_valid(EXAMPLE))

    def test_malformed_timestamp_is_rejected(self):
        invalid = copy.deepcopy(EXAMPLE)
        invalid["timestamp"] = "not-a-timestamp"
        self.assertFalse(VALIDATOR.is_valid(invalid))


if __name__ == "__main__":
    unittest.main()
