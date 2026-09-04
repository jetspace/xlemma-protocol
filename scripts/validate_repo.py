#!/usr/bin/env python3
"""Structural and protocol-invariant validation for the xLemma repository."""

from __future__ import annotations

import hashlib
import json
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10 compatibility for local validation.
    import tomli as tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "schemas"
EXAMPLE = ROOT / "examples" / "no-arbitrage"
NODE_EXAMPLE = ROOT / "examples" / "node-network"

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
    "spec/018-xlmp-wire-protocol.md",
    "spec/019-node-network.md",
    "spec/020-identity-credentials.md",
    "spec/021-alignment-rights-and-impact.md",
    "spec/022-researcher-sovereignty.md",
    "openapi/openapi.yaml",
    "contracts/src/ResearchVault.sol",
    "contracts/src/ResearcherCredit.sol",
    "contracts/src/ProofRegistry.sol",
    "contracts/src/ResearchCommitmentRegistry.sol",
    "contracts/src/PoIRCertificateRegistry.sol",
    "contracts/src/NodeBondRegistry.sol",
    "contracts/src/BountyEscrow.sol",
    "contracts/src/RevenueRouter.sol",
    "contracts/src/LemmaCapsule1155.sol",
    "lean/XLemma.lean",
    "latex/xlemma.sty",
    "crates/xlemma-crypto/src/lib.rs",
    "crates/xlemma-node/src/lib.rs",
    "crates/xlemma-node/src/marketplace.rs",
    "crates/xlemma-node/src/credentials.rs",
    "crates/xlemma-core/src/identity.rs",
    "crates/xlemma-core/src/capture.rs",
    "crates/xlemma-core/src/governance.rs",
    "crates/xlemma-core/src/network.rs",
    "crates/xlemma-core/src/sovereignty.rs",
    "crates/xlemma-xlmp/src/lib.rs",
    "schemas/node-network-common.schema.json",
    "schemas/node-service-advertisement.schema.json",
    "schemas/node-reputation.schema.json",
    "schemas/node-bond.schema.json",
    "schemas/node-discovery-request.schema.json",
    "schemas/node-discovery-result.schema.json",
    "schemas/service-order.schema.json",
    "schemas/service-match.schema.json",
    "schemas/committee-sortition-request.schema.json",
    "schemas/committee-selection.schema.json",
    "schemas/eligible-node.schema.json",
    "schemas/identity-common.schema.json",
    "schemas/user-credential.schema.json",
    "schemas/operator-credential.schema.json",
    "schemas/node-credential.schema.json",
    "schemas/credential-status-proof.schema.json",
    "schemas/node-credential-chain.schema.json",
    "schemas/credential-revocation.schema.json",
    "schemas/statement-alignment-receipt.schema.json",
    "schemas/protocol-success-estimates.schema.json",
    "schemas/compute-savings-evidence.schema.json",
    "schemas/impact-pool-authorization.schema.json",
    "schemas/researcher-sovereignty-bundle.schema.json",
    "schemas/researcher-portability-manifest.schema.json",
    "schemas/researcher-residual-right.schema.json",
    "schemas/economic-constitution.schema.json",
    "schemas/research-graph-edge.schema.json",
    "schemas/verification-profile.schema.json",
    "schemas/research-compute-cooperative.schema.json",
    "schemas/capture-resistance-dashboard.schema.json",
    "schemas/node-work-receipt.schema.json",
    "schemas/node-exposure-limit.schema.json",
    "schemas/objective-misconduct-record.schema.json",
    "schemas/upstream-allocation.schema.json",
    "schemas/knowledge-productivity-observation.schema.json",
    "schemas/constitutional-commitment.schema.json",
    "schemas/fork-exit-plan.schema.json",
    "schemas/governance-proposal.schema.json",
    "schemas/credential-issuer-policy.schema.json",
    "schemas/independent-credential-attestation.schema.json",
    "schemas/funding-receipt.schema.json",
    "schemas/compute-procurement-instrument.schema.json",
    "schemas/compute-concentration-policy.schema.json",
    "schemas/reproduction-observation.schema.json",
    "schemas/research-verification-certificate.schema.json",
    "schemas/economic-compliance-certificate.schema.json",
    "examples/node-network/advertisement.json",
    "examples/node-network/reputation.json",
    "examples/node-network/bond.json",
    "examples/node-network/xlmp-node-advertise.json",
    "examples/node-network/eligible-nodes.json",
    "examples/node-network/sortition-request.json",
    "examples/node-network/committee-selection.json",
    "examples/node-network/user-credential.json",
    "examples/node-network/operator-credential.json",
    "examples/node-network/node-credential.json",
    "examples/node-network/credential-status.json",
    "examples/node-network/credential-chain.json",
    "examples/node-network/credential-revocation.json",
    "examples/no-arbitrage/statement-alignment-receipt.json",
    "examples/no-arbitrage/protocol-success-estimates.json",
    "examples/no-arbitrage/compute-savings-evidence.json",
    "examples/no-arbitrage/impact-pool-authorization.json",
    "examples/no-arbitrage/sovereignty-bundle.json",
    "examples/no-arbitrage/portability-manifest.json",
    "examples/no-arbitrage/residual-right.json",
    "examples/no-arbitrage/economic-constitution.json",
    "examples/no-arbitrage/verification-profile.json",
    "examples/no-arbitrage/evidence-graph-edge.json",
    "examples/node-network/compute-cooperative.json",
    "examples/node-network/capture-dashboard.json",
    "examples/node-network/node-work-receipt.json",
    "examples/node-network/node-exposure-limit.json",
    "examples/node-network/objective-misconduct.json",
    "examples/no-arbitrage/upstream-allocation.json",
    "examples/no-arbitrage/knowledge-productivity.json",
    "examples/no-arbitrage/funding-receipt.json",
    "examples/no-arbitrage/compute-procurement-instrument.json",
    "examples/no-arbitrage/compute-concentration-policy.json",
    "examples/no-arbitrage/computational-verification-profile.json",
    "examples/no-arbitrage/computational-verification-job.json",
    "examples/no-arbitrage/computational-observation-a.json",
    "examples/no-arbitrage/computational-observation-b.json",
    "examples/no-arbitrage/computational-observations.json",
    "examples/no-arbitrage/computational-research-certificate.json",
    "examples/no-arbitrage/economic-compliance-certificate.json",
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
    "xlmp-envelope.json": "xlmp-envelope.schema.json",
    "statement-alignment-receipt.json": "statement-alignment-receipt.schema.json",
    "protocol-success-estimates.json": "protocol-success-estimates.schema.json",
    "compute-savings-evidence.json": "compute-savings-evidence.schema.json",
    "impact-pool-authorization.json": "impact-pool-authorization.schema.json",
    "sovereignty-bundle.json": "researcher-sovereignty-bundle.schema.json",
    "portability-manifest.json": "researcher-portability-manifest.schema.json",
    "residual-right.json": "researcher-residual-right.schema.json",
    "economic-constitution.json": "economic-constitution.schema.json",
    "verification-profile.json": "verification-profile.schema.json",
    "evidence-graph-edge.json": "research-graph-edge.schema.json",
    "upstream-allocation.json": "upstream-allocation.schema.json",
    "knowledge-productivity.json": "knowledge-productivity-observation.schema.json",
    "funding-receipt.json": "funding-receipt.schema.json",
    "compute-procurement-instrument.json": "compute-procurement-instrument.schema.json",
    "compute-concentration-policy.json": "compute-concentration-policy.schema.json",
    "computational-verification-profile.json": "verification-profile.schema.json",
    "computational-verification-job.json": "verification-job.schema.json",
    "computational-observation-a.json": "reproduction-observation.schema.json",
    "computational-observation-b.json": "reproduction-observation.schema.json",
    "computational-research-certificate.json": "research-verification-certificate.schema.json",
    "economic-compliance-certificate.json": "economic-compliance-certificate.schema.json",
}

NODE_EXAMPLE_SCHEMA_MAP = {
    "advertisement.json": "node-service-advertisement.schema.json",
    "reputation.json": "node-reputation.schema.json",
    "bond.json": "node-bond.schema.json",
    "xlmp-node-advertise.json": "xlmp-envelope.schema.json",
    "sortition-request.json": "committee-sortition-request.schema.json",
    "committee-selection.json": "committee-selection.schema.json",
    "user-credential.json": "user-credential.schema.json",
    "operator-credential.json": "operator-credential.schema.json",
    "node-credential.json": "node-credential.schema.json",
    "credential-status.json": "credential-status-proof.schema.json",
    "credential-chain.json": "node-credential-chain.schema.json",
    "credential-revocation.json": "credential-revocation.schema.json",
    "compute-cooperative.json": "research-compute-cooperative.schema.json",
    "capture-dashboard.json": "capture-resistance-dashboard.schema.json",
    "node-work-receipt.json": "node-work-receipt.schema.json",
    "node-exposure-limit.json": "node-exposure-limit.schema.json",
    "objective-misconduct.json": "objective-misconduct-record.schema.json",
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


def validate_no_symlinks() -> None:
    links = [path.relative_to(ROOT) for path in ROOT.rglob("*") if path.is_symlink()]
    if links:
        fail(f"repository must not contain symlinks: {links}")


def validate_json_syntax_and_schemas() -> None:
    schema_objects = {}
    for path in sorted(SCHEMAS.glob("*.json")):
        schema_objects[path.name] = load_json(path)

    if len(schema_objects) < 54:
        fail(f"expected at least 54 protocol schemas, found {len(schema_objects)}")

    try:
        import jsonschema
    except ImportError:
        print("warning: jsonschema unavailable; schema instance validation skipped")
        return

    try:
        from referencing import Registry, Resource
    except ImportError:
        Registry = None
        Resource = None

    for name, schema in schema_objects.items():
        jsonschema.Draft202012Validator.check_schema(schema)

    schema_store = {schema["$id"]: schema for schema in schema_objects.values()}
    registry = None
    if Registry is not None and Resource is not None:
        registry = Registry()
        for schema in schema_objects.values():
            registry = registry.with_resource(
                schema["$id"], Resource.from_contents(schema)
            )

    def schema_validator(schema):
        if registry is not None:
            return jsonschema.Draft202012Validator(schema, registry=registry)
        # jsonschema 4.17 and earlier use RefResolver rather than the separate
        # referencing package. Keep full local validation available there.
        resolver = jsonschema.RefResolver.from_schema(schema, store=schema_store)
        return jsonschema.Draft202012Validator(schema, resolver=resolver)

    # Relative refs such as common.schema.json are resolved against each schema's $id.
    for example_name, schema_name in EXAMPLE_SCHEMA_MAP.items():
        instance = load_json(EXAMPLE / example_name)
        validator = schema_validator(schema_objects[schema_name])
        errors = sorted(validator.iter_errors(instance), key=lambda err: list(err.path))
        if errors:
            joined = "\n".join(f"  {list(e.path)}: {e.message}" for e in errors)
            fail(f"{example_name} failed {schema_name}:\n{joined}")

    for example_name, schema_name in NODE_EXAMPLE_SCHEMA_MAP.items():
        instance = load_json(NODE_EXAMPLE / example_name)
        validator = schema_validator(schema_objects[schema_name])
        errors = sorted(validator.iter_errors(instance), key=lambda err: list(err.path))
        if errors:
            joined = "\n".join(f"  {list(e.path)}: {e.message}" for e in errors)
            fail(f"node-network/{example_name} failed {schema_name}:\n{joined}")

    eligible_validator = schema_validator(schema_objects["eligible-node.schema.json"])
    for index, node in enumerate(load_json(NODE_EXAMPLE / "eligible-nodes.json")):
        errors = list(eligible_validator.iter_errors(node))
        if errors:
            fail(f"eligible-nodes.json[{index}] invalid: {errors[0].message}")

    message_variants = schema_objects["xlmp-envelope.schema.json"]["properties"]["message"]["oneOf"]
    if len(message_variants) != 40:
        fail(f"expected exactly 40 XLMP/1 message variants, found {len(message_variants)}")

    observation_schema = schema_objects["observation.schema.json"]
    observation_validator = schema_validator(observation_schema)
    for index, observation in enumerate(load_json(EXAMPLE / "observations.json")):
        errors = list(observation_validator.iter_errors(observation))
        if errors:
            fail(f"observations.json[{index}] invalid: {errors[0].message}")

    reproduction_schema = schema_objects["reproduction-observation.schema.json"]
    reproduction_validator = schema_validator(reproduction_schema)
    for index, observation in enumerate(load_json(EXAMPLE / "computational-observations.json")):
        errors = list(reproduction_validator.iter_errors(observation))
        if errors:
            fail(f"computational-observations.json[{index}] invalid: {errors[0].message}")

    offer_schema = schema_objects["compute-offer.schema.json"]
    offer_validator = schema_validator(offer_schema)
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
    if config["committee"]["maximum_eligible_nodes"] != 1024:
        fail("default committee eligible-set bound must match XLIP-019")
    if config["committee"]["maximum_committee_slots"] != 32:
        fail("default committee slot bound must match XLIP-019")
    if config["committee"]["maximum_search_states"] != 1_000_000:
        fail("default committee search bound must match XLIP-019")
    if config["committee"]["minimum_credential_tier"] != "v2_verified_operator":
        fail("committee consensus must require at least a V2 operator credential")
    if not config["committee"]["verified_user_enforcement"]:
        fail("committee selection must enforce ultimate verified-user independence")
    if config["identity"]["raw_legal_identity_on_protocol"]:
        fail("raw legal identity must remain off protocol")
    if not config["identity"]["append_only_credentials_and_revocations"]:
        fail("credential and revocation records must remain append-only")
    issuer_policy = config["identity"]["multi_issuer"]
    if issuer_policy["minimum_distinct_issuers"] < 2:
        fail("consensus identity must require multiple independent issuers")
    if issuer_policy["maximum_issuer_attestation_share_bps"] >= 10_000:
        fail("one credential issuer must not control all attestations")
    if issuer_policy["single_company_issuer_allowed"]:
        fail("one xLemma company must not be the sole credential issuer")
    if not issuer_policy["selective_disclosure_required"] or not issuer_policy["public_revocation_roots_required"]:
        fail("multi-issuer credentials require selective disclosure and public revocation roots")
    if not config["node_marketplace"]["append_only_history"]:
        fail("node marketplace history must remain append-only")
    if not config["node_marketplace"]["checked_integer_prices"]:
        fail("node marketplace must use checked integer price arithmetic")
    if config["astra"]["self_certification_allowed"]:
        fail("ASTRA self-certification must remain disabled")
    if config["x402"]["facilitator_participates_in_research_consensus"]:
        fail("payment facilitator must remain outside research consensus")
    if config["revenue"]["upstream_dependency_pool_bps"] != 0:
        fail("Open Commons default must not impose a dependency royalty pool")
    if not config["impact_pool"]["economic_policy_required"]:
        fail("compute impact allocations require a prescriptive economic policy")
    if not config["impact_pool"]["non_recursive"]:
        fail("impact pool allocations must not recursively charge one revenue event")
    if config["impact_pool"]["lower_confidence_multiplier_bps"] != 16_450:
        fail("impact confidence multiplier must use the fixed-point basis-point field")
    if config["protocol_version"] != "XLMP/1":
        fail("the canonical protocol version must be XLMP/1")
    sovereignty = config["sovereignty"]
    if sovereignty["default_economic_mode"] != "commons":
        fail("Commons must remain the default capsule economic mode")
    if sovereignty["downstream_veto_allowed"] or sovereignty["recursive_revenue_charging_allowed"]:
        fail("economic participation cannot create a downstream veto or recursive charge")
    if sovereignty["minimum_artifact_storage_providers"] < 2 or sovereignty["minimum_event_log_providers"] < 2:
        fail("portable recovery requires independent artifact and event-log providers")
    verification_profiles = config["verification_profiles"]
    required_profiles = {"formal", "computational", "statistical", "simulation", "empirical", "hybrid"}
    if set(verification_profiles["enabled"]) != required_profiles:
        fail("all six XLMP verification profiles must remain enabled")
    if verification_profiles["producer_self_certification_allowed"]:
        fail("a research producer cannot count as its own independent reproducer")
    if verification_profiles["mixed_pass_fail_outcome"] != "divergent":
        fail("mixed reproduction evidence must remain divergent")
    compute_market = config["compute_market"]
    if compute_market["canonical_generation_service"] != "research_prover_generation":
        fail("canonical compute service names must remain provider-neutral")
    if compute_market["transferable_derivatives_enabled"]:
        fail("XLMP/1 compute procurement instruments must remain nontransferable")
    if min(compute_market[key] for key in ["minimum_provider_clusters", "minimum_model_families", "minimum_regions"]) < 2:
        fail("compute routes require provider, model, and region diversity")
    capture = config["capture_dashboard"]
    if set(capture["required_layers"]) != {"identity", "compute", "models", "verification", "storage", "settlement", "discovery", "governance"}:
        fail("capture dashboard must publish all eight critical layers")
    if not capture["beneficial_owner_clustering_required"] or not capture["effective_score_uses_weakest_layer"]:
        fail("capture resistance must use beneficial control and the weakest layer")
    node_economics = config["node_economics"]
    if node_economics["pay_for_idle_existence"] or node_economics["honest_divergence_slashable"]:
        fail("nodes earn for work and honest divergence is not slashable")
    if not node_economics["work_evidence_required"] or not node_economics["external_settlement_evidence_required"]:
        fail("node revenue requires work and external settlement evidence")
    governance = config["governance"]
    if set(governance["required_chambers"]) != {"researcher", "operator", "commons"}:
        fail("material governance requires all three constituencies")
    if governance["truth_vote_allowed"] or governance["material_timelock_seconds"] < 604800:
        fail("governance cannot vote on truth and material changes require a seven-day timelock")
    if set(config["funding"]["rails"]) != {"market", "commons", "assurance"}:
        fail("market, commons, and assurance funding rails are all required")
    if config["funding"]["unsettled_value_counts_as_funding"]:
        fail("unsettled value cannot be represented as funding")


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
    if "PaymentOfferRequest" in schemas:
        fail("OpenAPI must not accept caller-authored payment offers")
    if "/xlmp/v1/messages" not in openapi["paths"]:
        fail("OpenAPI is missing the canonical XLMP message ingress")
    if "/v1/verification-jobs/{jobId}/payment-required" in openapi["paths"]:
        fail("reference API must not construct payment requirements from job requests")

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
    if len({o["operator_id"] for o in observations}) != len(observations):
        fail("example checker observations reuse an OperatorID")
    if len({o["verified_user_id"] for o in observations}) != len(observations):
        fail("example checker observations reuse a VerifiedUserID")
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
    if len({o["operator_id"] for o in observations}) < policy["minimum_operators"]:
        fail("example observations lack required OperatorID diversity")
    if len({o["verified_user_id"] for o in observations}) < policy["minimum_verified_users"]:
        fail("example observations lack required verified-participant diversity")
    if policy["required_family_counts"].get("lean_kernel") != 2:
        fail("example policy must require two Lean-kernel observations")
    if policy["required_family_counts"].get("nanoda") != 1:
        fail("example policy must require one independent nanoda observation")

    alignment = load_json(EXAMPLE / "statement-alignment-receipt.json")
    reviewer_clusters = [
        reviewer["operator_cluster_id"] for reviewer in alignment["domain_reviewers"]
    ]
    if len(set(reviewer_clusters)) != len(reviewer_clusters):
        fail("statement-alignment reviewers share an operator-control cluster")

    impact_evidence = load_json(EXAMPLE / "compute-savings-evidence.json")
    impact_authorization = load_json(EXAMPLE / "impact-pool-authorization.json")
    if impact_authorization["compute_savings_evidence_id"] != impact_evidence["evidence_id"]:
        fail("impact authorization does not bind the published evidence")


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
        "multidimensional",
        "service advertisement",
        "verified participant",
        "operatorcredential",
        "non-revocation",
    ]
    missing = [phrase for phrase in required_phrases if phrase not in combined]
    if missing:
        fail(f"core design phrases missing: {missing}")



def validate_source_tree() -> None:
    cargo_root = tomllib.loads((ROOT / "Cargo.toml").read_text())
    members = cargo_root.get("workspace", {}).get("members", [])
    if len(members) < 13:
        fail(f"expected at least 13 Rust workspace members, found {len(members)}")
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
    if len(specs) < 21:
        fail(f"expected at least 21 numbered specifications, found {len(specs)}")

    full_design = (ROOT / "docs/FULL_DESIGN.md").read_text()
    if len(full_design.splitlines()) < 1000:
        fail("integrated design is unexpectedly incomplete")

    if "example.invalid" in (ROOT / "Cargo.toml").read_text():
        fail("placeholder repository URL remains in Cargo metadata")

    openapi = (ROOT / "openapi/openapi.yaml").read_text()
    required_paths = [
        "/xlmp/v1/messages",
        "/v1/claims/{claimId}/formalize",
        "/v1/claims/{claimId}/prove",
        "/v1/proofs/{proofId}/verify",
        "/v1/verification-jobs/{jobId}/evaluate",
        "/v1/compute/quote",
        "/v1/node-advertisements",
        "/v1/node-discovery",
        "/v1/service-orders",
        "/v1/committee-sortitions",
        "/v1/credentials/users",
        "/v1/credentials/operators",
        "/v1/credentials/nodes",
        "/v1/credentials/revocations",
    ]
    missing_paths = [path for path in required_paths if path not in openapi]
    if missing_paths:
        fail(f"OpenAPI is missing required paths: {missing_paths}")

    core_state = (ROOT / "crates/xlemma-core/src/state.rs").read_text()
    if "AstraProver" in core_state:
        fail("core node roles must remain provider-neutral; ASTRA belongs in its adapter")

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
    excluded_parts = {".git", "target", ".lake", "out", "cache", "__pycache__"}
    secret_patterns = [
        re.compile(r"sk-[A-Za-z0-9_-]{20,}"),
        re.compile(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    ]
    for path in ROOT.rglob("*"):
        relative_parts = path.relative_to(ROOT).parts
        if (
            not path.is_file()
            or any(part in excluded_parts for part in relative_parts)
            or path.suffix in {".zip", ".png", ".jpg", ".pdf"}
        ):
            continue
        text = path.read_text(errors="ignore")
        for pattern in secret_patterns:
            if pattern.search(text):
                fail(f"possible secret in {path.relative_to(ROOT)}")


def main() -> int:
    checks = [
        validate_required_files,
        validate_no_symlinks,
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
