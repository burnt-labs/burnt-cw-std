use cw_storage_plus::{Item, Map};
use cosmwasm_std::Addr;

pub const ALLOWED_ADDRS: Map<Addr, bool> = Map::new("allowed_addrs");
pub const ENABLED: Item<bool> = Item::new("enabled");
