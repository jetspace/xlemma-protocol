use serde::Serialize;
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_core::{Amount, MoneyError, ResearcherId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BackedCreditLedger {
    researcher_id: ResearcherId,
    settlement_asset: String,
    credit_asset: String,
    decimals: u8,
    backing_units: u128,
    outstanding_credit_units: u128,
    locked_credit_units: u128,
    authorizations: BTreeMap<String, CreditAuthorization>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CreditAuthorization {
    authorization_id: String,
    maximum_units: u128,
    settled_units: u128,
    closed: bool,
}

impl CreditAuthorization {
    pub fn authorization_id(&self) -> &str {
        &self.authorization_id
    }

    pub fn maximum_units(&self) -> u128 {
        self.maximum_units
    }

    pub fn settled_units(&self) -> u128 {
        self.settled_units
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Error)]
pub enum CreditError {
    #[error("amount asset or decimals does not match the vault")]
    IncompatibleAmount,
    #[error("credit ledger would become under-collateralized")]
    UnderCollateralized,
    #[error("insufficient unlocked credits")]
    InsufficientUnlockedCredits,
    #[error("settlement exceeds authorized maximum")]
    ExceedsAuthorization,
    #[error("authorization is already closed")]
    AuthorizationClosed,
    #[error("authorization identifier is empty or already exists")]
    InvalidAuthorizationId,
    #[error("authorization does not exist")]
    UnknownAuthorization,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

impl BackedCreditLedger {
    pub fn new(
        researcher_id: ResearcherId,
        settlement_asset: impl Into<String>,
        credit_asset: impl Into<String>,
        decimals: u8,
    ) -> Self {
        Self {
            researcher_id,
            settlement_asset: settlement_asset.into(),
            credit_asset: credit_asset.into(),
            decimals,
            backing_units: 0,
            outstanding_credit_units: 0,
            locked_credit_units: 0,
            authorizations: BTreeMap::new(),
        }
    }

    pub fn deposit_and_mint(&mut self, deposit: &Amount) -> Result<Amount, CreditError> {
        self.ensure_settlement_amount(deposit)?;
        let next_backing = self
            .backing_units
            .checked_add(deposit.units)
            .ok_or(CreditError::UnderCollateralized)?;
        let next_outstanding = self
            .outstanding_credit_units
            .checked_add(deposit.units)
            .ok_or(CreditError::UnderCollateralized)?;
        if next_backing < next_outstanding {
            return Err(CreditError::UnderCollateralized);
        }
        self.backing_units = next_backing;
        self.outstanding_credit_units = next_outstanding;
        Ok(Amount::new(
            deposit.units,
            self.credit_asset.clone(),
            self.decimals,
        ))
    }

    pub fn authorize(
        &mut self,
        authorization_id: impl Into<String>,
        maximum: &Amount,
    ) -> Result<CreditAuthorization, CreditError> {
        self.ensure_credit_amount(maximum)?;
        let authorization_id = authorization_id.into();
        if authorization_id.trim().is_empty() || self.authorizations.contains_key(&authorization_id)
        {
            return Err(CreditError::InvalidAuthorizationId);
        }
        let unlocked = self
            .outstanding_credit_units
            .checked_sub(self.locked_credit_units)
            .ok_or(CreditError::UnderCollateralized)?;
        if maximum.units > unlocked {
            return Err(CreditError::InsufficientUnlockedCredits);
        }
        let next_locked = self
            .locked_credit_units
            .checked_add(maximum.units)
            .ok_or(CreditError::UnderCollateralized)?;
        if next_locked > self.outstanding_credit_units {
            return Err(CreditError::UnderCollateralized);
        }
        let authorization = CreditAuthorization {
            authorization_id: authorization_id.clone(),
            maximum_units: maximum.units,
            settled_units: 0,
            closed: false,
        };
        self.locked_credit_units = next_locked;
        self.authorizations
            .insert(authorization_id, authorization.clone());
        Ok(authorization)
    }

    /// Burns only actual credits consumed, releases corresponding stable
    /// backing to service nodes, and unlocks the unused authorization.
    pub fn settle(
        &mut self,
        authorization_id: &str,
        actual: &Amount,
    ) -> Result<Amount, CreditError> {
        let authorization = self
            .authorizations
            .get(authorization_id)
            .ok_or(CreditError::UnknownAuthorization)?;
        if authorization.closed {
            return Err(CreditError::AuthorizationClosed);
        }
        self.ensure_credit_amount(actual)?;
        if actual.units > authorization.maximum_units {
            return Err(CreditError::ExceedsAuthorization);
        }

        let next_locked = self
            .locked_credit_units
            .checked_sub(authorization.maximum_units)
            .ok_or(CreditError::UnderCollateralized)?;
        let next_outstanding = self
            .outstanding_credit_units
            .checked_sub(actual.units)
            .ok_or(CreditError::UnderCollateralized)?;
        let next_backing = self
            .backing_units
            .checked_sub(actual.units)
            .ok_or(CreditError::UnderCollateralized)?;
        if next_backing < next_outstanding || next_locked > next_outstanding {
            return Err(CreditError::UnderCollateralized);
        }

        self.locked_credit_units = next_locked;
        self.outstanding_credit_units = next_outstanding;
        self.backing_units = next_backing;
        let authorization = self
            .authorizations
            .get_mut(authorization_id)
            .ok_or(CreditError::UnknownAuthorization)?;
        authorization.settled_units = actual.units;
        authorization.closed = true;

        Ok(Amount::new(
            actual.units,
            self.settlement_asset.clone(),
            self.decimals,
        ))
    }

    /// Releases an unused authorization without burning credits or backing.
    pub fn cancel(&mut self, authorization_id: &str) -> Result<(), CreditError> {
        let authorization = self
            .authorizations
            .get(authorization_id)
            .ok_or(CreditError::UnknownAuthorization)?;
        if authorization.closed {
            return Err(CreditError::AuthorizationClosed);
        }
        let next_locked = self
            .locked_credit_units
            .checked_sub(authorization.maximum_units)
            .ok_or(CreditError::UnderCollateralized)?;
        self.locked_credit_units = next_locked;
        let authorization = self
            .authorizations
            .get_mut(authorization_id)
            .ok_or(CreditError::UnknownAuthorization)?;
        authorization.settled_units = 0;
        authorization.closed = true;
        self.ensure_solvent()
    }

    pub fn authorization(&self, authorization_id: &str) -> Option<&CreditAuthorization> {
        self.authorizations.get(authorization_id)
    }

    pub fn researcher_id(&self) -> &ResearcherId {
        &self.researcher_id
    }

    pub fn settlement_asset(&self) -> &str {
        &self.settlement_asset
    }

    pub fn credit_asset(&self) -> &str {
        &self.credit_asset
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn backing_units(&self) -> u128 {
        self.backing_units
    }

    pub fn outstanding_credit_units(&self) -> u128 {
        self.outstanding_credit_units
    }

    pub fn locked_credit_units(&self) -> u128 {
        self.locked_credit_units
    }

    pub fn reserve_ratio_bps(&self) -> u128 {
        if self.outstanding_credit_units == 0 {
            return u128::MAX;
        }
        self.backing_units.saturating_mul(10_000) / self.outstanding_credit_units
    }

    pub fn ensure_solvent(&self) -> Result<(), CreditError> {
        if self.backing_units < self.outstanding_credit_units
            || self.locked_credit_units > self.outstanding_credit_units
        {
            Err(CreditError::UnderCollateralized)
        } else {
            Ok(())
        }
    }

    fn ensure_settlement_amount(&self, amount: &Amount) -> Result<(), CreditError> {
        if amount.asset == self.settlement_asset && amount.decimals == self.decimals {
            Ok(())
        } else {
            Err(CreditError::IncompatibleAmount)
        }
    }

    fn ensure_credit_amount(&self, amount: &Amount) -> Result<(), CreditError> {
        if amount.asset == self.credit_asset && amount.decimals == self.decimals {
            Ok(())
        } else {
            Err(CreditError::IncompatibleAmount)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_remain_fully_backed_through_usage_settlement() {
        let researcher = ResearcherId::derive(&"researcher").unwrap();
        let mut ledger = BackedCreditLedger::new(researcher, "USDC", "R-IAN", 6);
        ledger
            .deposit_and_mint(&Amount::new(1_000_000_000, "USDC", 6))
            .unwrap();
        let auth = ledger
            .authorize("proof-job", &Amount::new(125_000_000, "R-IAN", 6))
            .unwrap();
        assert_eq!(auth.authorization_id(), "proof-job");
        let payout = ledger
            .settle("proof-job", &Amount::new(40_000_000, "R-IAN", 6))
            .unwrap();
        assert_eq!(payout.units, 40_000_000);
        assert_eq!(ledger.backing_units(), 960_000_000);
        assert_eq!(ledger.outstanding_credit_units(), 960_000_000);
        assert_eq!(ledger.locked_credit_units(), 0);
        ledger.ensure_solvent().unwrap();
    }

    #[test]
    fn forged_or_cloned_authorization_cannot_unlock_another_reservation() {
        let researcher = ResearcherId::derive(&"researcher").unwrap();
        let mut ledger = BackedCreditLedger::new(researcher, "USDC", "R-IAN", 6);
        ledger
            .deposit_and_mint(&Amount::new(100, "USDC", 6))
            .unwrap();
        let snapshot = ledger
            .authorize("job-a", &Amount::new(80, "R-IAN", 6))
            .unwrap();

        assert!(matches!(
            ledger.settle("job-b", &Amount::new(10, "R-IAN", 6)),
            Err(CreditError::UnknownAuthorization)
        ));
        assert_eq!(snapshot.maximum_units(), 80);
        assert_eq!(ledger.locked_credit_units(), 80);
        ledger.cancel("job-a").unwrap();
        assert!(ledger.authorization("job-a").unwrap().is_closed());
    }
}
