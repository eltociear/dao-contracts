use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Decimal as StdDecimal, Uint128};

use crate::abc::{CommonsPhase, CommonsPhaseConfig, CurveType, MinMax, ReserveToken, SupplyToken};

#[cw_serde]
pub struct InstantiateMsg {
    /// The code id of the cw-tokenfactory-issuer contract
    pub token_issuer_code_id: u64,

    /// Supply token information
    pub supply: SupplyToken,

    /// Reserve token information
    pub reserve: ReserveToken,

    /// Curve type for this contract
    pub curve_type: CurveType,

    /// Hatch configuration information
    pub phase_config: CommonsPhaseConfig,

    /// TODO different ways of doing this, for example DAO members?
    /// Using a whitelist contract? Merkle tree?
    /// Hatcher allowlist
    pub hatcher_allowlist: Option<Vec<String>>,
}

/// Update the phase configurations.
/// These can only be called by the owner.
#[cw_serde]
pub enum UpdatePhaseConfigMsg {
    /// Update the hatch phase configuration
    Hatch {
        contribution_limits: Option<MinMax>,
        exit_tax: Option<StdDecimal>,
        // TODO what is the minimum used for?
        initial_raise: Option<MinMax>,
        initial_allocation_ratio: Option<StdDecimal>,
    },
    /// Update the open phase configuration.
    Open {
        exit_tax: Option<StdDecimal>,
        allocation_percentage: Option<StdDecimal>,
    },
    /// Update the closed phase configuration.
    /// TODO Set the curve type to be used on close?
    Closed {},
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Buy will attempt to purchase as many supply tokens as possible.
    /// You must send only reserve tokens in that message
    Buy {},
    /// Burn is a base message to destroy tokens forever
    Burn {},
    /// Donate will add reserve tokens to the funding pool
    Donate {},
    /// Sets (or unsets if set to None) the maximum supply
    SetMaxSupply {
        /// The maximum supply able to be minted.
        max_supply: Option<Uint128>,
    },
    /// Updates the curve type used for pricing tokens.
    /// Only callable by owner.
    /// TODO think about other potential limitations on this.
    UpdateCurve { curve_type: CurveType },
    /// Update the hatch phase allowlist.
    /// This can only be called by the owner.
    UpdateHatchAllowlist {
        /// Addresses to be added.
        to_add: Vec<String>,
        /// Addresses to be removed.
        to_remove: Vec<String>,
    },
    /// Update the configuration of a certain phase.
    /// This can only be called by the owner.
    UpdatePhaseConfig(UpdatePhaseConfigMsg),
    /// Closing the bonding curve means no more buys are enabled and exit tax is set
    /// to zero.
    /// For example, this could be used in the event of a project shutting down.
    Close {},
}

// TODO Price queries:
// - Price to buy a certain amount?
// - What can be bought for a certain amount?
#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the reserve and supply quantities, as well as the spot price to buy 1 token
    /// Returns [`CurveInfoResponse`]
    #[returns(CurveInfoResponse)]
    CurveInfo {},
    /// Returns the current phase configuration
    /// Returns [`CommonsPhaseConfigResponse`]
    #[returns(CommonsPhaseConfigResponse)]
    PhaseConfig {},
    /// Returns a list of the donors and their donations
    /// Returns [`DonationsResponse`]
    #[returns(DonationsResponse)]
    Donations {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// List the hatchers and their contributions
    /// Returns [`HatchersResponse`]
    #[returns(HatchersResponse)]
    Hatchers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns Token Factory Denom for the supply
    #[returns(DenomResponse)]
    Denom {},
    /// Returns the address of the cw-tokenfactory-issuer contract
    #[returns(::cosmwasm_std::Addr)]
    TokenContract {},
}

#[cw_serde]
pub struct CurveInfoResponse {
    /// How many reserve tokens have been received
    pub reserve: Uint128,
    /// How many supply tokens have been issued
    pub supply: Uint128,
    /// The amount of tokens in the funding pool
    pub funding: Uint128,
    /// Current spot price of the token
    pub spot_price: Decimal,
    /// Current reserve denom
    pub reserve_denom: String,
}

#[cw_serde]
pub struct DenomResponse {
    pub denom: String,
}

#[cw_serde]
pub struct HatcherAllowlistResponse {
    /// Hatcher allowlist
    pub allowlist: Option<Vec<Addr>>,
}

#[cw_serde]
pub struct CommonsPhaseConfigResponse {
    /// The phase configuration
    pub phase_config: CommonsPhaseConfig,

    /// Current phase
    pub phase: CommonsPhase,
}

#[cw_serde]
pub struct DonationsResponse {
    /// The donators mapped to their donation in the reserve token
    pub donations: Vec<(Addr, Uint128)>,
}

#[cw_serde]
pub struct HatchersResponse {
    /// The hatchers mapped to their contribution in the reserve token
    pub hatchers: Vec<(Addr, Uint128)>,
}

#[cw_serde]
pub struct MigrateMsg {}
