use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{Coin, CustomMsg, Deps, Env, MessageInfo};
use cw_storage_plus::Map;
use ownable::Ownable;
use redeemable::Redeemable;
use serde::{de::DeserializeOwned, Serialize};
use token::Tokens;

use crate::{errors::ContractError, RSellable, Sellable};

pub trait SellableModule<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn check_ownable(
        &self,
        deps: &Deps,
        _env: &Env,
        info: &MessageInfo,
        ownable: &Ownable,
    ) -> Result<(), ContractError>;

    fn check_redeemable(
        &self,
        deps: &Deps,
        _env: &Env,
        _info: &MessageInfo,
        token_id: &str,
        redeemable: &Redeemable,
    ) -> Result<(), ContractError>;

    fn check_is_allowed(
        &self,
        deps: &Deps,
        info: &MessageInfo,
    ) -> Result<(), ContractError>;

    fn get_token_module(&self) -> Rc<RefCell<Tokens<'a, T, C, E, Q>>>;

    fn get_ownable_module(&self) -> Rc<RefCell<Ownable<'a>>>;

    // Some modules may not have redeemable module
    fn get_redeemable_module(&self) -> Option<Rc<RefCell<Redeemable<'a>>>>;

    fn get_listed_tokens(&self) -> Map<'a, &'a str, Coin>;
}

impl<'a, T, C, E, Q> SellableModule<'a, T, C, E, Q> for Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn check_ownable(
        &self,
        deps: &Deps,
        _env: &Env,
        info: &MessageInfo,
        ownable: &Ownable,
    ) -> Result<(), ContractError> {
        if !ownable.is_owner(deps, &info.sender)? {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    fn check_redeemable(
        &self,
        _deps: &Deps,
        _env: &Env,
        _info: &MessageInfo,
        _token_id: &str,
        _redeemable: &Redeemable,
    ) -> Result<(), ContractError> {
        // Sellable module does not check redeemable
        Ok(())
    }

    fn check_is_allowed(
        &self,
        deps: &Deps,
        info: &MessageInfo,
    ) -> Result<(), ContractError> {
        let allowable = &self.allowable.borrow();
        if !allowable.is_allowed(deps, info.sender.clone())? {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    fn get_token_module(&self) -> Rc<RefCell<Tokens<'a, T, C, E, Q>>> {
        self.tokens.clone()
    }

    fn get_ownable_module(&self) -> Rc<RefCell<Ownable<'a>>> {
        self.ownable.clone()
    }

    fn get_redeemable_module(&self) -> Option<Rc<RefCell<Redeemable<'a>>>> {
        None
    }

    fn get_listed_tokens(&self) -> Map<'a, &'a str, Coin> {
        self.listed_tokens.clone()
    }
}

impl<'a, T, C, E, Q> SellableModule<'a, T, C, E, Q> for RSellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn check_ownable(
        &self,
        deps: &Deps,
        _env: &Env,
        info: &MessageInfo,
        ownable: &Ownable,
    ) -> Result<(), ContractError> {
        if !ownable.is_owner(deps, &info.sender)? {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    fn check_redeemable(
        &self,
        deps: &Deps,
        _env: &Env,
        _info: &MessageInfo,
        token_id: &str,
        redeemable: &Redeemable,
    ) -> Result<(), ContractError> {
        // confirm token aren't locked or redeemed
        let locked_tokens = redeemable.locked_items.load(deps.storage)?;
        if locked_tokens.contains(token_id) {
            return Err(ContractError::TicketRedeemed);
        }
        Ok(())
    }

    fn check_is_allowed(
        &self,
        deps: &Deps,
        info: &MessageInfo,
    ) -> Result<(), ContractError> {
        let allowable = &self.allowable.borrow();
        if !allowable.is_allowed(deps, info.sender.clone())? {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    fn get_token_module(&self) -> Rc<RefCell<Tokens<'a, T, C, E, Q>>> {
        self.tokens.clone()
    }

    fn get_ownable_module(&self) -> Rc<RefCell<Ownable<'a>>> {
        self.ownable.clone()
    }

    fn get_redeemable_module(&self) -> Option<Rc<RefCell<Redeemable<'a>>>> {
        Some(self.redeemable.clone())
    }

    fn get_listed_tokens(&self) -> Map<'a, &'a str, Coin> {
        self.listed_tokens.clone()
    }
}
