use cosmwasm_std::{Coin, Uint64, Timestamp};
use cw721_base::MintMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::PrimarySale;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    PrimarySales(Vec<PrimarySale>),
    ActivePrimarySale(Option<PrimarySale>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CreatePrimarySale {
    pub total_supply: Uint64,
    pub start_time: Uint64, // timestamp in seconds
    pub end_time: Uint64,   // timestamp in seconds
    pub price: Vec<Coin>,
}

impl Into<PrimarySale> for CreatePrimarySale {
    fn into(self) -> PrimarySale {
        PrimarySale {
            total_supply: self.total_supply,
            tokens_minted: Uint64::from(0 as u8),
            start_time: Timestamp::from_seconds(self.start_time.u64()),
            end_time: Timestamp::from_seconds(self.end_time.u64()),
            price: self.price,
            disabled: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    PrimarySale(CreatePrimarySale),
    HaltSale {},
    BuyItem(MintMsg<T>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns all currently listed tokens
    PrimarySales {},
    ActivePrimarySale {},
}
