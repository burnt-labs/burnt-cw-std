pub mod errors;
pub mod execute;
pub mod query;
pub mod state;

use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo};
use cw_storage_plus::Item;
use errors::ContractError;
use schemars::{JsonSchema, Set};
use serde::{Deserialize, Serialize};
use state::LOCKED_ITEMS;

use burnt_glue::module::Module;
use burnt_glue::response::Response;

pub struct Redeemable<'a> {
    pub locked_items: Item<'a, Set<String>>,
}

impl Default for Redeemable<'_> {
    fn default() -> Self {
        Self {
            locked_items: LOCKED_ITEMS,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    locked_items: Set<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsRedeemed(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    IsRedeemed(bool),
}

impl Module for Redeemable<'_> {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = ContractError;

    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        _env: &Env,
        _info: &MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, Self::Error> {
        self.locked_items.save(deps.storage, &msg.locked_items)?;
        Ok(Response::new())
    }

    fn execute(
        &mut self,
        _deps: &mut DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: ExecuteMsg,
    ) -> Result<Response, Self::Error> {
        unimplemented!("execute not implemented")
    }

    fn query(&self, deps: &Deps, _env: Env, msg: QueryMsg) -> Result<Self::QueryResp, Self::Error> {
        match msg {
            QueryMsg::IsRedeemed(token_id) => {
                let is_redeemed = self.is_redeemed(deps, token_id).unwrap_or(false);
                Ok(QueryResp::IsRedeemed(is_redeemed))
            }
        }
    }
}
