//! Deterministic discovery and order-book logic for the XLMP node marketplace.

use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    canonical_json_hash, AdvertisementId, Amount, DiscoveryId, NodeDiscoveryRequest,
    NodeDiscoveryResult, NodeReputationSnapshot, NodeServiceAdvertisement, ReputationId,
    ServiceCapability, ServiceMatch, ServiceMatchId, ServiceOrder, ServiceOrderId,
};

/// Cryptographic checks are mandatory at every marketplace state boundary.
/// Deployments normally back this with node credentials and an operator or
/// committee key registry; there is intentionally no accept-all implementation.
pub trait MarketplaceProofVerifier {
    fn verify_advertisement(&self, advertisement: &NodeServiceAdvertisement) -> bool;
    fn verify_order(&self, order: &ServiceOrder) -> bool;
    fn verify_match(&self, service_match: &ServiceMatch) -> bool;
    fn verify_reputation(&self, reputation: &NodeReputationSnapshot) -> bool;
}

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("advertisement has an invalid identity, window, endpoint, capability, or signature")]
    InvalidAdvertisement,
    #[error(
        "advertisement supersession must preserve node/operator identity and increase sequence"
    )]
    InvalidSupersession,
    #[error("discovery request is invalid or expired")]
    InvalidDiscovery,
    #[error("service order is invalid or expired")]
    InvalidOrder,
    #[error("matching reputation snapshot is absent, invalid, or belongs to another node")]
    InvalidReputation,
    #[error("service price asset, decimals, unit, or scale is incompatible")]
    IncompatiblePrice,
    #[error("price calculation overflow")]
    PriceOverflow,
    #[error("no advertised service satisfies every requested constraint")]
    NoMatchingService,
    #[error("service match schedule is invalid")]
    InvalidSchedule,
    #[error("append-only order book already contains this identifier")]
    DuplicateIdentifier,
    #[error("service order already has a recorded match")]
    OrderAlreadyMatched,
    #[error("marketplace signature or credential proof is invalid")]
    InvalidProof,
    #[error(transparent)]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
    #[error(transparent)]
    Identifier(#[from] xlemma_core::IdError),
}

#[derive(Default)]
pub struct ServiceOrderBook {
    advertisements: BTreeMap<AdvertisementId, NodeServiceAdvertisement>,
    orders: BTreeMap<ServiceOrderId, ServiceOrder>,
    matches: BTreeMap<ServiceMatchId, ServiceMatch>,
    matched_orders: BTreeSet<ServiceOrderId>,
}

impl ServiceOrderBook {
    pub fn publish_advertisement(
        &mut self,
        advertisement: NodeServiceAdvertisement,
        now: DateTime<Utc>,
        verifier: &impl MarketplaceProofVerifier,
    ) -> Result<(), MarketplaceError> {
        validate_advertisement(&advertisement, now)?;
        if !verifier.verify_advertisement(&advertisement) {
            return Err(MarketplaceError::InvalidProof);
        }
        if self
            .advertisements
            .contains_key(&advertisement.advertisement_id)
        {
            return Err(MarketplaceError::DuplicateIdentifier);
        }
        if self.advertisements.values().any(|existing| {
            existing.node_id == advertisement.node_id
                && (existing.operator_id != advertisement.operator_id
                    || existing.operator_cluster_id != advertisement.operator_cluster_id)
        }) {
            return Err(MarketplaceError::InvalidSupersession);
        }
        let latest = self
            .advertisements
            .values()
            .filter(|existing| {
                existing.node_id == advertisement.node_id
                    && existing.operator_cluster_id == advertisement.operator_cluster_id
            })
            .max_by_key(|existing| existing.sequence);
        match latest {
            None if advertisement.sequence != 1 || advertisement.supersedes.is_some() => {
                return Err(MarketplaceError::InvalidSupersession);
            }
            Some(parent)
                if advertisement.supersedes.as_ref() != Some(&parent.advertisement_id)
                    || parent.sequence.checked_add(1) != Some(advertisement.sequence) =>
            {
                return Err(MarketplaceError::InvalidSupersession);
            }
            _ => {}
        }
        self.advertisements
            .insert(advertisement.advertisement_id.clone(), advertisement);
        Ok(())
    }

    pub fn submit_order(
        &mut self,
        order: ServiceOrder,
        now: DateTime<Utc>,
        verifier: &impl MarketplaceProofVerifier,
    ) -> Result<(), MarketplaceError> {
        validate_order(&order, now)?;
        if !verifier.verify_order(&order) {
            return Err(MarketplaceError::InvalidProof);
        }
        match self.orders.entry(order.order_id.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(order);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                Err(MarketplaceError::DuplicateIdentifier)
            }
        }
    }

    pub fn record_match(
        &mut self,
        service_match: ServiceMatch,
        verifier: &impl MarketplaceProofVerifier,
    ) -> Result<(), MarketplaceError> {
        if !verifier.verify_match(&service_match) {
            return Err(MarketplaceError::InvalidProof);
        }
        let order = self
            .orders
            .get(&service_match.order_id)
            .ok_or(MarketplaceError::InvalidOrder)?;
        let advertisement = self
            .advertisements
            .get(&service_match.advertisement_id)
            .ok_or(MarketplaceError::InvalidAdvertisement)?;
        if self.matched_orders.contains(&service_match.order_id) {
            return Err(MarketplaceError::OrderAlreadyMatched);
        }
        if self.matches.contains_key(&service_match.match_id) {
            return Err(MarketplaceError::DuplicateIdentifier);
        }
        service_match.match_id.validate()?;
        let request = discovery_request_for_order(order, service_match.matched_at)?;
        let price_and_capability_match = advertisement.capabilities.iter().any(|capability| {
            capability_matches(&request, capability)
                && total_price(service_match.reserved_units, &capability.price)
                    .is_ok_and(|total| total == service_match.agreed_price)
        });
        if service_match.match_id != expected_service_match_id(&service_match)?
            || service_match.advertisement_sequence != advertisement.sequence
            || service_match.node_id != advertisement.node_id
            || service_match.operator_id != advertisement.operator_id
            || service_match.operator_cluster_id != advertisement.operator_cluster_id
            || service_match.service != order.service
            || service_match.reserved_units != order.quantity_units
            || service_match
                .agreed_price
                .ensure_compatible(&order.maximum_total_price)
                .is_err()
            || service_match.agreed_price.units > order.maximum_total_price.units
            || service_match.matched_at < order.created_at
            || service_match.matched_at >= order.expires_at
            || service_match.matched_at < advertisement.valid_from
            || service_match.matched_at >= advertisement.valid_until
            || service_match.scheduled_start < service_match.matched_at
            || service_match.scheduled_end <= service_match.scheduled_start
            || service_match.scheduled_end > order.delivery_deadline
            || service_match.signature.trim().is_empty()
            || !price_and_capability_match
        {
            return Err(MarketplaceError::InvalidSchedule);
        }
        self.matched_orders.insert(service_match.order_id.clone());
        self.matches
            .insert(service_match.match_id.clone(), service_match);
        Ok(())
    }

    pub fn active_advertisements(&self, now: DateTime<Utc>) -> Vec<&NodeServiceAdvertisement> {
        let superseded = self
            .advertisements
            .values()
            .filter_map(|advertisement| advertisement.supersedes.as_ref())
            .collect::<BTreeSet<_>>();
        self.advertisements
            .values()
            .filter(|advertisement| {
                !superseded.contains(&advertisement.advertisement_id)
                    && advertisement.valid_from <= now
                    && advertisement.valid_until > now
            })
            .collect()
    }

    pub fn order(&self, order_id: &ServiceOrderId) -> Option<&ServiceOrder> {
        self.orders.get(order_id)
    }
}

pub fn expected_advertisement_id(
    advertisement: &NodeServiceAdvertisement,
) -> Result<AdvertisementId, MarketplaceError> {
    Ok(advertisement.derive_advertisement_id()?)
}

pub fn expected_service_match_id(
    service_match: &ServiceMatch,
) -> Result<ServiceMatchId, MarketplaceError> {
    Ok(service_match.derive_match_id()?)
}

pub fn validate_advertisement(
    advertisement: &NodeServiceAdvertisement,
    now: DateTime<Utc>,
) -> Result<(), MarketplaceError> {
    advertisement.advertisement_id.validate()?;
    advertisement.node_id.validate()?;
    advertisement.operator_id.validate()?;
    advertisement.operator_cluster_id.validate()?;
    advertisement.user_credential_id.validate()?;
    advertisement.operator_credential_id.validate()?;
    advertisement.node_credential_id.validate()?;
    advertisement.reputation_snapshot_id.validate()?;
    advertisement.bond_id.validate()?;
    if advertisement.advertisement_id != expected_advertisement_id(advertisement)?
        || advertisement.sequence == 0
        || advertisement.roles.is_empty()
        || advertisement.endpoints.is_empty()
        || advertisement.capabilities.is_empty()
        || advertisement.credential_chain_root.trim().is_empty()
        || advertisement.jurisdiction_class.trim().is_empty()
        || advertisement.terms_root.trim().is_empty()
        || advertisement.delegation_signature.trim().is_empty()
        || advertisement.signature.trim().is_empty()
        || advertisement.valid_from > now
        || advertisement.valid_until <= now
        || advertisement.valid_from >= advertisement.valid_until
        || advertisement.endpoints.iter().any(|endpoint| {
            endpoint.transport.trim().is_empty()
                || endpoint.uri.trim().is_empty()
                || endpoint.authentication_profile.trim().is_empty()
        })
        || advertisement.capabilities.iter().any(|capability| {
            !advertisement
                .roles
                .contains(&xlemma_core::service_role(capability.service))
                || capability.maximum_parallel_jobs == 0
                || capability.capacity_units == 0
                || capability.available_units > capability.capacity_units
                || capability.p50_latency_ms > capability.p95_latency_ms
                || capability.price.quantity_scale == 0
                || capability.price.amount.units == 0
                || capability.price.amount.asset.trim().is_empty()
                || capability.price.unit_name.trim().is_empty()
                || (matches!(
                    capability.service,
                    xlemma_core::NodeServiceKind::OfficialVerification
                        | xlemma_core::NodeServiceKind::IndependentVerification
                ) && capability.checker_families.is_empty())
        })
    {
        return Err(MarketplaceError::InvalidAdvertisement);
    }
    Ok(())
}

pub fn validate_order(order: &ServiceOrder, now: DateTime<Utc>) -> Result<(), MarketplaceError> {
    order.order_id.validate()?;
    order.job_id.validate()?;
    if order.requester.trim().is_empty()
        || order.required_role != xlemma_core::service_role(order.service)
        || order.quantity_units == 0
        || order.maximum_total_price.units == 0
        || order.maximum_total_price.asset.trim().is_empty()
        || order.terms_root.trim().is_empty()
        || order.signature.trim().is_empty()
        || order.created_at > now
        || order.expires_at <= now
        || order.delivery_deadline <= now
        || order.created_at >= order.expires_at
        || order.expires_at > order.delivery_deadline
    {
        return Err(MarketplaceError::InvalidOrder);
    }
    Ok(())
}

pub fn discover_services(
    request: &NodeDiscoveryRequest,
    advertisements: &[&NodeServiceAdvertisement],
    reputations: &BTreeMap<ReputationId, NodeReputationSnapshot>,
    now: DateTime<Utc>,
    signature: String,
    verifier: &impl MarketplaceProofVerifier,
) -> Result<NodeDiscoveryResult, MarketplaceError> {
    if request.requester.trim().is_empty()
        || request.services.is_empty()
        || request.requested_at > now
        || request.expires_at <= now
        || request.requested_at >= request.expires_at
        || signature.trim().is_empty()
    {
        return Err(MarketplaceError::InvalidDiscovery);
    }
    request.discovery_id.validate()?;

    let ranked = ranked_advertisements(request, advertisements, reputations, now, verifier)?;
    let advertisement_ids = ranked
        .into_iter()
        .map(|candidate| candidate.advertisement.advertisement_id.clone())
        .collect::<Vec<_>>();
    let root_digest = canonical_json_hash(
        "xlemma-node-discovery-result-v1",
        &(&request.discovery_id, &advertisement_ids, now),
    )?;
    Ok(NodeDiscoveryResult {
        discovery_id: request.discovery_id.clone(),
        advertisement_ids,
        advertisement_set_root: format!(
            "blake3:{}",
            blake3::Hash::from_bytes(root_digest).to_hex()
        ),
        generated_at: now,
        signature,
    })
}

// Schedule, signature, and proof-verifier inputs remain explicit so callers
// cannot accidentally inherit untrusted matching context from mutable state.
#[allow(clippy::too_many_arguments)]
pub fn match_service_order(
    order: &ServiceOrder,
    advertisements: &[&NodeServiceAdvertisement],
    reputations: &BTreeMap<ReputationId, NodeReputationSnapshot>,
    now: DateTime<Utc>,
    scheduled_start: DateTime<Utc>,
    scheduled_end: DateTime<Utc>,
    signature: String,
    verifier: &impl MarketplaceProofVerifier,
) -> Result<ServiceMatch, MarketplaceError> {
    validate_order(order, now)?;
    if !verifier.verify_order(order) {
        return Err(MarketplaceError::InvalidProof);
    }
    if scheduled_start < now
        || scheduled_end <= scheduled_start
        || scheduled_end > order.delivery_deadline
        || signature.trim().is_empty()
    {
        return Err(MarketplaceError::InvalidSchedule);
    }
    let request = discovery_request_for_order(order, now)?;
    let mut selected = None;
    for candidate in ranked_advertisements(&request, advertisements, reputations, now, verifier)? {
        let agreed_price = total_price(order.quantity_units, &candidate.capability.price)?;
        if agreed_price
            .ensure_compatible(&order.maximum_total_price)
            .is_ok()
            && agreed_price.units <= order.maximum_total_price.units
        {
            selected = Some((candidate, agreed_price));
            break;
        }
    }
    let (candidate, agreed_price) = selected.ok_or(MarketplaceError::NoMatchingService)?;

    let mut service_match = ServiceMatch {
        match_id: ServiceMatchId::derive(&"pending-service-match")?,
        order_id: order.order_id.clone(),
        advertisement_id: candidate.advertisement.advertisement_id.clone(),
        advertisement_sequence: candidate.advertisement.sequence,
        node_id: candidate.advertisement.node_id.clone(),
        operator_id: candidate.advertisement.operator_id.clone(),
        operator_cluster_id: candidate.advertisement.operator_cluster_id.clone(),
        service: order.service,
        reserved_units: order.quantity_units,
        agreed_price,
        scheduled_start,
        scheduled_end,
        matched_at: now,
        signature,
    };
    service_match.match_id = service_match.derive_match_id()?;
    Ok(service_match)
}

fn discovery_request_for_order(
    order: &ServiceOrder,
    requested_at: DateTime<Utc>,
) -> Result<NodeDiscoveryRequest, MarketplaceError> {
    Ok(NodeDiscoveryRequest {
        discovery_id: DiscoveryId::derive(&(
            "service-order-discovery-v1",
            &order.order_id,
            requested_at,
        ))?,
        requester: order.requester.clone(),
        services: BTreeSet::from([order.service]),
        required_roles: BTreeSet::from([order.required_role]),
        required_checker_families: order.required_checker_families.clone(),
        theory_id: order.theory_id.clone(),
        domains: order.domains.clone(),
        minimum_available_units: order.quantity_units,
        maximum_p95_latency_ms: order.maximum_p95_latency_ms,
        maximum_unit_price: None,
        reputation_requirements: order.reputation_requirements.clone(),
        excluded_operator_clusters: order.excluded_operator_clusters.clone(),
        requested_at,
        expires_at: order.expires_at,
    })
}

struct RankedAdvertisement<'a> {
    advertisement: &'a NodeServiceAdvertisement,
    capability: &'a ServiceCapability,
}

fn ranked_advertisements<'a>(
    request: &NodeDiscoveryRequest,
    advertisements: &'a [&NodeServiceAdvertisement],
    reputations: &BTreeMap<ReputationId, NodeReputationSnapshot>,
    now: DateTime<Utc>,
    verifier: &impl MarketplaceProofVerifier,
) -> Result<Vec<RankedAdvertisement<'a>>, MarketplaceError> {
    let mut ranked = Vec::new();
    for advertisement in advertisements {
        if validate_advertisement(advertisement, now).is_err()
            || !verifier.verify_advertisement(advertisement)
            || !request.required_roles.is_subset(&advertisement.roles)
            || request
                .excluded_operator_clusters
                .contains(&advertisement.operator_cluster_id)
        {
            continue;
        }
        let Some(reputation) = reputations.get(&advertisement.reputation_snapshot_id) else {
            continue;
        };
        if reputation.reputation_id != advertisement.reputation_snapshot_id
            || reputation.reputation_id.validate().is_err()
            || reputation.node_id.validate().is_err()
            || reputation.operator_id.validate().is_err()
            || reputation.operator_cluster_id.validate().is_err()
            || reputation.policy_id.validate().is_err()
            || reputation
                .supersedes
                .as_ref()
                .is_some_and(|identifier| identifier.validate().is_err())
            || reputation.node_id != advertisement.node_id
            || reputation.operator_id != advertisement.operator_id
            || reputation.operator_cluster_id != advertisement.operator_cluster_id
            || !reputation.vector.meets(&request.reputation_requirements)
            || reputation.period_start >= reputation.period_end
            || reputation.period_end > reputation.assessed_at
            || reputation.assessed_at > now
            || reputation.evidence_root.trim().is_empty()
            || reputation.assessor_signature.trim().is_empty()
            || !verifier.verify_reputation(reputation)
        {
            continue;
        }

        let mut capabilities = advertisement
            .capabilities
            .iter()
            .filter(|capability| capability_matches(request, capability))
            .collect::<Vec<_>>();
        let mut overflowed = false;
        capabilities.sort_by(
            |left, right| match market_price_order(&left.price, &right.price) {
                Ok(ordering) => ordering,
                Err(_) => {
                    overflowed = true;
                    std::cmp::Ordering::Equal
                }
            },
        );
        if overflowed {
            return Err(MarketplaceError::PriceOverflow);
        }
        if let Some(capability) = capabilities.into_iter().next() {
            ranked.push(RankedAdvertisement {
                advertisement,
                capability,
            });
        }
    }
    let mut overflowed = false;
    ranked.sort_by(|left, right| {
        market_price_order(&left.capability.price, &right.capability.price)
            .unwrap_or_else(|_| {
                overflowed = true;
                std::cmp::Ordering::Equal
            })
            .then_with(|| {
                left.capability
                    .p95_latency_ms
                    .cmp(&right.capability.p95_latency_ms)
            })
            .then_with(|| {
                left.advertisement
                    .advertisement_id
                    .cmp(&right.advertisement.advertisement_id)
            })
    });
    if overflowed {
        return Err(MarketplaceError::PriceOverflow);
    }
    if ranked.is_empty() {
        return Err(MarketplaceError::NoMatchingService);
    }
    Ok(ranked)
}

fn capability_matches(request: &NodeDiscoveryRequest, capability: &ServiceCapability) -> bool {
    if !request.services.contains(&capability.service)
        || capability.available_units < request.minimum_available_units
        || !request
            .required_checker_families
            .is_subset(&capability.checker_families)
        || !request.domains.is_subset(&capability.domains)
        || request
            .maximum_p95_latency_ms
            .is_some_and(|maximum| capability.p95_latency_ms > maximum)
        || request.theory_id.as_ref().is_some_and(|theory_id| {
            !capability.supported_theory_ids.is_empty()
                && !capability.supported_theory_ids.contains(theory_id)
        })
    {
        return false;
    }
    request
        .maximum_unit_price
        .as_ref()
        .is_none_or(|maximum| price_lte(&capability.price, maximum).unwrap_or(false))
}

fn compare_prices(
    left: &xlemma_core::ServicePrice,
    right: &xlemma_core::ServicePrice,
) -> Result<std::cmp::Ordering, MarketplaceError> {
    compatible_prices(left, right)?;
    let left_scaled = left
        .amount
        .units
        .checked_mul(u128::from(right.quantity_scale))
        .ok_or(MarketplaceError::PriceOverflow)?;
    let right_scaled = right
        .amount
        .units
        .checked_mul(u128::from(left.quantity_scale))
        .ok_or(MarketplaceError::PriceOverflow)?;
    Ok(left_scaled.cmp(&right_scaled))
}

/// Establishes a deterministic total order across price families while only
/// comparing numeric unit prices inside the same asset/decimal/unit family.
fn market_price_order(
    left: &xlemma_core::ServicePrice,
    right: &xlemma_core::ServicePrice,
) -> Result<std::cmp::Ordering, MarketplaceError> {
    let family_order = left
        .amount
        .asset
        .cmp(&right.amount.asset)
        .then_with(|| left.amount.decimals.cmp(&right.amount.decimals))
        .then_with(|| left.unit_name.cmp(&right.unit_name));
    if !family_order.is_eq() {
        return Ok(family_order);
    }
    Ok(compare_prices(left, right)?
        .then_with(|| {
            pricing_model_label(left.pricing_model).cmp(pricing_model_label(right.pricing_model))
        })
        .then_with(|| left.quantity_scale.cmp(&right.quantity_scale))
        .then_with(|| left.amount.units.cmp(&right.amount.units)))
}

fn pricing_model_label(model: xlemma_core::PricingModel) -> &'static str {
    match model {
        xlemma_core::PricingModel::Fixed => "fixed",
        xlemma_core::PricingModel::Metered => "metered",
        xlemma_core::PricingModel::UpTo => "up_to",
        xlemma_core::PricingModel::BatchSettlement => "batch_settlement",
        xlemma_core::PricingModel::InstitutionalInvoice => "institutional_invoice",
    }
}

fn price_lte(
    offered: &xlemma_core::ServicePrice,
    maximum: &xlemma_core::ServicePrice,
) -> Result<bool, MarketplaceError> {
    Ok(compare_prices(offered, maximum)?.is_le())
}

fn compatible_prices(
    left: &xlemma_core::ServicePrice,
    right: &xlemma_core::ServicePrice,
) -> Result<(), MarketplaceError> {
    if left.unit_name != right.unit_name
        || left.amount.asset != right.amount.asset
        || left.amount.decimals != right.amount.decimals
        || left.quantity_scale == 0
        || right.quantity_scale == 0
    {
        return Err(MarketplaceError::IncompatiblePrice);
    }
    Ok(())
}

fn total_price(
    quantity_units: u64,
    price: &xlemma_core::ServicePrice,
) -> Result<Amount, MarketplaceError> {
    if price.quantity_scale == 0 {
        return Err(MarketplaceError::IncompatiblePrice);
    }
    let numerator = u128::from(quantity_units)
        .checked_mul(price.amount.units)
        .ok_or(MarketplaceError::PriceOverflow)?;
    let scale = u128::from(price.quantity_scale);
    let units = numerator
        .checked_add(scale - 1)
        .ok_or(MarketplaceError::PriceOverflow)?
        / scale;
    Ok(Amount::new(
        units,
        price.amount.asset.clone(),
        price.amount.decimals,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use xlemma_core::{
        BondId, HardwareProfile, NodeCredentialId, NodeId, NodeReputationVector, NodeRole,
        NodeServiceKind, OperatorClusterId, OperatorCredentialId, OperatorId, PolicyId,
        PricingModel, ReputationMetric, ReputationRequirements, ServiceEndpoint, ServicePrice,
        UserCredentialId,
    };

    struct AcceptTestProofs;

    impl MarketplaceProofVerifier for AcceptTestProofs {
        fn verify_advertisement(&self, _: &NodeServiceAdvertisement) -> bool {
            true
        }

        fn verify_order(&self, _: &ServiceOrder) -> bool {
            true
        }

        fn verify_match(&self, _: &ServiceMatch) -> bool {
            true
        }

        fn verify_reputation(&self, _: &NodeReputationSnapshot) -> bool {
            true
        }
    }

    fn metric(score: u16) -> ReputationMetric {
        ReputationMetric {
            score_bps: score,
            sample_size: 100,
            evidence_root: "blake3:metric".into(),
        }
    }

    fn reputation(
        node: &NodeId,
        operator_id: &OperatorId,
        operator: &OperatorClusterId,
        id: &ReputationId,
        now: DateTime<Utc>,
    ) -> NodeReputationSnapshot {
        NodeReputationSnapshot {
            reputation_id: id.clone(),
            operator_id: operator_id.clone(),
            node_id: node.clone(),
            operator_cluster_id: operator.clone(),
            vector: NodeReputationVector {
                formal_accuracy: metric(9_800),
                availability: metric(9_700),
                latency: metric(9_500),
                novelty_calibration: metric(9_000),
                challenge_quality: metric(9_100),
                independence: metric(9_900),
                storage_quality: metric(9_500),
                integrity: metric(9_900),
            },
            policy_id: PolicyId::derive(&"reputation-policy").unwrap(),
            period_start: now - Duration::days(30),
            period_end: now,
            evidence_root: "blake3:reputation".into(),
            assessed_at: now,
            supersedes: None,
            assessor_signature: "signature".into(),
        }
    }

    fn advertisement(
        label: &str,
        price_units: u128,
        now: DateTime<Utc>,
    ) -> NodeServiceAdvertisement {
        let node_id = NodeId::derive(&label).unwrap();
        let operator_id = OperatorId::derive(&format!("operator-{label}")).unwrap();
        let operator_cluster_id = OperatorClusterId::derive(&format!("operator-{label}")).unwrap();
        let reputation_snapshot_id = ReputationId::derive(&label).unwrap();
        let mut advertisement = NodeServiceAdvertisement {
            advertisement_id: AdvertisementId::derive(&"placeholder").unwrap(),
            node_id,
            operator_id,
            operator_cluster_id,
            user_credential_id: UserCredentialId::derive(&format!("user-credential-{label}"))
                .unwrap(),
            operator_credential_id: OperatorCredentialId::derive(&format!(
                "operator-credential-{label}"
            ))
            .unwrap(),
            node_credential_id: NodeCredentialId::derive(&format!("node-credential-{label}"))
                .unwrap(),
            credential_chain_root: format!("blake3:credential-chain-{label}"),
            jurisdiction_class: "privacy-preserving-verified".into(),
            sequence: 1,
            roles: BTreeSet::from([NodeRole::OfficialKernelChecker]),
            endpoints: vec![ServiceEndpoint {
                transport: "https".into(),
                uri: format!("https://{label}.example/xlmp"),
                authentication_profile: "did-key-v1".into(),
            }],
            capabilities: vec![ServiceCapability {
                service: NodeServiceKind::OfficialVerification,
                implementation_id: Some("lean4checker@4.33.1".into()),
                checker_families: BTreeSet::from([xlemma_core::CheckerFamily::LeanKernel]),
                supported_theory_ids: BTreeSet::new(),
                domains: BTreeSet::from(["formal-mathematics".into()]),
                hardware: Some(HardwareProfile {
                    class: "cpu".into(),
                    architecture: "arm64".into(),
                    accelerator: None,
                    memory_mib: Some(8_192),
                    trusted_execution_attestation: None,
                }),
                maximum_parallel_jobs: 4,
                capacity_units: 10_000,
                available_units: 8_000,
                p50_latency_ms: 500,
                p95_latency_ms: 1_000,
                price: ServicePrice {
                    pricing_model: PricingModel::Metered,
                    unit_name: "checker-ms".into(),
                    quantity_scale: 100,
                    amount: Amount::new(price_units, "USDC", 6),
                },
            }],
            reputation_snapshot_id,
            bond_id: BondId::derive(&label).unwrap(),
            terms_root: "blake3:terms".into(),
            valid_from: now - Duration::minutes(1),
            valid_until: now + Duration::hours(1),
            supersedes: None,
            delegation_signature: "delegation-signature".into(),
            signature: "signature".into(),
        };
        advertisement.advertisement_id = expected_advertisement_id(&advertisement).unwrap();
        advertisement
    }

    fn discovery(now: DateTime<Utc>) -> NodeDiscoveryRequest {
        NodeDiscoveryRequest {
            discovery_id: DiscoveryId::derive(&"discovery").unwrap(),
            requester: "did:key:researcher".into(),
            services: BTreeSet::from([NodeServiceKind::OfficialVerification]),
            required_roles: BTreeSet::from([NodeRole::OfficialKernelChecker]),
            required_checker_families: BTreeSet::from([xlemma_core::CheckerFamily::LeanKernel]),
            theory_id: None,
            domains: BTreeSet::from(["formal-mathematics".into()]),
            minimum_available_units: 100,
            maximum_p95_latency_ms: Some(2_000),
            maximum_unit_price: Some(ServicePrice {
                pricing_model: PricingModel::UpTo,
                unit_name: "checker-ms".into(),
                quantity_scale: 100,
                amount: Amount::new(200, "USDC", 6),
            }),
            reputation_requirements: ReputationRequirements::default(),
            excluded_operator_clusters: BTreeSet::new(),
            requested_at: now,
            expires_at: now + Duration::minutes(5),
        }
    }

    fn reputation_map(
        advertisements: &[NodeServiceAdvertisement],
        now: DateTime<Utc>,
    ) -> BTreeMap<ReputationId, NodeReputationSnapshot> {
        advertisements
            .iter()
            .map(|advertisement| {
                (
                    advertisement.reputation_snapshot_id.clone(),
                    reputation(
                        &advertisement.node_id,
                        &advertisement.operator_id,
                        &advertisement.operator_cluster_id,
                        &advertisement.reputation_snapshot_id,
                        now,
                    ),
                )
            })
            .collect()
    }

    #[test]
    fn discovery_is_constraint_filtered_and_price_ordered() {
        let now = Utc::now();
        let advertisements = vec![
            advertisement("expensive", 150, now),
            advertisement("cheap", 50, now),
        ];
        let reputations = reputation_map(&advertisements, now);
        let references = advertisements.iter().collect::<Vec<_>>();
        let result = discover_services(
            &discovery(now),
            &references,
            &reputations,
            now,
            "signature".into(),
            &AcceptTestProofs,
        )
        .unwrap();
        assert_eq!(
            result.advertisement_ids[0],
            advertisements[1].advertisement_id
        );
        assert_eq!(result.advertisement_ids.len(), 2);
    }

    #[test]
    fn order_match_reserves_compatible_capacity_without_exceeding_budget() {
        let now = Utc::now();
        let mut incompatible = advertisement("incompatible", 1, now);
        incompatible.capabilities[0].price.amount.asset = "BTC".into();
        incompatible.capabilities[0].p50_latency_ms = 5;
        incompatible.capabilities[0].p95_latency_ms = 10;
        incompatible.advertisement_id = expected_advertisement_id(&incompatible).unwrap();
        let compatible = advertisement("compatible", 50, now);
        let compatible_id = compatible.advertisement_id.clone();
        let advertisements = vec![incompatible, compatible];
        let reputations = reputation_map(&advertisements, now);
        let references = advertisements.iter().collect::<Vec<_>>();
        let order = ServiceOrder {
            order_id: ServiceOrderId::derive(&"order").unwrap(),
            job_id: xlemma_core::JobId::derive(&"job").unwrap(),
            requester: "did:key:researcher".into(),
            service: NodeServiceKind::OfficialVerification,
            required_role: NodeRole::OfficialKernelChecker,
            required_checker_families: BTreeSet::from([xlemma_core::CheckerFamily::LeanKernel]),
            theory_id: None,
            domains: BTreeSet::from(["formal-mathematics".into()]),
            quantity_units: 250,
            maximum_total_price: Amount::new(130, "USDC", 6),
            maximum_p95_latency_ms: Some(2_000),
            reputation_requirements: ReputationRequirements::default(),
            excluded_operator_clusters: BTreeSet::new(),
            delivery_deadline: now + Duration::minutes(30),
            terms_root: "blake3:order-terms".into(),
            created_at: now,
            expires_at: now + Duration::minutes(5),
            signature: "signature".into(),
        };
        let matched = match_service_order(
            &order,
            &references,
            &reputations,
            now,
            now + Duration::minutes(1),
            now + Duration::minutes(10),
            "signature".into(),
            &AcceptTestProofs,
        )
        .unwrap();
        assert_eq!(matched.agreed_price.units, 125);
        assert_eq!(matched.reserved_units, 250);
        assert_eq!(matched.advertisement_id, compatible_id);

        let mut book = ServiceOrderBook::default();
        for advertisement in advertisements {
            book.publish_advertisement(advertisement, now, &AcceptTestProofs)
                .unwrap();
        }
        book.submit_order(order, now, &AcceptTestProofs).unwrap();
        book.record_match(matched.clone(), &AcceptTestProofs)
            .unwrap();
        assert!(matches!(
            book.record_match(matched, &AcceptTestProofs),
            Err(MarketplaceError::OrderAlreadyMatched)
        ));
    }

    #[test]
    fn order_book_preserves_superseded_advertisements() {
        let now = Utc::now();
        let first = advertisement("node", 50, now);
        let mut second = first.clone();
        second.sequence = 2;
        second.supersedes = Some(first.advertisement_id.clone());
        second.capabilities[0].available_units = 7_000;
        second.advertisement_id = expected_advertisement_id(&second).unwrap();

        let mut book = ServiceOrderBook::default();
        book.publish_advertisement(first.clone(), now, &AcceptTestProofs)
            .unwrap();
        book.publish_advertisement(second.clone(), now, &AcceptTestProofs)
            .unwrap();
        assert_eq!(book.advertisements.len(), 2);
        assert_eq!(
            book.active_advertisements(now)[0].advertisement_id,
            second.advertisement_id
        );
    }

    #[test]
    fn published_advertisement_vector_has_canonical_identity() {
        let advertisement: NodeServiceAdvertisement = serde_json::from_str(include_str!(
            "../../../examples/node-network/advertisement.json"
        ))
        .unwrap();
        let observed_at = "2026-09-03T12:00:00Z".parse().unwrap();
        validate_advertisement(&advertisement, observed_at).unwrap();
        assert_eq!(
            advertisement.advertisement_id,
            advertisement.derive_advertisement_id().unwrap()
        );
    }
}
