#!/usr/bin/env python3
"""Local EVM integration: real funding -> signed Rust round -> registry -> USDC payout.

The research checker is a deterministic test double in the Rust fixture. This
test establishes transport, accounting and settlement integration, not science.
Starts a fresh isolated Anvil; never connects to a configured production chain.
"""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from types import SimpleNamespace

from discovery_chain import CATEGORIES, Rpc, digest, observe, run, transaction

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contracts", type=Path, default=ROOT / "contracts")
    parser.add_argument("--cast", default="cast")
    parser.add_argument("--anvil", default="anvil")
    parser.add_argument("--cli", default=str(ROOT / "target/debug/xlemma-cli"))
    args = parser.parse_args()
    with socket.socket() as socket_file:
        socket_file.bind(("127.0.0.1", 0))
        port = socket_file.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="xlemma-discovery-evm-") as directory:
        directory = Path(directory)
        with open(directory / "anvil.log", "w", encoding="utf-8") as output:
            process = subprocess.Popen([args.anvil, "--host", "127.0.0.1", "--port", str(port), "--timestamp", "1", "--silent"], stdout=output, stderr=output)
        try:
            rpc = Rpc(f"http://127.0.0.1:{port}")
            for attempt in range(100):
                try:
                    accounts = rpc("eth_accounts", [])
                    break
                except OSError:
                    time.sleep(0.05)
            else:
                raise AssertionError("local Anvil did not start")
            assert int(rpc("eth_chainId", []),16) == 31337
            admin, reviewer_a, reviewer_b = accounts[:3]

            def send(data, to=None, sender=admin):
                request = {"from":sender,"data":data,"gas":"0x989680"}
                if to:
                    request["to"] = to
                tx = rpc("eth_sendTransaction",[request])
                receipt = None
                deadline = time.monotonic() + 10
                while receipt is None and time.monotonic() < deadline:
                    receipt = rpc("eth_getTransactionReceipt",[tx])
                    if receipt is None:
                        time.sleep(0.02)
                assert receipt and receipt["status"] == "0x1", receipt
                return receipt

            def call(to, signature, *values, sender=admin):
                return send(run(args.cast,"calldata",signature,*values),to,sender)

            def deploy(name, signature=None, *values):
                artifact = json.loads((args.contracts / "out" / f"{name}.sol" / f"{name}.json").read_text())
                code = artifact["bytecode"]["object"]
                if signature:
                    code += run(args.cast,"abi-encode",signature,*values).removeprefix("0x")
                return send(code)["contractAddress"]

            token = deploy("MockUSDC")
            registry = deploy("DiscoveryEvidenceRegistry","f(address,uint64)",admin,3600)
            escrow = deploy("DiscoveryRoundEscrow","f(address,address,address)",token,registry,admin)
            for target, assignments in [(escrow,[("PLANNER_ROLE",admin),("REVIEWER_ROLE",reviewer_a),("REVIEWER_ROLE",reviewer_b)]),
                                        (registry,[("RELAY_ROLE",reviewer_a),("RELAY_ROLE",reviewer_b)])]:
                for role, who in assignments:
                    call(target,"grantRole(bytes32,address)",run(args.cast,"keccak",role),who)
                for index,who in enumerate([admin,reviewer_a,reviewer_b]):
                    call(target,"setOperatorCluster(address,bytes32)",who,"0x"+format(index+100,"064x"))
            trust = json.loads((ROOT / "examples/discovery/service-trust.json").read_text())
            trust.update(escrow_address=escrow,usdc_asset=f"eip155:31337/erc20:{token.lower()}")
            policy = json.loads((ROOT / "examples/discovery/service-policy.json").read_text())
            policy["economics"]["usdc_asset"] = trust["usdc_asset"]
            trust_path, policy_path = directory/"trust.json", directory/"policy.json"
            trust_path.write_text(json.dumps(trust)); policy_path.write_text(json.dumps(policy))
            roots = json.loads(run(args.cli,"discovery-prepare",trust_path,policy_path))
            params = SimpleNamespace(**vars(args),action="create-round",plan=str(directory/"plan.json"))
            prepared = transaction(params,trust,policy,roots)
            send(prepared["data"],prepared["to"])
            call(token,"mint(address,uint256)",admin,3_500_000_000)
            call(token,"approve(address,uint256)",escrow,3_500_000_000)
            receipts = [call(escrow,"fund(bytes32,uint8,uint256,bytes32)",digest(roots["round_id"]),i,500_000_000,"0x"+"8"*64) for i in range(7)]
            rpc("anvil_mine",["0xc"])
            code_hash = run(args.cast,"keccak",rpc("eth_getCode",[escrow,"latest"]))
            donors = json.loads((ROOT/"examples/discovery/pilot.json").read_text())["funding"][0]
            funding=[]
            for receipt in receipts:
                log=next(log for log in receipt["logs"] if log["address"].lower()==escrow.lower())
                params.action="observe-funding";params.transaction=receipt["transactionHash"];params.log_index=int(log["logIndex"],16)
                params.confirmations=12;params.code_hash=code_hash
                params.donor_cluster=donors["donor_cluster"];params.administrator_cluster=donors["administrator_cluster"]
                funding.append(observe(params,trust,policy,roots,rpc))
            assert [f["category"] for f in funding]==CATEGORIES
            (directory/"funding.json").write_text(json.dumps(funding))
            env=dict(os.environ,XLEMMA_EVM_TEST_DIR=str(directory))
            check=subprocess.run(["cargo","test","--locked","-p","xlemma-economics","--test","discovery_service","funded_evm_round_fixture","--","--ignored"],cwd=ROOT,env=env,capture_output=True,text=True,timeout=180)
            assert check.returncode==0,check.stdout+check.stderr
            plan=json.loads((directory/"plan.json").read_text())
            assert plan["round_id"]==roots["round_id"] and plan["total_units"]==301_200_000
            rpc("evm_setNextBlockTimestamp",[1210]);rpc("evm_mine",[])
            params.action="publish-evidence";params.evidence=str(directory/"publication.json");params.registry=registry
            publication=transaction(params,trust,policy,roots)
            send(publication["data"],registry,reviewer_a);send(publication["data"],registry,reviewer_b)
            params.action="propose-plan"
            proposed=transaction(params,trust,policy,roots);send(proposed["data"],escrow)
            commitment=rpc("eth_call",[{"to":escrow,"data":run(args.cast,"calldata","planCommitment(bytes32)",digest(plan["plan_id"]))},"latest"])
            for who in [reviewer_a,reviewer_b]:
                call(escrow,"approvePlan(bytes32,bytes32)",digest(plan["plan_id"]),commitment,sender=who)
            rpc("evm_setNextBlockTimestamp",[5000]);rpc("evm_mine",[])
            settled=call(escrow,"executePlan(bytes32)",digest(roots["round_id"]))
            rpc("anvil_mine",["0xc"])
            log=next(log for log in settled["logs"] if log["address"].lower()==escrow.lower())
            params.action="observe-settlement";params.transaction=settled["transactionHash"];params.log_index=int(log["logIndex"],16)
            observed=observe(params,trust,policy,roots,rpc)
            assert observed["plan_id"]==plan["plan_id"]
            for item in plan["items"]:
                balance=int(rpc("eth_call",[{"to":token,"data":run(args.cast,"calldata","balanceOf(address)",item["recipient"])},"latest"]),16)
                assert balance==sum(i["amount_units"] for i in plan["items"] if i["recipient"]==item["recipient"])
            for category in range(7):
                call(escrow,"refund(bytes32,uint8)",digest(roots["round_id"]),category)
            remaining=int(rpc("eth_call",[{"to":token,"data":run(args.cast,"calldata","balanceOf(address)",escrow)},"latest"]),16)
            assert remaining==0
            # A separate unfunded allocation expires onchain; receipt observation
            # must distinguish this from a successful settlement.
            expired_policy=json.loads(json.dumps(policy))
            expired_policy["economics"].update(name="unpaid-expiry-case",opens_at=6000,submissions_close_at=6100,appeals_close_at=6200,review_deadline=6300)
            policy_path.write_text(json.dumps(expired_policy))
            expired_roots=json.loads(run(args.cli,"discovery-prepare",trust_path,policy_path))
            params.action="create-round"
            prepared=transaction(params,trust,expired_policy,expired_roots);send(prepared["data"],escrow)
            rpc("evm_setNextBlockTimestamp",[expired_policy["settlement_expires_at"]+1]);rpc("evm_mine",[])
            expired=call(escrow,"expire(bytes32)",digest(expired_roots["round_id"]))
            rpc("anvil_mine",["0xc"])
            log=next(log for log in expired["logs"] if log["address"].lower()==escrow.lower())
            params.action="observe-expiry";params.transaction=expired["transactionHash"];params.log_index=int(log["logIndex"],16)
            observed_expiry=observe(params,trust,expired_policy,expired_roots,rpc)
            assert observed_expiry["kind"]=="observe_expiry" and observed_expiry["round_id"]==expired_roots["round_id"]
            print("PASS: confirmed EVM funding, signed Rust allocation, independent certificate publication, exact USDC settlement and refunds")
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill();process.wait(timeout=5)


if __name__ == "__main__":
    main()
