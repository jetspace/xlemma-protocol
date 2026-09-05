#!/usr/bin/env python3
"""Prepare unsigned escrow transactions and independently observe confirmed EVM receipts.

Never signs a wallet transaction or submits a discovery command. Observers must
independently operate this tool against their own RPC and then sign the output.
"""
import argparse
import datetime
import json
import os
import re
import subprocess
import tempfile
import urllib.parse
import urllib.request

CATEGORIES = ["foundational_research", "discovery", "formalization", "proof_improvement",
              "replication", "research_tools", "negative_result"]
ITEM_ABI = "(uint8,bool,address,uint256,bytes32,bytes32,bytes32,bytes32,bytes32)[]"
FUNDED = "Funded(bytes32,uint8,address,uint256,bytes32,uint256)"
SETTLED = "Settled(bytes32,bytes32,bytes32,uint256)"
EXPIRED = "Expired(bytes32)"
SAFE = 9_007_199_254_740_991


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(value):
    result = value.rsplit(":", 1)[-1].removeprefix("0x")
    require(bool(re.fullmatch(r"[0-9a-f]{64}", result)), "invalid protocol digest")
    return "0x" + result


def run(binary, *args):
    result = subprocess.run([binary, *map(str, args)], capture_output=True, text=True, timeout=30)
    # Do not expose RPC credentials or private subprocess diagnostics.
    require(result.returncode == 0, "local ABI or identity tool failed")
    return result.stdout.strip()


def identity(cli, value):
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json") as file:
        json.dump(value, file)
        file.flush()
        return run(cli, "derive-id", "receipt", file.name)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError("RPC redirects are disabled")


class Rpc:
    def __init__(self, url):
        parsed = urllib.parse.urlsplit(url)
        require(parsed.scheme == "https" or
                (parsed.scheme == "http" and parsed.hostname in ("localhost", "127.0.0.1", "::1")),
                "RPC must use HTTPS or loopback HTTP")
        self.url = url
        self.opener = urllib.request.build_opener(NoRedirect)

    def __call__(self, method, params):
        body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
        request = urllib.request.Request(self.url, body, {"Content-Type": "application/json"})
        with self.opener.open(request, timeout=15) as response:
            raw = response.read(2 * 1024 * 1024 + 1)
        require(len(raw) <= 2 * 1024 * 1024, "RPC response exceeds bound")
        value = json.loads(raw)
        require(value.get("id") == 1 and "error" not in value and "result" in value, "RPC request failed")
        return value["result"]


def item_tuple(item):
    address = item["recipient"]
    require(bool(re.fullmatch(r"0x[0-9a-fA-F]{40}", address)), "invalid recipient")
    amount = item["amount_units"]
    require(isinstance(amount, int) and 0 < amount <= SAFE, "invalid award amount")
    values = [CATEGORIES.index(item["category"]), str(item["completed_review"]).lower(), address, amount]
    values += [digest(item[key]) for key in ("certificate_digest", "claim_digest", "artifact_digest", "policy_digest", "work_root")]
    return "(" + ",".join(map(str, values)) + ")"


def transaction(args, trust, policy, roots):
    economics = policy["economics"]
    if args.action == "publish-evidence":
        evidence = json.load(open(args.evidence,encoding="utf-8"))
        clusters = sorted({digest(c) for c in evidence["producer_clusters"]})
        data = run(args.cast,"calldata","attest(bytes32,bytes32,bytes32,bytes32,bytes32,bytes32[],uint64)",
                   *[digest(evidence[k]) for k in ("certificate","claim","artifact","policy","observation_root")],
                   "["+",".join(clusters)+"]",evidence["challenge_ends_at"])
        require(bool(re.fullmatch(r"0x[0-9a-fA-F]{40}",args.registry or "")),"registry address required")
        return {"chainId":trust["chain_id"],"to":args.registry,"value":"0x0","data":data,"broadcast":False}
    if args.action == "create-round":
        solver = [economics["budgets"].get(c, {}).get("solver_units", 0) for c in CATEGORIES]
        review = [sum(economics["budgets"].get(c, {}).get(k, 0) for k in ("verification_units", "appeal_units")) for c in CATEGORIES]
        data = run(args.cast, "calldata", "createRound(bytes32,bytes32,uint64,uint64,uint64,uint64,uint16,uint256[7],uint256[7])",
                   digest(roots["round_id"]), digest(roots["policy_id"]), economics["opens_at"],
                   economics["submissions_close_at"], economics["appeals_close_at"], policy["settlement_expires_at"],
                   policy["minimum_foundational_bps"], json.dumps(solver), json.dumps(review))
    else:
        plan = json.load(open(args.plan, encoding="utf-8"))
        require(plan["round_id"] == roots["round_id"] and plan["chain_id"] == trust["chain_id"]
                and plan["escrow_address"] == trust["escrow_address"] and plan["usdc_asset"] == trust["usdc_asset"],
                "plan settlement domain mismatch")
        require(0 < len(plan["items"]) <= 256 and sum(i["amount_units"] for i in plan["items"]) == plan["total_units"], "invalid plan total")
        items = "[" + ",".join(item_tuple(i) for i in plan["items"]) + "]"
        data = run(args.cast, "calldata", f"proposePlan(bytes32,bytes32,{ITEM_ABI})", digest(roots["round_id"]), digest(plan["plan_id"]), items)
    return {"chainId": trust["chain_id"], "to": trust["escrow_address"], "value": "0x0", "data": data,
            "round_id": roots["round_id"], "policy_id": roots["policy_id"], "broadcast": False}


def observe(args, trust, policy, roots, rpc):
    require(args.confirmations >= 1, "at least one confirmation is required")
    require(int(rpc("eth_chainId", []), 16) == trust["chain_id"], "wrong chain")
    tx = digest(args.transaction)
    receipt = rpc("eth_getTransactionReceipt", [tx])
    require(receipt and receipt["status"] == "0x1" and receipt["transactionHash"].lower() == tx,
            "transaction failed, missing or mismatched")
    height = int(receipt["blockNumber"], 16)
    head = int(rpc("eth_blockNumber", []), 16)
    require(head - height + 1 >= args.confirmations, "insufficient confirmations")
    block = rpc("eth_getBlockByNumber", [receipt["blockNumber"], False])
    require(block and block["hash"] == receipt["blockHash"], "receipt belongs to a noncanonical block")
    escrow = trust["escrow_address"]
    # Pin the qualified implementation rather than trusting an arbitrary lookalike event emitter.
    code = rpc("eth_getCode", [escrow, receipt["blockNumber"]])
    require(code != "0x" and run(args.cast, "keccak", code) == digest(args.code_hash), "escrow implementation mismatch")
    token = rpc("eth_call", [{"to": escrow, "data": run(args.cast, "calldata", "asset()")}, receipt["blockNumber"]])
    expected_asset = f"eip155:{trust['chain_id']}/erc20:0x{token[-40:].lower()}"
    require(trust["usdc_asset"].lower() == expected_asset.lower(), "escrow asset differs from pinned USDC deployment")
    state = rpc("eth_call", [{"to": escrow, "data": run(args.cast, "calldata", "round(bytes32)", digest(roots["round_id"]))}, receipt["blockNumber"]])
    require(state[:66] == digest(roots["policy_id"]) and len(state) == 2 + 38*64, "onchain round policy mismatch")
    state_words = [int(state[2+i:2+i+64],16) for i in range(0,38*64,64)]
    economics = policy["economics"]
    require(state_words[1:5] == [economics["opens_at"],economics["submissions_close_at"],economics["appeals_close_at"],policy["settlement_expires_at"]], "onchain deadlines differ from signed policy")
    require(state_words[10:17] == [economics["budgets"].get(c,{}).get("solver_units",0) for c in CATEGORIES], "onchain solver caps differ")
    require(state_words[17:24] == [sum(economics["budgets"].get(c,{}).get(k,0) for k in ("verification_units","appeal_units")) for c in CATEGORIES], "onchain review reserves differ")
    logs = [log for log in receipt["logs"] if int(log["logIndex"], 16) == args.log_index
            and log["address"].lower() == escrow.lower() and not log.get("removed", False)]
    require(len(logs) == 1, "missing or ambiguous escrow log")
    log = logs[0]
    require(log.get("transactionHash", tx).lower() == tx and log.get("blockHash", block["hash"]) == block["hash"], "log binding mismatch")
    topics = log["topics"]
    event = {"observe-funding":FUNDED,"observe-settlement":SETTLED,"observe-expiry":EXPIRED}[args.action]
    require(len(topics) == {FUNDED:4,SETTLED:3,EXPIRED:2}[event] and topics[0] == run(args.cast, "keccak", event)
            and topics[1] == digest(roots["round_id"]), "wrong event or round")
    if event == EXPIRED:
        require(state_words[6] == 0 and state_words[7] == 1 and log["data"] == "0x", "round did not expire unpaid")
        return {"kind":"observe_expiry","round_id":roots["round_id"],"transaction_hash":tx,"block_hash":block["hash"]}
    data = log["data"].removeprefix("0x")
    require(bool(re.fullmatch(r"[0-9a-fA-F]+", data)) and len(data) == (192 if event == FUNDED else 128), "invalid event data")
    words = [data[i:i+64] for i in range(0, len(data), 64)]
    if event == SETTLED:
        plan = json.load(open(args.plan, encoding="utf-8"))
        require(plan["round_id"] == roots["round_id"] and topics[2] == digest(plan["plan_id"])
                and int(words[1], 16) == plan["total_units"], "settled plan or total differs")
        items = "[" + ",".join(item_tuple(i) for i in plan["items"]) + "]"
        encoded = run(args.cast, "abi-encode", f"f(uint256,address,bytes32,bytes32,bytes32,{ITEM_ABI})",
                      trust["chain_id"], escrow, digest(roots["round_id"]), digest(plan["plan_id"]), digest(roots["policy_id"]), items)
        require("0x" + words[0] == run(args.cast, "keccak", encoded), "settlement item commitment mismatch")
        return {"kind": "observe_settlement", "round_id": roots["round_id"], "plan_id": plan["plan_id"], "transaction_hash": tx, "block_hash": block["hash"]}
    category = int(topics[2], 16)
    amount = int(words[0], 16)
    require(category < len(CATEGORIES) and 0 < amount <= SAFE, "invalid funding category or amount")
    # One chain/log has one identity even if RPCs observe different confirmation depths.
    settlement_id = identity(args.cli, ["evm-discovery-funding-v1", trust["chain_id"], escrow.lower(), tx, args.log_index])
    evidence_id = identity(args.cli, ["evm-receipt-evidence-v1", trust["chain_id"],
        escrow.lower(), tx, block["hash"].lower(), height, args.log_index,
        [topic.lower() for topic in topics], log["data"].lower(), digest(args.code_hash)])
    purpose = {"formalization": "formal_library", "negative_result": "negative_result", "research_tools": "benchmark_suite"}.get(CATEGORIES[category], "foundational_research")
    timestamp = datetime.datetime.fromtimestamp(int(block["timestamp"],16),datetime.timezone.utc).isoformat().replace("+00:00","Z")
    return {"kind": "observe_funding", "round_id": roots["round_id"], "category": CATEGORIES[category],
            "donor_cluster": args.donor_cluster, "administrator_cluster": args.administrator_cluster,
            "receipt": {"funding_receipt_id": "xlfunding:blake3:" + "0"*64, "rail": "commons", "purpose": purpose,
                        "payer": "0x" + topics[3][-40:], "beneficiary_researcher_id": None, "destination_vault": escrow,
                        "settled_amount": {"units": amount, "asset": trust["usdc_asset"], "decimals": 6},
                        "settlement_receipt_id": settlement_id, "external_value_evidence_root": evidence_id,
                        "policy_id": roots["policy_id"], "related_party": False, "settled_at": timestamp,
                        "signature": "evm-receipt:" + tx + ":" + str(args.log_index)}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["create-round", "propose-plan", "publish-evidence", "observe-funding", "observe-settlement", "observe-expiry"])
    parser.add_argument("--trust", required=True)
    parser.add_argument("--policy", required=True)
    parser.add_argument("--plan")
    parser.add_argument("--evidence")
    parser.add_argument("--registry")
    parser.add_argument("--cli", default="target/debug/xlemma-cli")
    parser.add_argument("--cast", default="cast")
    parser.add_argument("--transaction")
    parser.add_argument("--log-index", type=int)
    parser.add_argument("--confirmations", type=int, default=12)
    parser.add_argument("--code-hash")
    parser.add_argument("--donor-cluster")
    parser.add_argument("--administrator-cluster")
    args = parser.parse_args()
    trust = json.load(open(args.trust, encoding="utf-8"))
    policy = json.load(open(args.policy, encoding="utf-8"))
    roots = json.loads(run(args.cli,"discovery-prepare",args.trust,args.policy))
    if args.action.startswith("observe-"):
        require(args.transaction and args.log_index is not None and args.code_hash, "receipt observation requires transaction, log index and pinned implementation hash")
        if args.action == "observe-funding":
            require(args.donor_cluster and args.administrator_cluster, "funding requires disclosed control clusters")
        result = observe(args, trust, policy, roots, Rpc(os.environ["XLEMMA_EVM_RPC_URL"]))
    else:
        result = transaction(args, trust, policy, roots)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, TypeError, KeyError, OSError, subprocess.SubprocessError) as error:
        # Only validation reasons are safe to print; transport exceptions can contain URLs.
        message = str(error) if type(error) is ValueError else "discovery chain operation failed"
        raise SystemExit(message) from None
