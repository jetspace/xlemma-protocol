#!/usr/bin/env python3
"""Exercise the Rust discovery pilot against honest and adversarial fixtures.

This runner never implements a second reward allocator. The deliberately
undetected semantic-duplicate case measures the external assessor dependency.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def synthetic_id(prefix: str, label: str) -> str:
    """Synthetic fixture identity, not a formal ClaimID derivation."""
    return f"{prefix}:blake3:{hashlib.sha256(label.encode()).hexdigest()}"


def scenarios(pilot: dict) -> list[tuple[str, dict, str | None, set[str], set[str]]]:
    base = copy.deepcopy(pilot)
    base["events"] = base["events"][:6] + [{"kind": "finalize", "at": 210}]
    honest = {e["submission"]["submission_id"] for e in base["events"][:6]}
    extra_id = pilot["events"][6]["submission"]["submission_id"]
    cases = [("all_categories", base, None, set(), honest),
             ("grouping_appeal_restores_legitimate_work", pilot, None, set(), honest | {extra_id})]
    wrong = copy.deepcopy(pilot)
    wrong["events"] = wrong["events"][:7] + [{"kind": "finalize", "at": 210}]
    cases.append(("overbroad_grouping_without_appeal", wrong, None, set(), honest | {extra_id}))
    flooded = copy.deepcopy(base)
    farmer_ids = set()
    for i in range(20):
        s = copy.deepcopy(base["events"][0]["submission"])
        s["submission_id"] = synthetic_id("xlr", f"partition-{i}")
        s["contributors"][0]["researcher_id"] = synthetic_id("xlresearcher", f"farmer-{i}")
        s["assessed_weight"] = 1000
        farmer_ids.add(s["submission_id"])
        flooded["events"].insert(6+i, {"kind": "submit", "at": 7+i, "submission": s})
    cases.append(("recognized_partition_and_identity_flood", flooded, None, farmer_ids, honest))
    hidden = copy.deepcopy(base)
    s = copy.deepcopy(base["events"][0]["submission"])
    s["submission_id"] = synthetic_id("xlr", "hidden-duplicate")
    s["group_id"] = synthetic_id("xlgroup", "incorrect-fresh-group")
    s["claims"] = [synthetic_id("xlc", "synthetic-unrecognized-variant")]
    hidden["events"].insert(6, {"kind": "submit", "at": 7, "submission": s})
    cases.append(("unrecognized_semantic_duplicate", hidden, None, {s["submission_id"]}, honest))
    inflated = copy.deepcopy(base)
    inflated["events"][0]["submission"]["reported_tokens"] = 9_000_000_000_000
    inflated["events"][0]["submission"]["reported_compute_units"] = 9_000_000_000_000
    cases.append(("inflated_compute_telemetry", inflated, None, set(), honest))
    replay = copy.deepcopy(base)
    replay["previously_rewarded_groups"] = [replay["events"][0]["submission"]["group_id"]]
    cases.append(("cross_round_group_replay", replay, None,
                  {replay["events"][0]["submission"]["submission_id"]}, honest - {replay["events"][0]["submission"]["submission_id"]}))
    unfunded = copy.deepcopy(base)
    unfunded["funding"][0]["gross_units"] = 1
    unfunded["funding"][0]["administrator_fee_units"] = 0
    unfunded["pledged_units"] = 8_000_000_000
    cases.append(("pledges_cannot_cover_restricted_shortfall", unfunded, "category budget exceeds restricted funding", set(), set()))
    replay_funds = copy.deepcopy(base)
    replay_funds["funding"].append(copy.deepcopy(replay_funds["funding"][0]))
    cases.append(("settlement_replay", replay_funds, "reused funding settlement", set(), set()))
    captured = copy.deepcopy(pilot)
    captured["events"][8]["reviewers"] = captured["policy"]["assessors"]
    cases.append(("original_panel_cannot_hear_appeal", captured, "unqualified or conflicted review panel", set(), set()))
    capacity = copy.deepcopy(base)
    capacity["policy"]["maximum_submissions"] = 5
    cases.append(("submission_flood_hits_capacity", capacity, "submission window or capacity exhausted", set(), set()))
    pending = copy.deepcopy(pilot)
    pending["events"] = pending["events"][:8]
    cases.append(("pending_appeal_holds_batch", pending, None, set(), set()))
    early = copy.deepcopy(pending)
    early["events"].append({"kind": "finalize", "at": 210})
    cases.append(("cannot_finalize_pending_appeal", early, "unresolved appeal holds entire allocation batch", set(), set()))
    expired = copy.deepcopy(pending)
    expired["events"].append({"kind": "expire", "at": 301})
    cases.append(("timeout_preserves_unresolved_case", expired, None, set(), set()))
    divergent = copy.deepcopy(pilot)
    divergent["events"][6]["submission"]["declared_evidence_status"] = "divergent"
    cases.append(("economic_appeal_cannot_override_divergence", divergent, None, {extra_id}, honest))
    null = copy.deepcopy(base)
    null["events"][3]["submission"]["registered_study"]["outcome"] = "null"
    cases.append(("replication_method_paid_for_null_outcome", null, None, set(), honest))
    retrospective = copy.deepcopy(base)
    retrospective["events"][3]["submission"]["registered_study"]["registered_at"] = 2
    cases.append(("retrospective_registration_rejected", retrospective, "registration must precede", set(), set()))
    return cases


def run_suite(binary: Path) -> dict:
    pilot = json.loads((ROOT / "examples/discovery/pilot.json").read_text())
    results = []
    full_reports = {}
    with tempfile.TemporaryDirectory(prefix="xlemma-discovery-") as temp:
        source = Path(temp) / "scenario.json"
        for name, scenario, expected_error, farmer_ids, honest_ids in scenarios(pilot):
            source.write_text(json.dumps(scenario))
            result = subprocess.run([str(binary), "simulate-discovery", str(source)],
                                    capture_output=True, text=True, timeout=60, check=False)
            if expected_error:
                if result.returncode == 0 or expected_error not in result.stderr:
                    raise AssertionError(f"{name}: expected rejection {expected_error!r}; {result.stderr}")
                results.append({"name": name, "outcome": "rejected_as_expected", "reason": expected_error})
                continue
            if result.returncode:
                raise AssertionError(f"{name}: {result.stderr}")
            r = json.loads(result.stdout)
            full_reports[name] = r
            assert r["simulation_only"] and not r["authenticates_evidence"] and not r["executes_payments"]
            assert r["declared_funding_units"] == sum(r[k] for k in ["administrator_fees_units", "verification_spent_units",
                    "appeal_spent_units", "allocated_units", "retained_units"])
            paid_ids = {a["submission_id"] for a in r["allocations"] if sum(a["contributor_units"].values())}
            leakage = sum(sum(a["contributor_units"].values()) for a in r["allocations"] if a["submission_id"] in farmer_ids)
            exclusions = len(honest_ids - paid_ids)
            expected_leakage = 150_000_000 if name == "unrecognized_semantic_duplicate" else 0
            expected_exclusions = 1 if name == "overbroad_grouping_without_appeal" else 0
            assert leakage == expected_leakage, (name, leakage)
            assert exclusions == expected_exclusions, (name, exclusions)
            results.append({"name": name, "outcome": "modeled", "state": r["state"],
                            "allocated_units": r["allocated_units"], "retained_units": r["retained_units"],
                            "reward_leakage_units": leakage, "legitimate_excluded_count": exclusions,
                            "unresolved_appeals": r["unresolved_appeals"],
                            "verification_spent_units": r["verification_spent_units"],
                            "appeal_spent_units": r["appeal_spent_units"]})
    assert full_reports["all_categories"]["allocations"] == full_reports["inflated_compute_telemetry"]["allocations"]
    assert full_reports["all_categories"]["allocations"] == full_reports["replication_method_paid_for_null_outcome"]["allocations"]
    expected = json.loads((ROOT / "examples/discovery/expected-report.json").read_text())
    assert full_reports["grouping_appeal_restores_legitimate_work"] == expected, "pilot vector drift"
    return {"simulation_only": True, "schema_version": 1, "scenario_count": len(results),
            "all_expected_behaviors_observed": True,
            "limitations": ["Synthetic adversarial cases are not real-world detection-rate estimates.",
                "Undetected semantic duplication deliberately leaks 150 USDC in one case; independent grouping remains an activation gate.",
                "One legitimate contribution is excluded by overbroad grouping and restored by the modeled independent appeal.",
                "No cryptographic authentication, checker execution, laboratory validation, or USDC transfer occurs."],
            "scenarios": results}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, help="Use this already-built xlemma binary")
    parser.add_argument("--output", type=Path, default=ROOT / "reports/discovery-simulation.json")
    parser.add_argument("--check", action="store_true", help="Compare the committed report without modifying it")
    args = parser.parse_args()
    if args.binary is None:
        build = subprocess.run(["cargo", "build", "--locked", "-p", "xlemma-cli", "--message-format=json"],
                               cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True)
        artifacts = [json.loads(line) for line in build.stdout.splitlines()]
        executables = [a["executable"] for a in artifacts if a.get("reason") == "compiler-artifact"
                       and a.get("target", {}).get("name") == "xlemma-cli" and a.get("executable")]
        if len(executables) != 1:
            raise SystemExit("could not locate the built xlemma-cli executable")
        args.binary = Path(executables[0])
    binary = args.binary.resolve()
    report = run_suite(binary)
    encoded = json.dumps(report, indent=2) + "\n"
    if args.check:
        if args.output.read_text() != encoded:
            raise SystemExit("discovery report differs; regenerate and review it")
    else:
        args.output.write_text(encoded)
    print(f"{report['scenario_count']} discovery scenarios matched expectations, including documented assessor failure cases")


if __name__ == "__main__":
    main()
