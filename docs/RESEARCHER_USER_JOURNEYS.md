# Researcher and participant journeys

This document defines the expected end-to-end experience for the protocol's target user: a decentralized researcher who controls their identity, treasury, proof graph, and research-credit economy.

## 1. Researcher onboarding

1. Generate an offline root identity key and separate operational signing keys.
2. Publish a `ResearcherNodeManifest` containing public identity keys, supported domains, governance policy, vault address, and researcher-credit identifier.
3. Optionally link the research persona to a pseudonymous UserCredential; this does not make legal identity public or grant consensus-node authority.
4. Deploy or join a reviewed `ResearchVault` with a neutral settlement asset.
5. Deposit backing or receive a grant allocation; mint no more research credits than backing received.
6. Configure default ASTRA budget, Lean trust policy, novelty policy, storage policy, revenue waterfall, and auto-compound rate.
7. Register public disclosures for institutional, university, employment, sponsor, and grant-related rights restrictions.
8. Select Commons, Reciprocal, Commercial Artifact, or Sponsored Challenge as the capsule economic constitution.
9. Run a local proof-validation dry run before purchasing independent assurance.

The protocol supports pseudonymity, but economic counterparties, jurisdictions, and regulated service providers may impose separate identity or screening requirements.

## 2. Create and verify a lemma

```text
research idea / formula
  → origin commitment
  → human-readable LaTeX statement
  → ASTRA-assisted formal target
  → researcher approves exact Lean target
  → independent statement-alignment review where the policy requires it
  → ClaimID derived from elaborated type
  → ASTRA proof search and repair
  → reproducible artifact bundle
  → x402 authorization in backed R_i
  → randomly assigned independent checkers
  → commit-reveal observations
  → PoIR certificate or quarantine
  → challenge period
  → published Lemma Capsule
```

### Researcher controls

The researcher selects:

- maximum spend;
- completion mode: spot, economy, deadline, reserved, or competitive;
- permitted models and data-disclosure policy;
- Lean theory and axiom policy;
- assurance level;
- publication visibility;
- rights and license terms that the researcher is actually entitled to grant;
- capsule economic mode and any explicit bounded economic-policy edges;
- revenue allocation and auto-compound percentage.

The researcher cannot select friendly final verifiers, weight validity with their own token, remove dissent, or bypass a mandatory challenge period.

## 3. Pay with the researcher's token

1. The researcher obtains a signed service quote in settlement-asset units.
2. The x402 extension identifies the job, claim, artifact commitment, verification policy, model policy, rights root, revenue route, and quote.
3. The researcher authorizes a maximum amount of their restricted `R_i` credits.
4. The vault locks those credits and binds settlement to the quoted payee or router.
5. Actual service usage burns only the consumed credits and releases exactly matching neutral backing.
6. Unused credits return to the researcher.
7. Payment and verification receipts remain separate.

An unbacked personal token may be accepted voluntarily by a private counterparty outside the standard path, but it does not qualify as protocol backing and cannot influence formal consensus.

## 4. Earn from a verified result

A result can generate realized revenue through:

- a pre-funded bounty;
- paid proof-generation or verification services;
- certified artifact access;
- commercial code, data, or implementation licenses;
- interactive explanations or integrations;
- explicitly funded downstream impact or contractual use;
- retrospective public-goods funding;
- training-data and proof-trajectory compensation;
- conservatively measured compute-impact allocations from a bounded pool.

Gross receipts are reduced by direct delivery cost, compute cost, refunds, maintenance reserves, and other declared costs. The remaining net revenue flows through an immutable revenue route.

The researcher's contributor allocation can be split between:

- a cash payout; and
- an auto-compound deposit into the Research Vault, which mints an equal amount of newly backed `R_i`.

Token price appreciation is never booked as research profit.

## 5. Support a decentralized researcher

A supporter chooses a clearly defined instrument:

- **Grant:** funds work without repayment rights; receives a support certificate and attribution.
- **Bounty:** escrows a reward against an exact claim and acceptance policy.
- **Compute pre-purchase:** purchases future verified services or compute credits.
- **Commercial co-development:** receives only the rights described in a separate signed agreement.
- **Donation to public-goods pool:** funds negative results, maintenance, formalization, or open infrastructure.

Support certificates do not imply ownership of mathematical truth. A transferable promise of passive profit is outside the core protocol and requires separate legal structuring.

## 6. Operate a prover node

A prover node:

1. publishes service capability, price, capacity, model policy, privacy guarantees, and collateral;
2. receives randomly routed tasks within declared capability;
3. runs ASTRA or another approved prover behind the provider-neutral adapter;
4. returns candidate artifacts plus signed compute receipts;
5. earns actual compute cost and a declared success bonus;
6. never certifies its own candidate for Gold status.

A prover should preserve failed trajectories when authorized because they can improve cost models and future theorem provers.

## 7. Operate a checker node

A checker-node operator:

1. obtains a pseudonymous verified-participant UserCredential;
2. delegates a V2-or-higher OperatorCredential and one NodeCredential per machine/key;
3. publishes a fresh non-revocation status proof and conservative OperatorClusterID;
4. bonds a neutral asset;
5. declares checker family, binary digest, environment image, provider, region, and capacity;
6. receives a committee assignment only if the credential, role, conflict, reputation, bond, and diversity gates pass;
7. executes the exact artifact in an isolated environment;
8. commits to its observation before learning peer observations;
9. reveals its full receipt and trace root;
10. receives its execution fee whether the result is `PASS`, `FAIL`, `ERROR`, or honest dissent.

Multiple NodeIDs under the same verified participant count as one independence
domain. The participant's private verification evidence remains with the
issuer; committee records expose only typed IDs, commitments, qualifications,
and the credential-chain root.

It can be slashed for provable fabrication, equivocation, concealed operator control, or false custody evidence—not merely for disagreeing.

## 8. Challenge a certificate

A challenger submits:

- the certificate identifier;
- a bonded challenge where required;
- reproducible counterevidence;
- the relevant checker, dependency, environment, rights, novelty, or availability failure class;
- a content-addressed evidence bundle.

The challenged object becomes fail-closed according to policy. An expanded committee reproduces the evidence. Outcomes are:

- challenge dismissed, followed by a fresh safety window;
- certificate quarantined pending repair;
- certificate rejected or revoked;
- rights/revenue route suspended without erasing attribution;
- challenger reward from objectively slashed collateral.

## 9. Reuse an upstream lemma

A downstream researcher:

1. imports a pinned, verified upstream artifact;
2. records only dependencies present in the final proof object;
3. receives a lower proof-search cost or smaller context requirement;
4. earns from the downstream result under its own contribution manifest;
5. separately accepts an economic policy, if any, that may fund a bounded upstream or impact pool;
6. optionally participates in randomized withholding experiments that estimate conservative compute impact.

The formal dependency never creates a debt. An allocation requires a settled
revenue event, eligible economic-policy edge, explicit pool, and non-recursive
treatment. Dependencies are clustered, capped, and cycle-checked to avoid
stuffing and royalty explosions.

## 10. Publish a negative result

A failed proof attempt can still become an attributable artifact when it contains reproducible value:

- a counterexample;
- a formal impossibility result;
- a disproved conjecture;
- a benchmark of failed approaches;
- a proof-state trajectory useful for training;
- a clearly bounded inconclusive search.

Negative results do not receive a false validity badge. They can receive public-goods grants, training-data rewards, storage support, and reputation for high-quality evidence.

## 11. Correct or supersede work

Historical objects are never silently edited. The researcher:

1. creates a new manifest and identifier;
2. links it through `AMENDS`, `CORRECTS`, `SUPERSEDES`, or formally proved `EQUIVALENT_TO` edges;
3. preserves the old artifact and receipts;
4. updates presentation and public status;
5. pauses or reroutes new revenue if a material issue affects validity or rights;
6. retains original attribution and the complete correction history.
