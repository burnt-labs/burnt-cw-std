use cosmwasm_std::Coin;
use cw_storage_plus::Map;

pub const LISTED_TOKENS: Map<&str, Coin> = Map::new("listed_tokens");
