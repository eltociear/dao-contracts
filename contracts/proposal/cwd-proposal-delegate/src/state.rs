use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

#[cw_serde]
pub struct Delegation {
    pub delegate: Addr,
    pub msgs: Vec<CosmosMsg<Empty>>,
    pub expiration: Option<Expiration>,

    pub policy_revocable: bool,
    pub policy_allow_retry_on_failure: bool,
}

pub const DELEGATIONS: Map<u64, Delegation> = Map::new("delegations");
pub const DELEGATION_COUNT: Item<u64> = Item::new("delegation_count");

pub const CONFIG: Item<Config> = Item::new("config");
