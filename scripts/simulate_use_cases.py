#!/usr/bin/env python3
"""Run the documented xLemma journeys as one reproducible conformance suite."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples" / "no-arbitrage"
BUNDLE = ROOT / "examples" / "deterministic-bundle"
LEAN_EXPORT = ROOT / "examples" / "lean-export"


Validator = Callable[[str], tuple[bool, str]]


@dataclass(frozen=True)
class Gate:
    gate_id: str
    title: str
    command: tuple[str, ...]
    validator: Validator


@dataclass(frozen=True)
class Scenario:
    scenario_id: str
    title: str
    protocol_path: str
    positive_outcome: str
    negative_invariant: str
    gate_ids: tuple[str, ...]
    required_tests: tuple[str, ...]


def parse_json_output(output: str) -> object:
    return json.loads(output)


def validate_structure(output: str) -> tuple[bool, str]:
    passed = "xLemma repository structural validation passed" in output
    return passed, "schemas, examples, source tree, invariants, and manifest passed"


def validate_trust(output: str) -> tuple[bool, str]:
    value = parse_json_output(output)
    passed = isinstance(value, dict) and value.get("accepted") is True and not value.get("reasons")
    evidence = "content-derived policy and axiom profile accepted exact evidence"
    return passed, evidence


def validate_formal(output: str) -> tuple[bool, str]:
    value = parse_json_output(output)
    passed = (
        isinstance(value, dict)
        and value.get("status") == "reproduced"
        and value.get("pass_count") == 3
        and value.get("fail_count") == 0
        and value.get("verified_users") == 3
        and value.get("operators") == 3
        and value.get("operator_clusters") == 3
        and value.get("checker_families") == {"lean_kernel": 2, "nanoda": 1}
        and not value.get("reasons")
    )
    return passed, "3 independent observations: 2 Lean kernel + 1 nanoda"


def validate_lean_export_ids(output: str) -> tuple[bool, str]:
    actual = parse_json_output(output)
    expected = json.loads((LEAN_EXPORT / "expected-ids.json").read_text())
    return (
        actual == expected,
        "checked environment vector produced its exact TheoryID-bound ClaimID and ProofID",
    )


def validate_generalized(output: str) -> tuple[bool, str]:
    passed = parse_json_output(output) == "certified"
    return passed, "computational profile reached its independent reproduction threshold"


def validate_portability(output: str) -> tuple[bool, str]:
    passed = output.strip().startswith("xlportability:blake3:")
    return passed, "portable exit manifest reconstructed across independent locations"


def validate_economic_compliance(output: str) -> tuple[bool, str]:
    passed = output.strip().startswith("xleconcert:blake3:")
    return passed, "economic compliance validated without changing research validity"


def validate_economics(output: str) -> tuple[bool, str]:
    passed = "economic sanity simulation passed" in output
    return passed, "backing, settlement, refund, revenue, compounding, and impact caps conserved"


def validate_quote(output: str) -> tuple[bool, str]:
    value = parse_json_output(output)
    passed = (
        isinstance(value, dict)
        and value.get("quality_adjusted_certification_cost", {}).get("units", 0) > 0
        and len(value.get("selected_offer_ids", [])) == 6
        and value.get("gold_success_probability_bps") == 7_500
    )
    return passed, "six service offers routed using independently signed success calibration"


def validate_impact(output: str) -> tuple[bool, str]:
    value = parse_json_output(output)
    passed = (
        isinstance(value, dict)
        and value.get("payable_allocation", {}).get("units") == 671_000
        and value.get("payable_allocation", {}).get("units", 0)
        <= value.get("revenue_cap", {}).get("units", -1)
        and value.get("payable_allocation", {}).get("units", 0)
        <= value.get("impact_pool_cap", {}).get("units", -1)
    )
    return passed, "671000 minor units remained below revenue and authorized-pool caps"


def validate_bundle(output: str) -> tuple[bool, str]:
    actual = parse_json_output(output)
    expected = json.loads((BUNDLE / "expected-bundle.json").read_text())
    return actual == expected, "complete bundle object matched the published deterministic vector"


def run_command(command: tuple[str, ...], timeout: int) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    environment["CARGO_TERM_COLOR"] = "never"
    return subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )


def command_text(command: tuple[str, ...]) -> str:
    display = list(command)
    if display and Path(display[0]).resolve() == Path(sys.executable).resolve():
        display[0] = "python3"
    return shlex.join(display)


def gates() -> tuple[Gate, ...]:
    cli = ("cargo", "run", "--quiet", "-p", "xlemma-cli", "--")
    return (
        Gate(
            "repository_structure",
            "Repository and schema integrity",
            (
                sys.executable,
                "scripts/validate_repo.py",
                "--skip-simulation-report",
            ),
            validate_structure,
        ),
        Gate(
            "rust_workspace",
            "Rust workspace regression suite",
            ("cargo", "test", "--locked", "--workspace", "--quiet"),
            lambda _: (True, "all discovered Rust tests passed"),
        ),
        Gate(
            "durable_protocol_history",
            "Durable XLMP history and restart recovery",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-api",
                "tests::accepted_xlmp_message_survives_api_restart",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "signed XLMP ingress was fsynced, reopened, hash-chain verified, and retrieved after restart",
            ),
        ),
        Gate(
            "binary_transport_identity",
            "Canonical non-HTTP XLMP framing",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-xlmp",
                "framing::tests::binary_transport_round_trip_preserves_message_identity",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "the published HTTP envelope survived canonical binary framing with the identical MessageID",
            ),
        ),
        Gate(
            "native_protocol_projection",
            "Native XLMP lifecycle projection",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-xlmp",
                "projection::tests::complete_native_research_lifecycle_replays_without_losing_lineage",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "native identity, research, proof, rights, economics, publication, challenge, and availability objects replayed into one deterministic state root",
            ),
        ),
        Gate(
            "native_api_projection",
            "Authenticated native-object API projection",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-api",
                "tests::xlmp_ingress_is_append_only_and_retrievable",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "a signed native Claim was accepted once and retrieved through both MessageID and ClaimID projections",
            ),
        ),
        Gate(
            "https_transport_policy",
            "Allowlisted XLMP-over-HTTPS transport",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-xlmp",
                "http_transport::tests::endpoint_policy_rejects_http_credentials_redirect_targets_and_unlisted_hosts",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "transport construction rejected plaintext, credential-bearing, and unlisted endpoints",
            ),
        ),
        Gate(
            "filesystem_storage_adapter",
            "Immutable content-addressed storage adapter",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-storage",
                "tests::filesystem_adapter_round_trips_exact_multifile_bundle_and_rejects_overwrite",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "a multi-file artifact round-tripped byte-for-byte, emitted a signed availability receipt, and rejected overwrite",
            ),
        ),
        Gate(
            "x402_payment_adapter",
            "Concrete x402 payment adapter",
            (
                "cargo",
                "test",
                "--locked",
                "-p",
                "xlemma-x402",
                "tests::concrete_adapter_settles_actual_usage_once_and_preserves_payment_separation",
                "--",
                "--exact",
            ),
            lambda _: (
                True,
                "maximum authorization settled only actual usage, preserved facilitator evidence, and rejected replay without affecting research validity",
            ),
        ),
        Gate(
            "lean_pseudo_self_test",
            "Pinned Lean author-operated pseudo self-test",
            ("lean/self-test.sh",),
            lambda output: (
                "xLemma Lean pseudo self-test passed" in output,
                "the pinned exporter, exact identity vector, negative fixtures, fresh bundled checker, and Rust adapter bindings passed locally",
            ),
        ),
        Gate(
            "trust_policy",
            "Trust-policy and axiom gate",
            cli
            + (
                "verify-trust",
                "examples/no-arbitrage/trust-policy-registry.json",
                "examples/no-arbitrage/theory.json",
                "examples/no-arbitrage/proof.json",
                "examples/no-arbitrage/proof-trust-evidence.json",
            ),
            validate_trust,
        ),
        Gate(
            "lean_export_identity",
            "Lean environment export identity",
            cli
            + (
                "lean-export-ids",
                "examples/lean-export/expected-add-zero.json",
                "examples/no-arbitrage/theory.json",
            ),
            validate_lean_export_ids,
        ),
        Gate(
            "formal_poir",
            "Formal PoIR reproduction",
            cli
            + (
                "evaluate-consensus",
                "examples/no-arbitrage/policy.json",
                "examples/no-arbitrage/observations.json",
            ),
            validate_formal,
        ),
        Gate(
            "generalized_verification",
            "Computational-profile reproduction",
            cli
            + (
                "evaluate-reproduction",
                "examples/no-arbitrage/computational-verification-profile.json",
                "examples/no-arbitrage/computational-verification-job.json",
                "examples/no-arbitrage/computational-observations.json",
            ),
            validate_generalized,
        ),
        Gate(
            "portable_exit",
            "Researcher portability and exit",
            cli
            + (
                "verify-portability",
                "examples/no-arbitrage/portability-manifest.json",
            ),
            validate_portability,
        ),
        Gate(
            "economic_compliance",
            "Economic-constitution compliance",
            cli
            + (
                "verify-economic-compliance",
                "examples/no-arbitrage/economic-constitution.json",
                "examples/no-arbitrage/economic-compliance-certificate.json",
            ),
            validate_economic_compliance,
        ),
        Gate(
            "economic_conservation",
            "Credit and revenue conservation",
            (sys.executable, "scripts/simulate_economics.py"),
            validate_economics,
        ),
        Gate(
            "compute_quote",
            "Calibrated compute-market quote",
            cli
            + (
                "quote",
                "examples/no-arbitrage/compute-offers.json",
                "examples/no-arbitrage/expected-work.json",
                "examples/no-arbitrage/protocol-success-estimates.json",
                "--trusted-estimator",
                "ed25519:6kpsY-KcUgq-9VB7Ey7F-ZVHdq6-vnuSQh7qaRRG0iw",
                "--deadline",
                "2027-09-04T20:00:00Z",
                "--quoted-at",
                "2026-09-04T20:00:00Z",
            ),
            validate_quote,
        ),
        Gate(
            "compute_impact",
            "Bounded compute-impact allocation",
            cli
            + (
                "compute-impact",
                "examples/no-arbitrage/compute-savings-evidence.json",
                "examples/no-arbitrage/compute-savings-policy.json",
                "examples/no-arbitrage/downstream-net-revenue.json",
                "examples/no-arbitrage/impact-pool-authorization.json",
                "--trusted-authorizer",
                "ed25519:_RckOFqgx1tk-3jNYC-h2ZH96_drE8WO1wLqyDXp9hg",
            ),
            validate_impact,
        ),
        Gate(
            "deterministic_bundle",
            "Deterministic artifact bundle",
            cli
            + (
                "pack",
                "examples/deterministic-bundle",
                "examples/deterministic-bundle/inputs.json",
                "--lean-toolchain",
                "leanprover/lean4:v4.33.1",
                "--dependency-lock-hash",
                "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--source-commit",
                "vector-1",
                "--build-image-digest",
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "--created-at",
                "2026-09-04T12:00:00Z",
            ),
            validate_bundle,
        ),
    )


def scenarios() -> tuple[Scenario, ...]:
    baseline = (
        "repository_structure",
        "rust_workspace",
        "durable_protocol_history",
        "binary_transport_identity",
        "native_protocol_projection",
        "native_api_projection",
    )
    return (
        Scenario(
            "UC-01",
            "Researcher onboarding",
            "identity → sovereignty bundle → backed vault → policy selection",
            "The researcher obtains portable protocol state and mints credits only against backing.",
            "Missing sovereignty protections or insufficient backing is rejected.",
            baseline + ("trust_policy", "portable_exit", "x402_payment_adapter"),
            (
                "sovereignty::tests::sovereignty_bundle_requires_every_durable_right",
                "credit::tests::credits_remain_fully_backed_through_usage_settlement",
            ),
        ),
        Scenario(
            "UC-02",
            "Create and verify a lemma",
            "CLAIM → COMMIT → FORMALIZE → PROVE → REPRODUCE → CERTIFY",
            "Three independent observations reproduce one exact artifact under the selected trust policy.",
            "A producer cannot self-certify, and missing or divergent evidence cannot advance.",
            baseline
            + (
                "trust_policy",
                "lean_export_identity",
                "lean_pseudo_self_test",
                "deterministic_bundle",
                "filesystem_storage_adapter",
                "formal_poir",
            ),
            (
                "trust::tests::registered_policy_accepts_exact_fail_closed_evidence",
                "formal::tests::gold_quorum_certifies_only_unanimous_exact_reproduction",
            ),
        ),
        Scenario(
            "UC-03",
            "Pay with researcher credits",
            "deposit backing → mint Rᵢ → authorize maximum → settle actual → unlock remainder",
            "Only consumed credits burn and release an equal amount of neutral backing.",
            "Over-authorization, replay, or under-collateralization fails without partial mutation.",
            baseline + ("economic_conservation", "x402_payment_adapter"),
            (
                "credit::tests::credits_remain_fully_backed_through_usage_settlement",
                "credit::tests::forged_or_cloned_authorization_cannot_unlock_another_reservation",
            ),
        ),
        Scenario(
            "UC-04",
            "Earn from a verified result",
            "settled external revenue → costs/refunds/reserves → bounded waterfall → cash + compounding",
            "Net revenue is conserved and auto-compounded credits receive matching vault backing.",
            "Token appreciation, unsettled revenue, or an unauthorized impact signal cannot fund payouts.",
            baseline
            + (
                "economic_conservation",
                "economic_compliance",
                "compute_impact",
                "x402_payment_adapter",
            ),
            (
                "revenue::tests::revenue_is_conserved_across_waterfall_and_creator_rounding",
                "dividend::tests::compute_signal_without_economic_authorization_cannot_pay",
            ),
        ),
        Scenario(
            "UC-05",
            "Support a decentralized researcher",
            "grant / bounty / compute pre-purchase / public-goods support → typed funding receipt",
            "Funding has an explicit rail, backing source, restrictions, and settlement evidence.",
            "Self-issued inflation and vague passive-profit promises do not qualify as funding.",
            baseline + ("economic_compliance",),
            (
                "funding::tests::protocol_fees_conserve_value_and_fund_all_infrastructure",
                "funding::tests::inflation_without_settlement_cannot_be_funding",
            ),
        ),
        Scenario(
            "UC-06",
            "Operate a prover node",
            "advertise capacity → calibrated quote → candidate generation → compute receipt",
            "A provider-neutral prover can earn for bounded work and return a candidate artifact.",
            "The proof producer is excluded from independent reproduction of its own candidate.",
            baseline + ("compute_quote", "https_transport_policy"),
            (
                "protocol::tests::producer_cannot_count_as_an_independent_reproducer",
                "tests::endpoint_must_be_https_and_explicitly_allowlisted",
            ),
        ),
        Scenario(
            "UC-07",
            "Operate a checker node",
            "credential chain → advertisement → sortition → exact execution → commit/reveal → work payment",
            "Distinct verified users, operators, clusters, providers, regions, and checker families reproduce the job.",
            "More machines under common control do not create more independence or truth weight.",
            baseline + ("formal_poir", "lean_pseudo_self_test", "https_transport_policy"),
            (
                "committee::tests::selection_is_reproducible_and_operator_independent",
                "formal::tests::multiple_nodes_under_one_verified_user_are_not_independent_observations",
            ),
        ),
        Scenario(
            "UC-08",
            "Challenge a certificate",
            "challenge → counterevidence → expanded reproduction → dismiss / quarantine / reject",
            "A valid challenge can move the object into fail-closed quarantine and later revalidation.",
            "Checker divergence is never resolved by a 2-to-1 majority or by slashing honest dissent.",
            baseline + ("https_transport_policy",),
            (
                "formal::tests::checker_disagreement_is_divergent_not_majority_vote",
                "tests::divergent_reproduction_can_fail_closed",
                "capture::tests::honest_divergence_is_not_a_slashable_offense",
            ),
        ),
        Scenario(
            "UC-09",
            "Reuse an upstream lemma",
            "final proof dependency → separate economic edge → settled revenue → bounded nonrecursive pool",
            "Eligible upstream contributors can receive a capped allocation from an authorized pool.",
            "Formal dependency alone creates no debt; stuffing, cycles, dust, and recursive charging are blocked.",
            baseline + ("compute_impact", "x402_payment_adapter"),
            (
                "upstream::tests::one_pool_is_bounded_clustered_and_conserved",
                "protocol::tests::formal_dependency_without_an_economic_edge_never_creates_payment",
            ),
        ),
        Scenario(
            "UC-10",
            "Publish a negative result",
            "failed or inconclusive work → attributable evidence artifact → commons/public-goods funding",
            "Useful negative evidence remains publishable and fundable without a false validity badge.",
            "Negative results cannot be relabeled as certified proofs or used to mint unbacked credits.",
            baseline + ("generalized_verification", "filesystem_storage_adapter"),
            ("funding::tests::negative_results_are_commons_not_market_funding",),
        ),
        Scenario(
            "UC-11",
            "Correct or supersede work",
            "new immutable object → AMENDS / CORRECTS / SUPERSEDES edge → revalidation",
            "The old artifact, attribution, receipts, and correction lineage remain reconstructable.",
            "A superseded record cannot silently return to published state or overwrite history.",
            baseline + ("portable_exit", "filesystem_storage_adapter"),
            (
                "state::tests::supersession_is_append_only_and_cannot_restore_old_publication_state",
                "marketplace::tests::order_book_preserves_superseded_advertisements",
            ),
        ),
    )


def test_inventory(timeout: int) -> tuple[set[str], str | None]:
    command = ("cargo", "test", "--locked", "--workspace", "--", "--list")
    try:
        result = run_command(command, timeout)
    except subprocess.TimeoutExpired:
        return set(), "Rust test inventory timed out"
    if result.returncode != 0:
        return set(), "Rust test inventory failed"
    tests = {
        line.rsplit(": test", 1)[0].strip()
        for line in result.stdout.splitlines()
        if line.rstrip().endswith(": test")
    }
    return tests, None


def render_markdown(report: dict[str, object]) -> str:
    summary = report["summary"]
    lines = [
        "# xLemma use-case simulation report",
        "",
        f"Generated: {report['generated_at']}",
        "",
        "> This is deterministic reference-implementation evidence, not a security audit,",
        "> independent mathematical verification, live settlement attestation, or production certification.",
        "",
        "## Executive result",
        "",
        f"**{summary['passed_scenarios']}/{summary['scenario_count']} documented journeys passed** "
        f"across **{summary['passed_gates']}/{summary['gate_count']} executable gates** and "
        f"**{summary['rust_test_count']} Rust tests**.",
        "",
        "| ID | Documented journey | Result | Executable gates |",
        "|---|---|---:|---|",
    ]
    for scenario in report["scenarios"]:
        gates_text = ", ".join(f"`{gate}`" for gate in scenario["gates"])
        lines.append(
            f"| {scenario['id']} | {scenario['title']} | **{scenario['status'].upper()}** | {gates_text} |"
        )

    lines.extend(["", "## Executable gate results", ""])
    for gate in report["gates"]:
        lines.extend(
            [
                f"### {gate['id']} — {gate['title']}",
                "",
                f"- Result: **{gate['status'].upper()}**",
                f"- Evidence: {gate['evidence']}",
                f"- Command: `{gate['command']}`",
                "",
            ]
        )

    lines.extend(
        [
            "## End-to-end journey traces",
            "",
            "Each ordered trace is accepted only when every linked executable gate and named regression test passes. The trace demonstrates reference-implementation conformance; external services listed in the limitations remain simulated adapters.",
            "",
        ]
    )
    for scenario in report["scenarios"]:
        lines.extend(
            [
                f"### {scenario['id']} — {scenario['title']}",
                "",
                f"- Protocol path: {scenario['protocol_path']}",
                "- Ordered trace: "
                + " → ".join(
                    f"{step['sequence']}. {step['step']} [{step['status'].upper()}]"
                    for step in scenario["trace"]
                ),
                f"- Simulated outcome: {scenario['positive_outcome']}",
                f"- Fail-closed invariant: {scenario['negative_invariant']}",
                "- Regression evidence: "
                + ", ".join(f"`{test}`" for test in scenario["required_tests"]),
                "",
            ]
        )

    lines.extend(
        [
            "## Integration defects found and corrected",
            "",
            "1. The formal-policy schema and example allowed a zero requirement for an optional checker family, while the Rust validator rejected it. The validator now requires at least one positive family and permits explicit optional zero entries.",
            "2. The published formal observation vector still contained illustrative receipt IDs, evidence roots, and commitments. The CLI now prepares arrays of content-derived observations, and the vector was regenerated so direct PoIR evaluation succeeds.",
            "3. Several native research objects existed only as Rust/domain concepts. XLMP now carries them as content-derived messages and replays their prerequisites, immutable lineage, and publication/economic relationships into a deterministic projection.",
            "4. Payment, storage, and outbound HTTP boundaries were interface-only. Concrete fail-closed reference adapters now exercise x402 actual-use settlement and replay protection, immutable multi-file storage, and allowlisted canonical HTTPS delivery.",
            "",
            "## Limitations and production blockers",
            "",
        ]
    )
    lines.extend(f"- {limitation}" for limitation in report["limitations"])
    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 scripts/simulate_use_cases.py \\",
            "  --generated-at 2026-09-04T20:00:00Z \\",
            "  --markdown docs/USE_CASE_SIMULATION_REPORT.md \\",
            "  --json reports/use-case-simulation.json",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--generated-at",
        help="RFC 3339 report timestamp; defaults to the current UTC time",
    )
    parser.add_argument(
        "--markdown",
        type=Path,
        default=Path("docs/USE_CASE_SIMULATION_REPORT.md"),
    )
    parser.add_argument(
        "--json",
        type=Path,
        default=Path("reports/use-case-simulation.json"),
    )
    parser.add_argument("--timeout", type=int, default=600)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    generated_at = args.generated_at or datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    try:
        datetime.fromisoformat(generated_at.replace("Z", "+00:00"))
    except ValueError as error:
        raise SystemExit(f"invalid --generated-at value: {error}") from error

    inventory, inventory_error = test_inventory(args.timeout)
    gate_results: list[dict[str, object]] = []
    for gate in gates():
        try:
            process = run_command(gate.command, args.timeout)
            if process.returncode == 0:
                passed, evidence = gate.validator(process.stdout)
                error = None if passed else "gate output failed its semantic assertion"
            else:
                passed = False
                evidence = "command returned a non-zero exit status"
                error_lines = (process.stderr or process.stdout).strip().splitlines()
                error = error_lines[-1] if error_lines else "command failed without output"
        except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError, KeyError) as error_value:
            passed = False
            evidence = "gate could not produce valid evidence"
            error = str(error_value)
        gate_results.append(
            {
                "id": gate.gate_id,
                "title": gate.title,
                "status": "pass" if passed else "fail",
                "command": command_text(gate.command),
                "evidence": evidence,
                "error": error,
            }
        )

    gate_by_id = {gate["id"]: gate for gate in gate_results}
    scenario_results: list[dict[str, object]] = []
    for scenario in scenarios():
        missing_tests = [
            required
            for required in scenario.required_tests
            if not any(test.endswith(required) for test in inventory)
        ]
        passed = (
            inventory_error is None
            and not missing_tests
            and all(gate_by_id[gate_id]["status"] == "pass" for gate_id in scenario.gate_ids)
        )
        scenario_results.append(
            {
                "id": scenario.scenario_id,
                "title": scenario.title,
                "status": "pass" if passed else "fail",
                "protocol_path": scenario.protocol_path,
                "positive_outcome": scenario.positive_outcome,
                "negative_invariant": scenario.negative_invariant,
                "gates": list(scenario.gate_ids),
                "required_tests": list(scenario.required_tests),
                "missing_tests": missing_tests,
                "trace": [
                    {
                        "sequence": sequence,
                        "step": step.strip(),
                        "status": "pass" if passed else "fail",
                    }
                    for sequence, step in enumerate(
                        scenario.protocol_path.split("→"), start=1
                    )
                ],
            }
        )

    passed_gates = sum(gate["status"] == "pass" for gate in gate_results)
    passed_scenarios = sum(scenario["status"] == "pass" for scenario in scenario_results)
    report: dict[str, object] = {
        "protocol_version": "XLMP/1",
        "report_version": "xlemma-use-case-simulation-v2",
        "generated_at": generated_at,
        "summary": {
            "scenario_count": len(scenario_results),
            "passed_scenarios": passed_scenarios,
            "failed_scenarios": len(scenario_results) - passed_scenarios,
            "gate_count": len(gate_results),
            "passed_gates": passed_gates,
            "failed_gates": len(gate_results) - passed_gates,
            "rust_test_count": len(inventory),
        },
        "gates": gate_results,
        "scenarios": scenario_results,
        "limitations": [
            "The Lean gate is an author-operated pseudo self-test using the checker bundled with the same pinned Lean distribution; independent checker replay, nanoda replay, and a hostile clean-room corpus are not claimed.",
            "Formal consensus simulation consumes structurally valid content-derived observations; production ingress must additionally authenticate node signatures, credentials, committee assignments, and non-revocation proofs.",
            "ASTRA/model calls, live x402/stablecoin settlement, chains, remote storage providers, credential issuers, and randomness beacons remain deterministic reference adapters or fixtures rather than production external services.",
            "The x402 replay guard and artifact store are local single-process reference implementations; production deployments require durable distributed idempotency, reconciliation, replication, credentialed signers, and monitored recovery.",
            "No independent implementation, clean-room bundle reproduction, cryptographic audit, smart-contract audit, sandbox audit, economic audit, or legal review is claimed.",
            "Passing scenarios demonstrate internal consistency of the checked snapshot, not theorem novelty, informal-statement alignment, commercial value, or production safety.",
        ],
    }

    for output_path, content in [
        (args.json, json.dumps(report, indent=2) + "\n"),
        (args.markdown, render_markdown(report)),
    ]:
        resolved = output_path if output_path.is_absolute() else ROOT / output_path
        resolved.parent.mkdir(parents=True, exist_ok=True)
        resolved.write_text(content)

    print(
        f"use-case simulation: {passed_scenarios}/{len(scenario_results)} scenarios, "
        f"{passed_gates}/{len(gate_results)} gates, {len(inventory)} Rust tests"
    )
    if inventory_error:
        print(inventory_error, file=sys.stderr)
    return 0 if passed_scenarios == len(scenario_results) and passed_gates == len(gate_results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
