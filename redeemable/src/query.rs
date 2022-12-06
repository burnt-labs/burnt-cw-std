use cosmwasm_std::{Deps, StdResult};

use crate::{state::LOCKED_ITEMS, Redeemable};

impl Redeemable<'_> {
    pub fn is_redeemed(&self, deps: &Deps, token_id: String) -> StdResult<bool> {
        let locked_items = LOCKED_ITEMS.load(deps.storage)?;
        locked_items.contains(&token_id);
        Ok(locked_items.contains(&token_id))
    }
    pub fn is_locked(&self, deps: &Deps, token_id: String) -> StdResult<bool> {
        let locked_items = LOCKED_ITEMS.load(deps.storage)?;
        locked_items.contains(&token_id);
        Ok(locked_items.contains(&token_id))
    }
}
