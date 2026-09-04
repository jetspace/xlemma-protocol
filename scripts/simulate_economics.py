#!/usr/bin/env python3
"""Deterministic xLemma economic sanity simulation."""

from decimal import Decimal, ROUND_DOWN

BPS = Decimal(10_000)


def bps(amount: Decimal, rate: int) -> Decimal:
    return (amount * Decimal(rate) / BPS).quantize(Decimal("0.000001"), rounding=ROUND_DOWN)


def main() -> None:
    # Research-credit conservation.
    backing = Decimal("1000")
    supply = Decimal("1000")
    maximum_authorization = Decimal("125")
    actual_usage = Decimal("40")
    locked = maximum_authorization
    locked -= maximum_authorization
    supply -= actual_usage
    backing -= actual_usage
    assert backing == supply == Decimal("960")
    assert locked == 0

    # Net revenue and waterfall.
    gross = Decimal("1400")
    net = gross - Decimal("100") - Decimal("200") - Decimal("50") - Decimal("50")
    assert net == Decimal("1000")
    waterfall = {
        "creator": 6500,
        "upstream": 0,
        "security": 800,
        "open_research": 1700,
        "dispute": 500,
        "operations": 500,
    }
    assert sum(waterfall.values()) == 10_000
    allocations = {name: bps(net, rate) for name, rate in waterfall.items()}
    assert sum(allocations.values()) == net

    creator = allocations["creator"]
    compound = bps(creator, 6000)
    cash = creator - compound
    assert creator == Decimal("650.000000")
    assert compound == Decimal("390.000000")
    assert cash == Decimal("260.000000")

    # Conservative compute-savings signal, payable only from one explicitly
    # authorized, non-recursive impact pool.
    economic_policy_root = "blake3:economic-policy"
    eligible_economic_edge_root = "blake3:eligible-economic-edge"
    settlement_receipt_id = "xlr:blake3:settlement"
    impact_pool_budget = Decimal("100")
    assert economic_policy_root and eligible_economic_edge_root and settlement_receipt_id
    mean_without = Decimal("10")
    standard_error = Decimal("1")
    lcb_multiplier = Decimal("1.645")
    observed_with = Decimal("5")
    conservative_savings = max(
        Decimal(0), mean_without - lcb_multiplier * standard_error - observed_with
    )
    uncapped = conservative_savings * Decimal("0.20")
    cap = net * Decimal("0.10")
    impact_allocation = min(uncapped, cap, impact_pool_budget)
    assert impact_allocation == Decimal("0.67100")

    print("Research-credit backing after 40-unit settlement: 960")
    print("Net external research revenue: 1000")
    print("Creator allocation: 650")
    print("Auto-compounded into backed research credits: 390")
    print("Researcher cash payout: 260")
    print(f"Authorized compute-impact allocation: {impact_allocation}")
    print("economic sanity simulation passed")


if __name__ == "__main__":
    main()
