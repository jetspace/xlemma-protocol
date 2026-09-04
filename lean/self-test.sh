#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

expected_version="Lean (version 4.33.1,"
actual_version="$(lean --version)"
if [[ "$actual_version" != "$expected_version"* ]]; then
  printf 'Lean pseudo self-test failed: expected 4.33.1, got %s\n' "$actual_version" >&2
  exit 1
fi

lake build

export_output="$(lake env lean -DwarningAsError=true XLemma/Example.lean 2>&1)"
export_record="$(printf '%s\n' "$export_output" | python3 validate-export.py ../examples/lean-export/expected-add-zero.json)"
if [[ "$export_output" != *"does not depend on any axioms"* ]]; then
  printf 'Lean pseudo self-test failed: exact theorem axiom inventory was not empty\n' >&2
  exit 1
fi

set +e
unsafe_output="$(lake env lean -DwarningAsError=true tests/RejectUnsafe.lean 2>&1)"
unsafe_status=$?
valueless_output="$(lake env lean -DwarningAsError=true tests/RejectValueless.lean 2>&1)"
valueless_status=$?
set -e
if [[ "$unsafe_status" -eq 0 || "$unsafe_output" != *"refusing to export unsafe declaration"* ]]; then
  printf 'Lean pseudo self-test failed: unsafe declaration was not rejected\n' >&2
  exit 1
fi
if [[ "$valueless_status" -eq 0 || "$valueless_output" != *"has no checker-consumable value"* ]]; then
  printf 'Lean pseudo self-test failed: valueless declaration was not rejected\n' >&2
  exit 1
fi

lake env leanchecker --fresh XLemma.Example
cargo test --quiet --locked --manifest-path ../Cargo.toml -p xlemma-core lean_export::tests
cargo test --quiet --locked --manifest-path ../Cargo.toml -p xlemma-lean

printf '%s\n' "$actual_version"
printf '%s\n' "$export_record"
printf 'xLemma Lean pseudo self-test passed (author-operated, not independent verification)\n'
