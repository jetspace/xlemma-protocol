#!/usr/bin/env python3
"""Validate one #xlemma_export record and print its canonical outer JSON."""

from __future__ import annotations

import json
import sys
from pathlib import Path

MARKER = "XLMP_LEAN_EXPORT "
CANONICAL_FIELDS = ("canonical_declaration_name", "canonical_elaborated_type", "canonical_proof_object")


def die(message: str) -> None:
    raise SystemExit(f"Lean environment export validation failed: {message}")


if len(sys.argv) != 2:
    die("usage: validate-export.py EXPECTED.json")

records = []
for line in sys.stdin:
    if MARKER in line:
        records.append(json.loads(line.split(MARKER, 1)[1]))

if len(records) != 1:
    die(f"expected exactly one export record, found {len(records)}")

record = records[0]
expected = json.loads(Path(sys.argv[1]).read_text())
if record != expected:
    die("export differs from the checked-in deterministic vector")

if record.get("schema") != "xlemma-lean-environment-export/v1":
    die("unexpected schema discriminator")
if record.get("canonical_encoding") != "xlemma-lean-expr-v1":
    die("unexpected expression encoding")
if record.get("protocol_version") != "XLMP/1":
    die("unexpected protocol version")
if record.get("is_unsafe") or record.get("is_partial"):
    die("unsafe or partial declaration reached the export boundary")
if record.get("axioms"):
    die("the self-test declaration unexpectedly depends on axioms")

for field in CANONICAL_FIELDS:
    value = record.get(field)
    if not isinstance(value, str) or not value:
        die(f"{field} is not a nonempty canonical string")
    parsed = json.loads(value)
    canonical = json.dumps(parsed, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    if value != canonical:
        die(f"{field} is not compact canonical JSON")

print(json.dumps(record, ensure_ascii=False, separators=(",", ":"), sort_keys=True))
