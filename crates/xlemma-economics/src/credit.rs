use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{Amount, MoneyError, ResearcherId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackedCreditLedger {
    pub researcher_id: ResearcherId,
    pub settlement_asset: String,
    pub credit_asset: String,
    pub decimals: u8,
    pub backing_units: u128,
    pub outstanding_credit_units: u128,
    pub locked_credit_units: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditAuthorization {
    pub authorization_id: String,
    pub maximum_units: u128,
    pub settled_units: u128,
    pub closed: bool,
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
        self.locked_credit_units = next_locked;
        Ok(CreditAuthorization {
            authorization_id: authorization_id.into(),
            maximum_units: maximum.units,
            settled_units: 0,
            closed: false,
        })
    }

    /// Burns only actual credits consumed, releases corresponding stable
    /// backing to service nodes, and unlocks the unused authorization.
    pub fn settle(
        &mut self,
        authorization: &mut CreditAuthorization,
        actual: &Amount,
    ) -> Result<Amount, CreditError> {
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
        authorization.settled_units = actual.units;
        authorization.closed = true;

        Ok(Amount::new(
            actual.units,
            self.settlement_asset.clone(),
            self.decimals,
        ))
    }


    /// Releases an unused authorization without burning credits or backing.
    pub fn cancel(
        &mut self,
        authorization: &mut CreditAuthorization,
    ) -> Result<(), CreditError> {
        if authorization.closed {
            return Err(CreditError::AuthorizationClosed);
        }
        let next_locked = self
            .locked_credit_units
            .checked_sub(authorization.maximum_units)
            .ok_or(CreditError::UnderCollateralized)?;
        self.locked_credit_units = next_locked;
        authorization.settled_units = 0;
        authorization.closed = true;
        self.ensure_solvent()
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
        let mut auth = ledger
            .authorize(
                "proof-job",
                &Amount::new(125_000_000, "R-IAN", 6),
            )
            .unwrap();
        let payout = ledger
            .settle(&mut auth, &Amount::new(40_000_000, "R-IAN", 6))
            .unwrap();
        assert_eq!(payout.units, 40_000_000);
        assert_eq!(ledger.backing_units, 960_000_000);
        assert_eq!(ledger.outstanding_credit_units, 960_000_000);
        assert_eq!(ledger.locked_credit_units, 0);
        ledger.ensure_solvent().unwrap();
    }
}
