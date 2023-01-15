use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Map};

pub const PEOPLE_WHO_SAID_FOUR: Map<Addr, Empty> = Map::new("people_who_said_four");
