# Agent instructions for this repository

## Mission

Build xLemma as proof-carrying decentralized research infrastructure for sovereign researchers. Optimize for reproducibility, economic conservation, evidence transparency, and independent verification.

## Invariants agents must preserve

- Never replace exact Lean/checker reproduction with token voting.
- Never allow ASTRA or another proof producer to self-certify.
- Never mint research credits without independently valued backing.
- Never pay verifiers only when they return `PASS`.
- Never resolve checker-family divergence by majority vote.
- Never hash source text as the final formal ClaimID.
- Never make token transfer rewrite attribution or validity.
- Never describe unrealized token appreciation as research profit.
- Never reward unused dependencies or uncapped recursive royalties.
- Never silently mutate a historical proof object, contribution manifest, rights manifest, or receipt.

## Coding priorities

1. Pure deterministic logic with property tests.
2. Explicit state machines and typed IDs.
3. Fail-closed behavior across payments, verification, and rights.
4. Provider-neutral boundaries for models, chains, storage, and checkers.
5. Structured receipts for every external action.
6. No secrets or unpublished proof contents in logs.
7. Threat-model and traceability updates with material changes.

## Completion standard

A change is not complete until applicable formatting, unit, property, schema, contract, Lean, and integration tests pass and documentation reflects new trust assumptions.
