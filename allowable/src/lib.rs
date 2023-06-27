mod state;
mod msg;
mod errors;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, StdResult};
use std::rc::Rc;
use std::cell::RefCell;


use ownable::Ownable;
use burnt_glue::module::Module;
use burnt_glue::response::Response;
use cw_storage_plus::{Item, Map};
use crate::errors::AllowableError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryResp};
use crate::state::{ALLOWED_ADDRS, ENABLED};

pub struct Allowable<'a> {
    ownable: Rc<RefCell<Ownable<'a>>>,
    pub allowed_addrs: Map<'a, Addr, bool>,
    pub enabled: Item<'a, bool>,
}

impl<'a> Default for Allowable<'a> {
    fn default() -> Self {
        Self {
            ownable: Rc::new(RefCell::new(Ownable::default())),
            allowed_addrs: ALLOWED_ADDRS,
            enabled: ENABLED,
        }
    }
}

impl<'a> Allowable<'a> {
    pub fn get_enabled(&self, deps: &Deps) -> StdResult<bool> {
        self.enabled.load(deps.storage)
    }

    pub fn set_enabled(&self, deps: &mut DepsMut, enabled: bool) -> StdResult<()> {
        self.enabled.save(deps.storage, &enabled)
    }

    pub fn allow_addrs(&self, deps: &mut DepsMut, allowed_addrs: Vec<Addr>) -> StdResult<()> {
        for addr in allowed_addrs {
            self.allowed_addrs.save(deps.storage, addr, &true)?;
        }
        Ok(())
    }

    pub fn remove_addrs(&self, deps: &mut DepsMut, removed_addrs: Vec<Addr>) -> StdResult<()> {
        for addr in removed_addrs {
            self.allowed_addrs.remove(deps.storage, addr);
        }
        Ok(())
    }

    pub fn clear_addrs(&self, deps: &mut DepsMut) -> StdResult<()> {
        self.allowed_addrs.clear(deps.storage);
        Ok(())
    }

    pub fn is_allowed(&self, deps: &Deps, addr: Addr) -> StdResult<bool> {
        Ok(self.allowed_addrs.has(deps.storage, addr))
    }
}

impl<'a> Module for Allowable<'a> {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = AllowableError;

    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        _: &Env,
        _: &MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Self::Error> {
        self.enabled.save(deps.storage, &msg.enabled)?;
        self.allow_addrs(deps, msg.allowed_addrs)?;

        Ok(Response::new())
    }

    fn execute(
        &mut self,
        deps: &mut DepsMut,
        _: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error> {
        let owner_module = self.ownable.borrow();
        let loaded_owner = owner_module.get_owner(&deps.as_ref()).unwrap();
        if info.sender != loaded_owner {
            Err(AllowableError::Unauthorized {})
        } else {
            match msg {
                ExecuteMsg::SetEnabled(enabled) => {
                    self.set_enabled(deps, enabled)?;
                    let resp = Response::new();
                    Ok(resp)
                }

                ExecuteMsg::ClearAllowedAddrs() => {
                    self.clear_addrs(deps)?;
                    let resp = Response::new();
                    Ok(resp)
                }

                ExecuteMsg::AddAllowedAddrs(addrs) => {
                    self.allow_addrs(deps, addrs)?;
                    let resp = Response::new();
                    Ok(resp)
                }

                ExecuteMsg::RemoveAllowedAddrs(addrs) => {
                    self.remove_addrs(deps, addrs)?;
                    let resp = Response::new();
                    Ok(resp)
                }
            }
        }
    }

    fn query(
        &self,
        deps: &Deps,
        _: Env,
        msg: Self::QueryMsg,
    ) -> Result<Self::QueryResp, Self::Error> {
        match msg {
            QueryMsg::IsAllowed(address) => {
                let is_allowed = self.is_allowed(deps, address)?;
                let resp = QueryResp::IsAllowed(is_allowed);
                Ok(resp)
            }
            QueryMsg::IsEnabled() => {
                let is_enabled = self.get_enabled(deps)?;
                let resp = QueryResp::IsEnabled(is_enabled);
                Ok(resp)
            }
        }
    }
}
