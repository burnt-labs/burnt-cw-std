use cosmwasm_std::Uint64;
use cw_storage_plus::Map;

pub const LISTED_TOKENS: Map<&str, Uint64> = Map::new("listed_tokens");
