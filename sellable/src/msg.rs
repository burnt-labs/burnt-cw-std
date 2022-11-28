use cosmwasm_std::{Binary, Uint64};
use schemars::{JsonSchema, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    Result(Binary),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sellable specific functions

    /// Lists the NFT at the given price
    List { listings: Map<String, Uint64> },

    /// Purchases the cheapest listed NFT. The value passed along with the
    /// transaction will act as the upper bound for the purchase price.
    Buy {},

    /// Mark ticket has redeemed
    RedeemTicket { address: String, ticket_id: String },
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

pub trait SellableTrait {
    fn get_redeemed(&self) -> bool;
    fn get_locked(&self) -> bool;
    fn set_list_price(&mut self, price: Uint64) -> bool;
    fn get_list_price(&self) -> Uint64;
}
