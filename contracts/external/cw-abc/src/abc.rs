use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Decimal as StdDecimal, Uint128};
use dao_interface::token::NewDenomMetadata;

use crate::curves::{decimal, Constant, Curve, DecimalPlaces, Linear, SquareRoot};
use crate::ContractError;

#[cw_serde]
pub struct SupplyToken {
    /// The denom to create for the supply token
    pub subdenom: String,
    /// Metadata for the supply token to create
    pub metadata: Option<NewDenomMetadata>,
    /// Number of decimal places for the supply token, needed for proper curve math.
    /// Default for token factory is 6
    pub decimals: u8,
    // TODO max supply
    // pub max_supply: Uint128,
}

#[cw_serde]
pub struct ReserveToken {
    /// Reserve token denom (only support native for now)
    pub denom: String,
    /// Number of decimal places for the reserve token, needed for proper curve math.
    /// Same format as decimals above, eg. if it is uatom, where 1 unit is 10^-6 ATOM, use 6 here
    pub decimals: u8,
}

/// Struct for minimium and maximum values
#[cw_serde]
pub struct MinMax {
    pub min: Uint128,
    pub max: Uint128,
}

#[cw_serde]
pub struct HatchConfig {
    /// The minimum and maximum contribution amounts (min, max) in the reserve token
    pub contribution_limits: MinMax,
    /// The initial raise range (min, max) in the reserve token
    pub initial_raise: MinMax,
    /// The initial price (p0) per reserve token
    /// TODO: initial price is not implemented yet
    /// TODO: do we need this or is it just calculated?
    pub initial_price: Uint128,
    /// The initial allocation (θ), percentage of the initial raise allocated to the Funding Pool
    pub initial_allocation_ratio: StdDecimal,
    /// Exit tax for the hatch phase
    pub exit_tax: StdDecimal,
}

impl HatchConfig {
    /// Validate the hatch config
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure!(
            self.initial_raise.min < self.initial_raise.max,
            ContractError::HatchPhaseConfigError(
                "Initial raise minimum value must be less than maximum value.".to_string()
            )
        );

        ensure!(
            self.contribution_limits.max <= self.initial_raise.max,
            ContractError::HatchPhaseConfigError(
                "Max contribution limit cannot be greater than the maximum initial raise."
                    .to_string()
            )
        );

        ensure!(
            !self.initial_price.is_zero(),
            ContractError::HatchPhaseConfigError(
                "Initial price must be greater than zero.".to_string()
            )
        );

        // TODO: define better values
        // Q: is zero valid for initial allocation value? Isn't the whole point of the
        // hatch phase to initialize the DAO treasury?
        ensure!(
            self.initial_allocation_ratio <= StdDecimal::percent(100u64),
            ContractError::HatchPhaseConfigError(
                "Initial allocation percentage must be between 0 and 100.".to_string()
            )
        );

        ensure!(
            self.exit_tax <= StdDecimal::percent(100u64),
            ContractError::HatchPhaseConfigError(
                "Exit taxation percentage must be less than or equal to 100.".to_string()
            )
        );

        Ok(())
    }
}

#[cw_serde]
pub struct OpenConfig {
    /// Percentage of capital put into the Reserve Pool during the Open phase
    /// when buying from the curve.
    pub allocation_percentage: StdDecimal,
    /// Exit taxation ratio
    pub exit_tax: StdDecimal,
}

impl OpenConfig {
    /// Validate the open config
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure!(
            self.allocation_percentage <= StdDecimal::percent(100u64),
            ContractError::OpenPhaseConfigError(
                "Reserve percentage must be between 0 and 100.".to_string()
            )
        );

        ensure!(
            self.exit_tax <= StdDecimal::percent(100u64),
            ContractError::OpenPhaseConfigError(
                "Exit taxation percentage must be between 0 and 100.".to_string()
            )
        );

        Ok(())
    }
}

#[cw_serde]
pub struct ClosedConfig {}

impl ClosedConfig {
    /// Validate the closed config
    pub fn validate(&self) -> Result<(), ContractError> {
        Ok(())
    }
}

#[cw_serde]
pub struct CommonsPhaseConfig {
    /// The Hatch phase where initial contributors (Hatchers) participate in a hatch sale.
    pub hatch: HatchConfig,
    /// TODO Vest tokens after hatch phase
    /// The Vesting phase where tokens minted during the Hatch phase are locked (burning is disabled) to combat early speculation/arbitrage.
    /// pub vesting: VestingConfig,
    /// The Open phase where anyone can mint tokens by contributing the reserve token into the curve and becoming members of the Commons.
    pub open: OpenConfig,
    /// The Closed phase where the Commons is closed to new members.
    pub closed: ClosedConfig,
}

#[cw_serde]
pub enum CommonsPhase {
    Hatch,
    Open,
    Closed,
}

impl CommonsPhase {
    pub fn expect_hatch(&self) -> Result<(), ContractError> {
        ensure!(
            matches!(self, CommonsPhase::Hatch),
            ContractError::InvalidPhase {
                expected: "Hatch".to_string(),
                actual: format!("{:?}", self)
            }
        );
        Ok(())
    }

    pub fn expect_open(&self) -> Result<(), ContractError> {
        ensure!(
            matches!(self, CommonsPhase::Open),
            ContractError::InvalidPhase {
                expected: "Open".to_string(),
                actual: format!("{:?}", self)
            }
        );
        Ok(())
    }

    pub fn expect_closed(&self) -> Result<(), ContractError> {
        ensure!(
            matches!(self, CommonsPhase::Closed),
            ContractError::InvalidPhase {
                expected: "Closed".to_string(),
                actual: format!("{:?}", self)
            }
        );
        Ok(())
    }
}

impl CommonsPhaseConfig {
    /// Validate that the commons configuration is valid
    pub fn validate(&self) -> Result<(), ContractError> {
        self.hatch.validate()?;
        self.open.validate()?;
        self.closed.validate()?;

        Ok(())
    }
}

pub type CurveFn = Box<dyn Fn(DecimalPlaces) -> Box<dyn Curve>>;

// TODO add S-curve and taylor series
#[cw_serde]
pub enum CurveType {
    /// Constant always returns `value * 10^-scale` as spot price
    Constant { value: Uint128, scale: u32 },
    /// Linear returns `slope * 10^-scale * supply` as spot price
    Linear { slope: Uint128, scale: u32 },
    /// SquareRoot returns `slope * 10^-scale * supply^0.5` as spot price
    SquareRoot { slope: Uint128, scale: u32 },
}

impl CurveType {
    pub fn to_curve_fn(&self) -> CurveFn {
        match self.clone() {
            CurveType::Constant { value, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Constant::new(decimal(value, scale), places))
                };
                Box::new(calc)
            }
            CurveType::Linear { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Linear::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
            CurveType::SquareRoot { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(SquareRoot::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn validate_contribution_limit_not_gt_initial_raise() {
        let phase_config = CommonsPhaseConfig {
            hatch: HatchConfig {
                contribution_limits: MinMax {
                    min: Uint128::one(),
                    max: Uint128::MAX,
                },
                initial_raise: MinMax {
                    min: Uint128::one(),
                    max: Uint128::from(1000000u128),
                },
                initial_price: Uint128::one(),
                initial_allocation_ratio: StdDecimal::percent(10u64),
                exit_tax: StdDecimal::percent(10u64),
            },
            open: OpenConfig {
                allocation_percentage: StdDecimal::percent(10u64),
                exit_tax: StdDecimal::percent(10u64),
            },
            closed: ClosedConfig {},
        };
        let err = phase_config.validate().unwrap_err();
        assert_eq!(
            err,
            ContractError::HatchPhaseConfigError(
                "Max contribution limit cannot be greater than the maximum initial raise."
                    .to_string()
            )
        )
    }
}
