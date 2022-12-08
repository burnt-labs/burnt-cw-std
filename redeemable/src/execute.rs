use crate::{errors::ContractError, state::LOCKED_ITEMS, Redeemable};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_storage_plus::Item;
use schemars::Set;

impl<'a> Redeemable<'a> {
    pub fn new (item: Item<'a, Set<String>>) -> Self {
        Self { locked_items: item }
    }

    pub fn redeem_item(
        &mut self,
        deps: &mut DepsMut,
        _env: Env,
        _info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let mut locked_tokens = LOCKED_ITEMS.load(deps.storage)?;
        locked_tokens.insert(token_id);
        self.locked_items.save(deps.storage, &locked_tokens)?;
        Ok(Response::new().add_attribute("method", "lock ticket"))
    }
}
