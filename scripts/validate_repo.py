#!/usr/bin/env python3
"""Structural and protocol-invariant validation for the xLemma repository."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "schemas"
EXAMPLE = ROOT / "examples" / "no-arbitrage"

REQUIRED_FILES = [
    "README.md",
    "MANIFEST.sha256",
    ".editorconfig",
    "rustfmt.toml",
    "docs/FULL_DESIGN.md",
    "docs/TRACEABILITY_MATRIX.md",
    "docs/THREAT_MODEL.md",
    "docs/LEGAL_BOUNDARIES.md",
    "docs/ASTRA_PROMPTS.md",
    "docs/LEAN_LATEX_GUIDE.md",
    "docs/SOURCES.md",
    "docs/ARCHITECTURE_DIAGRAMS.md",
    "docs/RESEARCHER_USER_JOURNEYS.md",
    "docs/GOVERNANCE_CONSTITUTION.md",
    "docs/DATA_AND_TELEMETRY.md",
    "docs/DEPLOYMENT_ARCHITECTURE.md",
    "docs/OPERATOR_RUNBOOK.md",
    "docs/TESTING_STRATEGY.md",
    "docs/PRODUCTION_CHECKLIST.md",
    "docs/PRIOR_ART_AND_DIFFERENTIATION.md",
    "docs/VALIDATION_REPORT.md",
    "spec/000-overview.md",
    "spec/001-identifiers.md",
    "spec/002-proof-rights-capsule.md",
    "spec/003-poir-consensus.md",
    "spec/004-researcher-credit.md",
    "spec/005-compute-curve.md",
    "spec/006-revenue-and-dividends.md",
    "spec/007-astra-lean.md",
    "spec/008-x402-transport.md",
    "spec/009-node-roles.md",
    "spec/010-novelty-significance.md",
    "spec/011-tokenization.md",
    "spec/012-bounties-and-support.md",
    "spec/013-governance-disputes.md",
    "spec/014-api-protocol.md",
    "spec/015-storage-availability.md",
    "spec/016-privacy.md",
    "spec/017-deployment-operations.md",
    "openapi/openapi.yaml",
    "contracts/src/ResearchVault.sol",
    "contracts/src/ResearcherCredit.sol",
    "contracts/src/ProofRegistry.sol",
    "contracts/src/PoIRCertificateRegistry.sol",
    "contracts/src/NodeBondRegistry.sol",
    "contracts/src/BountyEscrow.sol",
    "contracts/src/RevenueRouter.sol",
    "contracts/src/LemmaCapsule1155.sol",
    "lean/XLemma.lean",
    "latex/xlemma.sty",
    "crates/xlemma-crypto/src/lib.rs",
    "crates/xlemma-node/src/lib.rs",
]

EXAMPLE_SCHEMA_MAP = {
    "theory.json": "theory.schema.json",
    "claim.json": "claim.schema.json",
    "proof.json": "proof.schema.json",
    "artifact.json": "artifact.schema.json",
    "contribution.json": "contribution.schema.json",
    "rights.json": "rights.schema.json",
    "policy.json": "formal-consensus-policy.schema.json",
    "researcher.json": "researcher-node.schema.json",
    "lemma-capsule.json": "lemma-capsule.schema.json",
    "x402-extension.json": "x402-extension.schema.json",
}


def fail(message: str) -> None:
    raise AssertionError(message)


def load_json(path: Path):
    try:
        return json.loads(path.read_text())
    except Exception as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def validate_required_files() -> None:
    missing = [name for name in REQUIRED_FILES if not (ROOT / name).is_file()]
    if missing:
        fail(f"missing required repository files: {missing}")


def validate_json_syntax_and_schemas() -> None:
    schema_objects = {}
    for path in sorted(SCHEMAS.glob("*.json")):
        schema_objects[path.name] = load_json(path)

    if len(schema_objects) < 20:
        fail(f"expected at least 20 protocol schemas, found {len(schema_objects)}")

    try:
        import jsonschema
        from referencing import Registry, Resource
    except ImportError:
        print("warning: jsonschema/referencing unavailable; schema instance validation skipped")
        return

    for name, schema in schema_objects.items():
        jsonschema.Draft202012Validator.check_schema(schema)

    registry = Registry()
    for schema in schema_objects.values():
        registry = registry.with_resource(schema["$id"], Resource.from_contents(schema))

    # Relative refs such as common.schema.json are resolved against each schema's $id.
    for example_name, schema_name in EXAMPLE_SCHEMA_MAP.items():
        instance = load_json(EXAMPLE / example_name)
        validator = jsonschema.Draft202012Validator(
            schema_objects[schema_name], registry=registry
        )
        errors = sorted(validator.iter_errors(instance), key=lambda err: list(err.path))
        if errors:
            joined = "\n".join(f"  {list(e.path)}: {e.message}" for e in errors)
            fail(f"{example_name} failed {schema_name}:\n{joined}")

    observation_schema = schema_objects["observation.schema.json"]
    observation_validator = jsonschema.Draft202012Validator(
        observation_schema, registry=registry
    )
    for index, observation in enumerate(load_json(EXAMPLE / "observations.json")):
        errors = list(observation_validator.iter_errors(observation))
        if errors:
            fail(f"observations.json[{index}] invalid: {errors[0].message}")

    offer_schema = schema_objects["compute-offer.schema.json"]
    offer_validator = jsonschema.Draft202012Validator(offer_schema, registry=registry)
    for index, offer in enumerate(load_json(EXAMPLE / "compute-offers.json")):
        errors = list(offer_validator.iter_errors(offer))
        if errors:
            fail(f"compute-offers.json[{index}] invalid: {errors[0].message}")


def validate_toml() -> None:
    for path in [ROOT / "Cargo.toml", ROOT / "config/default.toml", *sorted((ROOT / "config").glob("*-node.toml"))]:
        with path.open("rb") as handle:
            tomllib.load(handle)

    config = tomllib.loads((ROOT / "config/default.toml").read_text())
    revenue = config["revenue"]
    total = sum(
        revenue[key]
        for key in [
            "creator_pool_bps",
            "upstream_dependency_pool_bps",
            "reverification_security_pool_bps",
            "open_research_pool_bps",
            "dispute_insurance_pool_bps",
            "protocol_operations_bps",
        ]
    )
    if total != 10_000:
        fail(f"default revenue waterfall totals {total}, expected 10,000")
    if config["committee"]["stake_weighted_voting"]:
        fail("stake-weighted formal voting must remain disabled")
    if config["astra"]["self_certification_allowed"]:
        fail("ASTRA self-certification must remain disabled")
    if config["x402"]["facilitator_participates_in_research_consensus"]:
        fail("payment facilitator must remain outside research consensus")


def validate_yaml() -> None:
    try:
        import yaml
    except ImportError:
        print("warning: PyYAML unavailable; YAML parse validation skipped")
        return

    yaml_paths = [
        ROOT / "openapi/openapi.yaml",
        ROOT / "deploy/docker-compose.yml",
        ROOT / ".github/workflows/ci.yml",
    ]
    parsed = {}
    for path in yaml_paths:
        obj = yaml.safe_load(path.read_text())
        if not isinstance(obj, dict):
            fail(f"YAML root is not a mapping: {path.relative_to(ROOT)}")
        parsed[path] = obj

    openapi = parsed[ROOT / "openapi/openapi.yaml"]
    if openapi.get("openapi") != "3.1.0":
        fail("OpenAPI version must be 3.1.0")
    if not isinstance(openapi.get("paths"), dict) or not isinstance(
        openapi.get("components"), dict
    ):
        fail("OpenAPI must define paths and components mappings")

    schemas = openapi["components"].get("schemas")
    if not isinstance(schemas, dict):
        fail("OpenAPI components.schemas must be a mapping")
    if "PaymentOfferRequest" not in schemas:
        fail("OpenAPI is missing PaymentOfferRequest")
    if "/v1/verification-jobs/{jobId}/payment-required" not in openapi["paths"]:
        fail("OpenAPI is missing the verification payment-required endpoint")

    http_methods = {"get", "post", "put", "patch", "delete", "head", "options", "trace"}
    operation_ids = set()
    for route, path_item in openapi["paths"].items():
        if not isinstance(path_item, dict):
            fail(f"OpenAPI path item is not a mapping: {route}")
        for method, operation in path_item.items():
            if method.lower() not in http_methods:
                continue
            if not isinstance(operation, dict):
                fail(f"OpenAPI operation is not a mapping: {method.upper()} {route}")
            operation_id = operation.get("operationId")
            if not isinstance(operation_id, str) or not operation_id:
                fail(f"OpenAPI operation lacks operationId: {method.upper()} {route}")
            if operation_id in operation_ids:
                fail(f"duplicate OpenAPI operationId: {operation_id}")
            operation_ids.add(operation_id)
            if not isinstance(operation.get("responses"), dict) or not operation["responses"]:
                fail(f"OpenAPI operation lacks responses: {method.upper()} {route}")

    openapi_path = ROOT / "openapi/openapi.yaml"

    def resolve_pointer(document: object, pointer: str, original_ref: str) -> object:
        if pointer in {"", "#"}:
            return document
        if not pointer.startswith("#/"):
            fail(f"unsupported OpenAPI reference fragment: {original_ref}")
        cursor = document
        for raw_segment in pointer[2:].split("/"):
            segment = raw_segment.replace("~1", "/").replace("~0", "~")
            if not isinstance(cursor, dict) or segment not in cursor:
                fail(f"unresolved OpenAPI reference: {original_ref}")
            cursor = cursor[segment]
        return cursor

    def resolve_ref(ref: str) -> object:
        if ref.startswith("#"):
            return resolve_pointer(openapi, ref, ref)

        file_part, separator, fragment = ref.partition("#")
        target = (openapi_path.parent / file_part).resolve()
        try:
            target.relative_to(ROOT.resolve())
        except ValueError:
            fail(f"OpenAPI reference escapes the repository: {ref}")
        if not target.is_file():
            fail(f"OpenAPI reference target is missing: {ref}")
        if target.suffix.lower() != ".json":
            fail(f"unsupported external OpenAPI reference type: {ref}")
        document = load_json(target)
        pointer = f"#{fragment}" if separator else ""
        return resolve_pointer(document, pointer, ref)

    def walk(value: object) -> None:
        if isinstance(value, dict):
            ref = value.get("$ref")
            if ref is not None:
                if not isinstance(ref, str):
                    fail("OpenAPI $ref value must be a string")
                resolve_ref(ref)
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)

    walk(openapi)


def validate_example_invariants() -> None:
    contribution = load_json(EXAMPLE / "contribution.json")
    total_shares = sum(item["share_bps"] for item in contribution["contributors"])
    if total_shares != 10_000:
        fail(f"example contributor shares total {total_shares}")

    capsule = load_json(EXAMPLE / "lemma-capsule.json")
    waterfall = capsule["revenue_route"]["waterfall"]
    if sum(waterfall.values()) != 10_000:
        fail("example revenue waterfall does not total 10,000")

    observations = load_json(EXAMPLE / "observations.json")
    if len({o["operator_cluster_id"] for o in observations}) != len(observations):
        fail("example checker observations are not operator-independent")
    if {o["verdict"] for o in observations} != {"pass"}:
        fail("example happy-path observations must all pass")
    for root_name in ["artifact_root", "environment_root", "dependency_root", "axiom_set_root"]:
        if len({o[root_name] for o in observations}) != 1:
            fail(f"example observations disagree on {root_name}")

    policy = load_json(EXAMPLE / "policy.json")
    if len({o["infrastructure_provider"] for o in observations}) < policy["minimum_infrastructure_providers"]:
        fail("example observations lack required provider diversity")
    if len({o["region"] for o in observations}) < policy["minimum_regions"]:
        fail("example observations lack required regional diversity")
    if policy["required_family_counts"].get("lean_kernel") != 2:
        fail("example policy must require two Lean-kernel observations")
    if policy["required_family_counts"].get("nanoda") != 1:
        fail("example policy must require one independent nanoda observation")


def validate_documented_invariants() -> None:
    combined = "\n".join(
        (ROOT / path).read_text()
        for path in [
            "README.md",
            "docs/FULL_DESIGN.md",
            "spec/000-overview.md",
            "docs/THREAT_MODEL.md",
        ]
    ).lower()
    required_phrases = [
        "token-weighted",
        "divergent",
        "quarantined",
        "fully backed",
        "independent checker",
        "payment receipts",
        "append-only",
        "compute-savings",
        "latex",
        "astra",
    ]
    missing = [phrase for phrase in required_phrases if phrase not in combined]
    if missing:
        fail(f"core design phrases missing: {missing}")



def validate_source_tree() -> None:
    cargo_root = tomllib.loads((ROOT / "Cargo.toml").read_text())
    members = cargo_root.get("workspace", {}).get("members", [])
    if len(members) < 12:
        fail(f"expected at least 12 Rust workspace members, found {len(members)}")
    for member in members:
        manifest = ROOT / member / "Cargo.toml"
        source = ROOT / member / "src"
        if not manifest.is_file() or not source.is_dir():
            fail(f"incomplete Rust workspace member: {member}")
        with manifest.open("rb") as handle:
            tomllib.load(handle)

    solidity = sorted((ROOT / "contracts/src").glob("*.sol"))
    if len(solidity) < 9:
        fail(f"expected at least 9 Solidity reference contracts, found {len(solidity)}")

    specs = sorted((ROOT / "spec").glob("[0-9][0-9][0-9]-*.md"))
    if len(specs) < 18:
        fail(f"expected at least 18 numbered specifications, found {len(specs)}")

    full_design = (ROOT / "docs/FULL_DESIGN.md").read_text()
    if len(full_design.splitlines()) < 1000:
        fail("integrated design is unexpectedly incomplete")

    if "example.invalid" in (ROOT / "Cargo.toml").read_text():
        fail("placeholder repository URL remains in Cargo metadata")

    openapi = (ROOT / "openapi/openapi.yaml").read_text()
    required_paths = [
        "/v1/claims/{claimId}/formalize",
        "/v1/claims/{claimId}/prove",
        "/v1/proofs/{proofId}/verify",
        "/v1/verification-jobs/{jobId}/evaluate",
        "/v1/verification-jobs/{jobId}/payment-required",
        "/v1/compute/quote",
    ]
    missing_paths = [path for path in required_paths if path not in openapi]
    if missing_paths:
        fail(f"OpenAPI is missing required paths: {missing_paths}")

    contract_text = "\n".join(path.read_text() for path in solidity)
    contract_invariants = [
        "isFinalFor",
        "processedRevenueEvents",
        "operatorClusterId",
        "compoundRevenue",
        "RestrictedTransfer",
        "CapsuleIsNonTransferable",
    ]
    missing_contract_invariants = [
        name for name in contract_invariants if name not in contract_text
    ]
    if missing_contract_invariants:
        fail(f"contract source is missing expected safeguards: {missing_contract_invariants}")

    ci = (ROOT / ".github/workflows/ci.yml").read_text()
    required_ci_controls = [
        "leanchecker: true",
        "nanoda: true",
        "nanoda-allow-sorry: false",
        "axiom-audit: true",
        "forge build --sizes",
        "forge test -vvv",
        "--no-git",
    ]
    missing_ci_controls = [control for control in required_ci_controls if control not in ci]
    if missing_ci_controls:
        fail(f"CI is missing required native assurance controls: {missing_ci_controls}")

def validate_release_manifest() -> None:
    manifest_path = ROOT / "MANIFEST.sha256"
    excluded_parts = {".git", "target", ".lake", "out", "cache", "__pycache__"}
    expected_files = sorted(
        path
        for path in ROOT.rglob("*")
        if path.is_file()
        and path != manifest_path
        and not any(
            part in excluded_parts for part in path.relative_to(ROOT).parts
        )
    )

    entries: dict[str, str] = {}
    for line_number, line in enumerate(manifest_path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            digest, relative = line.split("  ", 1)
        except ValueError:
            fail(f"malformed MANIFEST.sha256 line {line_number}")
        if not re.fullmatch(r"[0-9a-f]{64}", digest):
            fail(f"invalid SHA-256 digest on manifest line {line_number}")
        if relative in entries:
            fail(f"duplicate manifest entry: {relative}")
        entries[relative] = digest

    expected_names = {path.relative_to(ROOT).as_posix() for path in expected_files}
    if set(entries) != expected_names:
        missing = sorted(expected_names - set(entries))
        extra = sorted(set(entries) - expected_names)
        fail(f"release manifest file-set mismatch; missing={missing}, extra={extra}")

    for path in expected_files:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        relative = path.relative_to(ROOT).as_posix()
        if entries[relative] != digest:
            fail(f"release manifest digest mismatch: {relative}")


def validate_source_placeholders() -> None:
    # Prevent accidental hardcoded real secrets or private keys in the archive.
    secret_patterns = [
        re.compile(r"sk-[A-Za-z0-9_-]{20,}"),
        re.compile(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    ]
    for path in ROOT.rglob("*"):
        if not path.is_file() or path.suffix in {".zip", ".png", ".jpg", ".pdf"}:
            continue
        text = path.read_text(errors="ignore")
        for pattern in secret_patterns:
            if pattern.search(text):
                fail(f"possible secret in {path.relative_to(ROOT)}")


def main() -> int:
    checks = [
        validate_required_files,
        validate_json_syntax_and_schemas,
        validate_toml,
        validate_yaml,
        validate_example_invariants,
        validate_documented_invariants,
        validate_source_tree,
        validate_release_manifest,
        validate_source_placeholders,
    ]
    for check in checks:
        check()
        print(f"ok: {check.__name__}")
    print("xLemma repository structural validation passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as exc:
        print(f"validation failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
