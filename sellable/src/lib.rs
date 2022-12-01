pub mod errors;
pub mod execute;
pub mod msg;
pub mod query;

use std::cell::RefCell;
use std::rc::Rc;

use cosmwasm_std::{to_binary, CustomMsg, Deps, DepsMut, Env, MessageInfo};
use errors::ContractError;
use msg::{ExecuteMsg, QueryMsg, QueryResp, SellableTrait};
use ownable::Ownable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use token::Tokens;

use burnt_glue::module::Module;
use burnt_glue::response::Response;

pub struct Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub tokens: Rc<RefCell<Tokens<'a, T, C, E, Q>>>,
    pub ownable: Rc<RefCell<Ownable<'a>>>,
}

impl<'a, T, C, E, Q> Default for Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn default() -> Self {
        Self {
            tokens: Rc::new(RefCell::new(Tokens::default())),
            ownable: Rc::new(RefCell::new(Ownable::default())),
        }
    }
}

impl<'a, 'b, T, C, E, Q> Module for Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
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
                self.try_buy(deps.branch(), info)?;
            }
            ExecuteMsg::List { listings } => {
                self.try_list(deps, env, info, listings)?;
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
                let response = self.listed_tokens(deps, start_after, limit);
                return Ok(QueryResp::Result(to_binary(&response.unwrap())?));
            }
        }
    }
}
