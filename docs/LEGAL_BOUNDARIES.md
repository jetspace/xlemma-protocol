# Legal and rights boundaries

This is an architectural issue list, not legal advice.

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

## Public claims

Do not display “owns theorem” or an undifferentiated “verified” badge. Prefer:

```text
Origin claim recorded at [time/order reference]
Formal status: Gold under policy [PolicyID]
Novelty status: [probability/decision/corpus cutoff]
Rights: see RightsManifest [hash]
Token: controls only the entitlements in that manifest
```
