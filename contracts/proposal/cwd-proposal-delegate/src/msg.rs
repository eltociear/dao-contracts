use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty};

use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Delegate {
        delegate: String,
        msgs: Vec<CosmosMsg<Empty>>,
        expiration: Option<Expiration>,

        policy_revocable: bool,
        policy_allow_retry_on_failure: bool,
    },
    /// Fails if delegation is non-revocable
    RemoveDelegation { delegation_id: u64 },
    /// Only delegate
    Execute { delegation_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
