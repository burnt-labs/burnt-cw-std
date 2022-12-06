use crate::{errors::ContractError, state::LOCKED_ITEMS, Redeemable};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

impl Redeemable<'_> {
    pub fn lock_token(
        &mut self,
        deps: &mut DepsMut,
        _env: Env,
        _info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let mut locked_tokens = LOCKED_ITEMS.load(deps.storage)?;
        locked_tokens.insert(token_id);
        Ok(Response::new().add_attribute("method", "lock ticket"))
    }
}
