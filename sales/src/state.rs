use cosmwasm_std::{Coin, Timestamp, Uint64};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct PrimarySale {
    pub total_supply: Uint64,
    pub tokens_minted: Uint64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub price: Coin,
    pub disabled: bool, // is sale still on ?
}

pub const PRIMARY_SALES: Item<Vec<PrimarySale>> = Item::new("primary_sales");
