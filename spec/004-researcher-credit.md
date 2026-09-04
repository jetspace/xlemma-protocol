# XLIP-004 — Research Credit and Research Vault

## Backing

For each researcher `i`, independently valued backing `A_i` MUST be at least outstanding credit supply `B_i`.

## Minting

Credits MAY be minted against stable deposits, settled external revenue, funded grants/bounties, or conservatively valued prepaid compute. Credits MUST NOT be minted against their own market price, expected future profit, or an unverified lemma.

## Authorization

A variable-cost job SHOULD lock a maximum authorization. Settlement MUST burn only actual consumed credit, release equivalent backing to service providers, and return or unlock the unused amount.

## Consensus neutrality

Research Credit MUST NOT weight formal consensus, committee selection, verifier reputation, or challenges.

## Transfers

V1 credits SHOULD restrict transfers to the researcher, vault, escrow, approved service nodes, and redemption routes. Public profit-linked instruments are outside this specification.
