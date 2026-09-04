# Legal and rights boundaries

This is an architectural issue list, not legal advice.

Under U.S. Copyright Office guidance, ideas, procedures, processes, systems,
methods, principles, and discoveries are not protected merely because they are
described in an artifact; original text, diagrams, code, or other expression
may be treated differently. USPTO eligibility analysis likewise does not make
a mathematical formula patent eligible by tokenizing it; the claim as a whole
must satisfy the applicable statutory analysis. Other jurisdictions differ,
and every deployment requires qualified counsel.

## Three objects hidden by “ownership”

1. **Origin/provenance:** nontransferable evidence that a researcher committed
   a claim or artifact at a time. It establishes a protocol record, not
   exclusive ownership of truth.
2. **Artifact/legal rights:** rights actually controlled in manuscripts, code,
   diagrams, data, experimental records, commercial implementations, eligible
   patent interests, or contracts.
3. **Economic participation:** entitlement to a defined revenue source under an
   explicit policy or agreement.

Economic participation must state the payer, source, calculation base,
exclusions, duration, share, cap, transfer rules, and dispute procedure. “Token
holders own future theorem royalties” is not a valid protocol representation.

## What a proof capsule can credibly represent

- a signed and timestamped priority claim;
- attribution and contribution evidence;
- immutable manuscript, code, data, model-output, and proof-artifact roots;
- the terms of an actual license or contract;
- custody or control of a legal rights-holding entity;
- access, support, or service entitlements;
- revenue routing under an enforceable agreement.

## What it cannot create by assertion

- ownership of an abstract mathematical truth;
- copyright in an unprotectable idea or fact;
- a patent on ineligible abstract mathematics;
- rights already assigned to an employer, university, sponsor, funder, or collaborator;
- human authorship merely because a person prompted a model;
- freedom from third-party claims;
- exemption from securities, commodities, tax, payments, sanctions, export, privacy, or consumer law.

## Required rights-clearance fields

- contributor and employer relationship;
- university or laboratory policy;
- grant and sponsor terms;
- collaborator agreements;
- prior publication and open-source licenses;
- patent filing or confidentiality status;
- dataset provenance and consent;
- ASTRA/model contribution disclosure;
- jurisdiction and governing agreement;
- transfer and sublicense limitations.

## Token architecture boundary

The V1 researcher token is a fully backed restricted service credit. A freely transferable token promoted around future researcher or protocol profits has a materially different risk profile and belongs in a separate legally reviewed vehicle. A technical ERC-4626 interface does not resolve that legal analysis.

## Capsule modes

- **Commons** permits public reuse without mandatory per-use protocol
  fees and may receive grants, donations, sponsorship, or impact-pool funding.
- **Reciprocal** applies a bounded, nonrecursive upstream pool to named xLemma
  economic flows without creating a downstream veto or ownership of truth.
- **Commercial Artifact** licenses only controlled artifacts or services under
  bounded, explicit terms.
- **Sponsored Challenge** declares funded acceptance, allocation, result
  rights, and disputes before work begins.

A formal dependency is never, by itself, a license or royalty obligation.

## Public claims

Do not display “owns theorem” or an undifferentiated “verified” badge. Prefer:

```text
Origin claim recorded at [time/order reference]
Formal status: Gold under policy [PolicyID]
Novelty status: [probability/decision/corpus cutoff]
Rights: see RightsManifest [hash]
Token: controls only the entitlements in that manifest
```
