use cw_storage_plus::Item;
use schemars::Set;

pub const LOCKED_ITEMS: Item<Set<String>> = Item::new("locked_item");
