"""Validate APO-11 benchmark and golden API fixtures against the frozen schema."""

import json
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "contracts" / "apo11"
schema = json.loads((CONTRACT / "quality-evaluation.schema.json").read_text())
Draft202012Validator.check_schema(schema)

benchmark = json.loads((CONTRACT / "benchmark.json").read_text())
benchmark_schema = {
    "$schema": schema["$schema"],
    "$defs": schema["$defs"],
    "$ref": "#/$defs/benchmark",
}
Draft202012Validator(benchmark_schema).validate(benchmark)

validator = Draft202012Validator(schema)
fixtures = sorted((CONTRACT / "fixtures").glob("*.json"))
if not fixtures:
    raise SystemExit("no APO-11 JSON fixtures found")
for path in fixtures:
    validator.validate(json.loads(path.read_text()))

print(f"validated benchmark and {len(fixtures)} APO-11 fixtures")
