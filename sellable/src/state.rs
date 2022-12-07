use cosmwasm_std::Uint64;
use cw_storage_plus::Item;
use schemars::Map;

pub const LISTED_TOKENS: Item<Map<String, Uint64>> = Item::new("listed_tokens");
