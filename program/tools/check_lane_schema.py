#!/usr/bin/env python3
"""Validate lane definitions against the Process Lane schema (WP-0701, ADR-0024).

Rules:
- program/spec/schemas/process-lane.schema.json is itself a valid JSON Schema.
- Every example in program/spec/schemas/examples/ validates against it.
  An invalid example fails the build (REQ-LANE-001, WP-0701 AC-2), so the
  demo and the core can never drift from the shared definition format.

Requires the `jsonschema` package (installed in CI by the program.yml step
before this check; locally: pip install jsonschema).

Usage: python3 program/tools/check_lane_schema.py   (exit 1 with a message per problem)
"""
import json
import sys
from pathlib import Path

import jsonschema


def check(schema, examples):
    """Validate the schema and every example.

    `schema` is the parsed process-lane.schema.json; `examples` maps a file
    name to its parsed JSON. Returns a list of error strings (empty = ok).
    """
    errors = []
    try:
        jsonschema.validators.validator_for(schema).check_schema(schema)
    except jsonschema.SchemaError as e:
        errors.append(f"process-lane.schema.json is not a valid JSON Schema: {e.message}")
        return errors
    validator = jsonschema.validators.validator_for(schema)(schema)
    if not examples:
        errors.append("program/spec/schemas/examples/ has no lane examples to validate")
    for name, example in sorted(examples.items()):
        for error in sorted(validator.iter_errors(example), key=lambda e: list(e.path)):
            location = ".".join(str(p) for p in error.path) or "(top level)"
            errors.append(f"{name} at {location}: {error.message}")
    return errors


def main():
    root = Path(__file__).resolve().parents[2]
    schema_path = root / "program/spec/schemas/process-lane.schema.json"
    examples_dir = root / "program/spec/schemas/examples"
    try:
        schema = json.loads(schema_path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        print(f"cannot read {schema_path}: {e}")
        sys.exit(1)
    examples = {}
    for path in sorted(examples_dir.glob("*.json")):
        try:
            examples[path.name] = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            print(f"cannot parse {path}: {e}")
            sys.exit(1)
    errors = check(schema, examples)
    for e in errors:
        print(e)
    if errors:
        sys.exit(1)
    print(f"lane schema: {len(examples)} example(s) validate")


if __name__ == "__main__":
    main()
