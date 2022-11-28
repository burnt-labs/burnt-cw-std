pub mod errors;
pub mod execute;
pub mod msg;
pub mod query;

use cosmwasm_std::{to_binary, CustomMsg, Deps, DepsMut, Env, MessageInfo};
use errors::ContractError;
use execute::try_list;
use msg::{ExecuteMsg, QueryMsg, QueryResp, SellableTrait};
use ownable::Ownable;
use query::listed_tokens;
use serde::de::DeserializeOwned;
use serde::Serialize;
use token::Tokens;

use burnt_glue::module::Module;
use burnt_glue::response::Response;

pub struct Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait<T>,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub tokens: Tokens<'a, T, C, E, Q>,
    pub ownable: Ownable<'a>,
}

impl<'a, T, C, E, Q> Default for Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait<T>,
    Q: CustomMsg,
    E: CustomMsg,
{
    fn default() -> Self {
        Self {
            tokens: Tokens::default(),
            ownable: Ownable::default(),
        }
    }
}

impl<'a, T, C, E, Q> Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait<T>,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub fn new(tokens_module: Tokens<'a, T, C, E, Q>, ownable_module: Ownable<'a>) -> Self {
        Self {
            tokens: tokens_module,
            ownable: ownable_module,
        }
    }
}

impl<'a, 'b, T, C, E, Q> Module for Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait<T>,
    Q: CustomMsg + DeserializeOwned,
    E: CustomMsg + DeserializeOwned,
    C: CustomMsg + DeserializeOwned,
{
    type InstantiateMsg = ();
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = ContractError;

    fn instantiate(
        &mut self,
        _deps: &mut DepsMut,
        _env: &Env,
        _info: &MessageInfo,
        _msg: (),
    ) -> Result<Response, Self::Error> {
        unimplemented!();
    }

    fn execute(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, Self::Error> {
        match msg {
            ExecuteMsg::Buy {} => {
                unimplemented!()
            }
            ExecuteMsg::List { listings } => {
                try_list(deps, env, info, listings, self)?;
            }
            ExecuteMsg::RedeemTicket { .. } => {
                unimplemented!()
            }
        }
        Ok(Response::new())
    }

    fn query(&self, deps: &Deps, _env: Env, msg: QueryMsg) -> Result<Self::QueryResp, Self::Error> {
        match msg {
            QueryMsg::ListedTokens { start_after, limit } => {
                let response = listed_tokens(deps, start_after, limit, self);
                return Ok(QueryResp::Result(to_binary(&response.unwrap())?));
            }
        }
    }
}
