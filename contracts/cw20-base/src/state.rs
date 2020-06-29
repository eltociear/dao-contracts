use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Meta {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub mint: Option<MinterData>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MinterData {
    pub minter: CanonicalAddr,
    /// cap is how many more tokens can be issued by the minter
    pub cap: Option<Uint128>,
}

impl Meta {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }
}

const META_KEY: &[u8] = b"meta";
const PREFIX_BALANCE: &[u8] = b"balance";

// meta is the token definition as well as the total_supply
pub fn meta<S: Storage>(storage: &mut S) -> Singleton<S, Meta> {
    singleton(storage, META_KEY)
}

pub fn meta_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, Meta> {
    singleton_read(storage, META_KEY)
}

/// balances are state of the erc20 tokens
pub fn balances<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(PREFIX_BALANCE, storage)
}

pub fn balances_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(PREFIX_BALANCE, storage)
}
