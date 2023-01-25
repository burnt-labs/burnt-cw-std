use cosmwasm_std::{Coin, Uint64};
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
pub enum ExecuteMsg<T> {
    PrimarySale {
        total_supply: Uint64,
        start_time: Uint64, // timestamp in seconds
        end_time: Uint64,   // timestamp in seconds
        price: Vec<Coin>,
    },
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
