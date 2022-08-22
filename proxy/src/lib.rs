use std::fmt::Debug;
use cosmwasm_std::{Addr, Api, Binary, BlockInfo, CustomQuery, Deps, DepsMut, Env, MessageInfo, OwnedDeps, Querier, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::StdError;
use thiserror::Error;

use burnt_glue::module::Module;
use burnt_glue::response::Response;
use ownable::{Ownable, OwnableError};

pub struct Proxy<'a> {
    ownable: &'a Ownable<'a>,
    destination: Item<'a, Addr>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Update(Addr),
    Call(Binary)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Destination returns the contract this proxy points to
    Destination,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    // Destination returns the contract this proxy points to
    Destination(Addr),
}


impl<'a> Module for Proxy<'a> {
    type InstantiateMsg = Addr;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = OwnableError;

    fn instantiate(&mut self, deps: &mut DepsMut, env: &Env, info: &MessageInfo, msg: Self::InstantiateMsg) -> Result<Response, Self::Error> {
        self.destination.save(deps.storage, &msg)?;

        Ok(Response::new())
    }

    fn execute(&mut self, deps: &mut DepsMut, env: Env, info: MessageInfo, msg: Self::ExecuteMsg) -> Result<Response, Self::Error> {
        let owner = self.ownable

        todo!()
    }

    fn query(&self, deps: &Deps, env: Env, msg: Self::QueryMsg) -> Result<Self::QueryResp, Self::Error> {
        todo!()
    }
}