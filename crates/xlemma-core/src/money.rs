use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    /// Integer amount in the asset's smallest declared unit.
    pub units: u128,
    /// Stable protocol symbol or chain asset identifier.
    pub asset: String,
    pub decimals: u8,
}

#[derive(Debug, Error)]
pub enum MoneyError {
    #[error("asset mismatch: {left} != {right}")]
    AssetMismatch { left: String, right: String },
    #[error("decimal mismatch: {left} != {right}")]
    DecimalMismatch { left: u8, right: u8 },
    #[error("amount overflow")]
    Overflow,
    #[error("insufficient amount")]
    Insufficient,
}

impl Amount {
    pub fn new(units: u128, asset: impl Into<String>, decimals: u8) -> Self {
        Self {
            units,
            asset: asset.into(),
            decimals,
        }
    }

    pub fn zero_like(&self) -> Self {
        Self::new(0, self.asset.clone(), self.decimals)
    }

    pub fn checked_add(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_compatible(other)?;
        let units = self.units.checked_add(other.units).ok_or(MoneyError::Overflow)?;
        Ok(Self::new(units, self.asset.clone(), self.decimals))
    }

    pub fn checked_sub(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_compatible(other)?;
        let units = self
            .units
            .checked_sub(other.units)
            .ok_or(MoneyError::Insufficient)?;
        Ok(Self::new(units, self.asset.clone(), self.decimals))
    }

    pub fn mul_bps(&self, basis_points: u16) -> Result<Self, MoneyError> {
        let numerator = self
            .units
            .checked_mul(u128::from(basis_points))
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::new(
            numerator / 10_000,
            self.asset.clone(),
            self.decimals,
        ))
    }

    pub fn ensure_compatible(&self, other: &Self) -> Result<(), MoneyError> {
        if self.asset != other.asset {
            return Err(MoneyError::AssetMismatch {
                left: self.asset.clone(),
                right: other.asset.clone(),
            });
        }
        if self.decimals != other.decimals {
            return Err(MoneyError::DecimalMismatch {
                left: self.decimals,
                right: other.decimals,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incompatible_assets_cannot_be_added() {
        let usdc = Amount::new(100, "USDC", 6);
        let dai = Amount::new(100, "DAI", 18);
        assert!(matches!(
            usdc.checked_add(&dai),
            Err(MoneyError::AssetMismatch { .. })
        ));
    }

    #[test]
    fn basis_point_allocation_rounds_down_without_overallocation() {
        let amount = Amount::new(101, "USDC", 6);
        assert_eq!(amount.mul_bps(5_000).unwrap().units, 50);
    }
}
