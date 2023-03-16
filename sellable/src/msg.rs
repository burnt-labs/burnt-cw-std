use cosmwasm_std::Coin;
use cw721_base::state::TokenInfo;
use schemars::{JsonSchema, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub tokens: Map<String, Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp<T> {
    ListedTokens(Vec<(String, Coin, TokenInfo<T>)>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sellable specific functions

    /// Lists the NFT at the given price
    List {
        listings: Map<String, Coin>,
    },

    /// Delist a listed NFT
    Delist {
        token_id: String,
    },
    /// Purchases the cheapest listed NFT. The value passed along with the
    /// transaction will act as the upper bound for the purchase price.
    Buy {},

    BuyToken {
        token_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns all currently listed tokens
    ListedTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
