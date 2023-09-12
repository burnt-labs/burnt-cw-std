use cosmwasm_std::{Coin, Timestamp, Uint64};
use cw721_base::MintMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::PrimarySale;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub sale: Option<CreatePrimarySale>,
}

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

impl From<CreatePrimarySale> for PrimarySale {
    fn from(val: CreatePrimarySale) -> Self {
        PrimarySale {
            total_supply: val.total_supply,
            tokens_minted: Uint64::from(0_u8),
            start_time: Timestamp::from_seconds(val.start_time.u64()),
            end_time: Timestamp::from_seconds(val.end_time.u64()),
            price: val.price,
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
    ClaimItem(MintMsg<T>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns all currently listed tokens
    PrimarySales {},
    ActivePrimarySale {},
}
