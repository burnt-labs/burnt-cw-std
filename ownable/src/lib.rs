use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, StdResult};
use cosmwasm_std::{Event, StdError};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

use crate::OwnableError::Unauthorized;
use burnt_glue::module::Module;
use burnt_glue::response::Response;

pub const OWNER_STATE: Item<Addr> = Item::new("owner");

pub struct Ownable<'a> {
    pub owner: Item<'a, Addr>,
}

impl<'a> Default for Ownable<'a> {
    fn default() -> Self {
        Self { owner: OWNER_STATE }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetOwner(Addr),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // IsOwner returns true if the address matches the owner
    IsOwner(Addr),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    // IsOwner returns true if the address matches the owner
    IsOwner(bool),
}

#[derive(Error, Debug)]
pub enum OwnableError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    // #[error("{0}")]
    // SerdeJson(#[from] serde_json::Error),
    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl<'a> Ownable<'a> {
    pub fn get_owner(&self, deps: &Deps) -> StdResult<Addr> {
        self.owner.load(deps.storage)
    }

    pub fn is_owner(&self, deps: &Deps, addr: &Addr) -> StdResult<bool> {
        self.owner.load(deps.storage).map(|owner| owner.eq(addr))
    }

    pub fn set_owner(&self, deps: &mut DepsMut, addr: &Addr) -> StdResult<()> {
        // validate Addr before saving
        deps.api.addr_validate(addr.as_str())?;
        self.owner.save(deps.storage, addr)
    }
}

impl<'a> Module for Ownable<'a> {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = OwnableError;

    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        _: Self::InstantiateMsg,
    ) -> Result<Response, Self::Error> {
        self.owner.save(deps.storage, &info.sender)?;
        let resp = Response::new()
            .add_event(Event::new("ownable-instantiate"))
            .add_attributes(vec![
                ("contract_address", env.contract.address.to_string()),
                ("owner", info.sender.to_string()),
            ]);
        Ok(resp)
    }

    fn execute(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error> {
        match msg {
            ExecuteMsg::SetOwner(owner) => {
                // validate Addr before saving
                deps.api.addr_validate(owner.as_str())?;

                let loaded_owner = self.owner.load(deps.storage).unwrap();
                if info.sender != loaded_owner {
                    Err(Unauthorized {})
                } else {
                    self.set_owner(deps, &owner)?;
                    let resp = Response::new().add_event(
                        Event::new("ownable-set_owner").add_attributes(vec![
                            ("contract_address", env.contract.address.to_string()),
                            ("owner", owner.to_string()),
                        ]),
                    );
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
            QueryMsg::IsOwner(address) => {
                let loaded_owner = self.owner.load(deps.storage).unwrap();
                let resp = QueryResp::IsOwner(loaded_owner == address);
                Ok(resp)
            }
        }
    }
}
