use cosmwasm_std::{Deps, StdResult};

use crate::Redeemable;

impl Redeemable<'_> {
    pub fn is_redeemed(&self, deps: &Deps, token_id: String) -> StdResult<bool> {
        let locked_items = self.locked_items.load(deps.storage)?;
        Ok(locked_items.contains(&token_id))
    }
}
