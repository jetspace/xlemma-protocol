//! Market, commons, and assurance funding rails backed by external settlement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{
    Amount, FundingReceiptId, IdError, MoneyError, PolicyId, ReceiptId, ResearcherId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundingRail {
    Market,
    Commons,
    Assurance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundingPurpose {
    Bounty,
    FormalizationContract,
    ProofApi,
    CommercialLicense,
    CertifiedImplementation,
    ReservedCompute,
    MaintenanceAgreement,
    FoundationalResearch,
    FormalLibrary,
    NegativeResult,
    BenchmarkSuite,
    ProofTactic,
    OpenDataset,
    Exposition,
    RetrospectiveImpact,
    VerificationBondReserve,
    ChallengeReserve,
    RevalidationReserve,
    CertificateWarranty,
    RelianceInsurance,
}

impl FundingPurpose {
    pub const fn rail(self) -> FundingRail {
        match self {
            Self::Bounty
            | Self::FormalizationContract
            | Self::ProofApi
            | Self::CommercialLicense
            | Self::CertifiedImplementation
            | Self::ReservedCompute
            | Self::MaintenanceAgreement => FundingRail::Market,
            Self::FoundationalResearch
            | Self::FormalLibrary
            | Self::NegativeResult
            | Self::BenchmarkSuite
            | Self::ProofTactic
            | Self::OpenDataset
            | Self::Exposition
            | Self::RetrospectiveImpact => FundingRail::Commons,
            Self::VerificationBondReserve
            | Self::ChallengeReserve
            | Self::RevalidationReserve
            | Self::CertificateWarranty
            | Self::RelianceInsurance => FundingRail::Assurance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundingReceipt {
    pub funding_receipt_id: FundingReceiptId,
    pub rail: FundingRail,
    pub purpose: FundingPurpose,
    pub payer: String,
    pub beneficiary_researcher_id: Option<ResearcherId>,
    pub destination_vault: String,
    pub settled_amount: Amount,
    pub settlement_receipt_id: ReceiptId,
    pub external_value_evidence_root: String,
    pub policy_id: PolicyId,
    pub related_party: bool,
    pub settled_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Serialize)]
struct FundingIdentity<'a> {
    rail: FundingRail,
    purpose: FundingPurpose,
    payer: &'a str,
    beneficiary_researcher_id: &'a Option<ResearcherId>,
    destination_vault: &'a str,
    settled_amount: &'a Amount,
    settlement_receipt_id: &'a ReceiptId,
    external_value_evidence_root: &'a str,
    policy_id: &'a PolicyId,
    related_party: bool,
    settled_at: DateTime<Utc>,
}

impl FundingReceipt {
    pub fn derive_funding_receipt_id(&self) -> Result<FundingReceiptId, IdError> {
        FundingReceiptId::derive(&FundingIdentity {
            rail: self.rail,
            purpose: self.purpose,
            payer: &self.payer,
            beneficiary_researcher_id: &self.beneficiary_researcher_id,
            destination_vault: &self.destination_vault,
            settled_amount: &self.settled_amount,
            settlement_receipt_id: &self.settlement_receipt_id,
            external_value_evidence_root: &self.external_value_evidence_root,
            policy_id: &self.policy_id,
            related_party: self.related_party,
            settled_at: self.settled_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), FundingError> {
        self.funding_receipt_id.validate()?;
        self.settlement_receipt_id.validate()?;
        self.policy_id.validate()?;
        if let Some(researcher_id) = &self.beneficiary_researcher_id {
            researcher_id.validate()?;
        }
        if self.funding_receipt_id != self.derive_funding_receipt_id()? {
            return Err(FundingError::IdentityMismatch);
        }
        if self.rail != self.purpose.rail() {
            return Err(FundingError::RailPurposeMismatch);
        }
        if self.payer.trim().is_empty()
            || self.destination_vault.trim().is_empty()
            || self.settled_amount.units == 0
            || self.settled_amount.asset.trim().is_empty()
            || self.external_value_evidence_root.trim().is_empty()
            || self.signature.trim().is_empty()
        {
            return Err(FundingError::UnsettledOrMissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolFeeRailAllocation {
    pub total_fee: Amount,
    pub commons: Amount,
    pub assurance: Amount,
    pub operations: Amount,
    pub rounding_remainder: Amount,
}

pub fn allocate_protocol_fee(
    total_fee: &Amount,
    commons_bps: u16,
    assurance_bps: u16,
    operations_bps: u16,
) -> Result<ProtocolFeeRailAllocation, FundingError> {
    if u32::from(commons_bps) + u32::from(assurance_bps) + u32::from(operations_bps) != 10_000
        || commons_bps == 0
        || assurance_bps == 0
        || total_fee.units == 0
        || total_fee.asset.trim().is_empty()
    {
        return Err(FundingError::InvalidFeePolicy);
    }
    let commons = total_fee.mul_bps(commons_bps)?;
    let assurance = total_fee.mul_bps(assurance_bps)?;
    let operations = total_fee.mul_bps(operations_bps)?;
    let allocated = commons
        .units
        .checked_add(assurance.units)
        .and_then(|value| value.checked_add(operations.units))
        .ok_or(FundingError::Overflow)?;
    Ok(ProtocolFeeRailAllocation {
        total_fee: total_fee.clone(),
        commons,
        assurance,
        operations,
        rounding_remainder: Amount::new(
            total_fee.units - allocated,
            total_fee.asset.clone(),
            total_fee.decimals,
        ),
    })
}

#[derive(Debug, Error)]
pub enum FundingError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error("funding receipt content identity does not match")]
    IdentityMismatch,
    #[error("funding purpose is assigned to the wrong market, commons, or assurance rail")]
    RailPurposeMismatch,
    #[error("funding lacks settled external value or required evidence")]
    UnsettledOrMissingEvidence,
    #[error("protocol fee policy must conserve 10,000 bps and fund commons and assurance")]
    InvalidFeePolicy,
    #[error("checked funding arithmetic overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_results_are_commons_not_market_funding() {
        assert_eq!(FundingPurpose::NegativeResult.rail(), FundingRail::Commons);
        assert_eq!(
            FundingPurpose::RelianceInsurance.rail(),
            FundingRail::Assurance
        );
        assert_eq!(FundingPurpose::ProofApi.rail(), FundingRail::Market);
    }

    #[test]
    fn protocol_fees_conserve_value_and_fund_all_infrastructure() {
        let allocation =
            allocate_protocol_fee(&Amount::new(101, "USDC", 6), 4_000, 3_000, 3_000).unwrap();
        assert_eq!(allocation.commons.units, 40);
        assert_eq!(allocation.assurance.units, 30);
        assert_eq!(allocation.operations.units, 30);
        assert_eq!(allocation.rounding_remainder.units, 1);
    }

    #[test]
    fn inflation_without_settlement_cannot_be_funding() {
        let mut receipt = FundingReceipt {
            funding_receipt_id: FundingReceiptId::derive(&"placeholder").unwrap(),
            rail: FundingRail::Commons,
            purpose: FundingPurpose::FoundationalResearch,
            payer: "grant-maker".into(),
            beneficiary_researcher_id: None,
            destination_vault: "vault:commons".into(),
            settled_amount: Amount::new(0, "USDC", 6),
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            external_value_evidence_root: "blake3:bank-settlement".into(),
            policy_id: PolicyId::derive(&"commons").unwrap(),
            related_party: false,
            settled_at: Utc::now(),
            signature: "signature".into(),
        };
        receipt.funding_receipt_id = receipt.derive_funding_receipt_id().unwrap();
        assert!(matches!(
            receipt.validate_integrity(),
            Err(FundingError::UnsettledOrMissingEvidence)
        ));
    }
}
