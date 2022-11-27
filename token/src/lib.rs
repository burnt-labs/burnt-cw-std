use std::fmt::Debug;
use cosmwasm_std::{ Deps, DepsMut, Env, MessageInfo, CustomMsg };
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Binary;
use cw721_base::{Cw721Contract, InstantiateMsg, ExecuteMsg, QueryMsg, ContractError};

use burnt_glue::module::Module;
use burnt_glue::response::Response;

pub struct Tokens<'a, T, C, E, Q>
where 
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub contract: Cw721Contract<'a, T, C, E, Q>,
}

impl<'a, T, C, E, Q> Default for Tokens<'a, T, C, E, Q> 
where 
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    fn default() -> Self {
        Self {
            contract: Cw721Contract::<T, C, E, Q>::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    Result(Binary),
}

impl<'a, T, C, E, Q> Tokens<'a, T, C, E, Q>
where 
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
}

impl<'a,'b, T, C, E, Q> Module for Tokens<'a, T, C, E, Q> 
where 
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg + DeserializeOwned,
    E: CustomMsg + DeserializeOwned,
    C: CustomMsg + DeserializeOwned,
{
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg<T, E>;
    type QueryMsg = QueryMsg<Q>;
    type QueryResp = QueryResp;
    type Error = ContractError;

    fn instantiate(&mut self,
                   deps: &mut DepsMut,
                   env: &Env,
                   info: &MessageInfo,
                   msg: InstantiateMsg, ) -> Result<Response, Self::Error> {
        self.contract.instantiate(deps.branch(), env.clone(), info.clone(), msg)?;
        Ok(Response::new())
    }

    fn execute(&mut self,
               deps: &mut DepsMut,
               env: Env,
               info: MessageInfo,
               msg: ExecuteMsg<T, E>, ) -> Result<Response, Self::Error> {
       self.contract.execute(deps.branch(), env.clone(), info.clone(), msg)?;
       Ok(Response::new())
        
    }

    fn query(&self,
             deps: &Deps,
             env: Env,
             msg: QueryMsg<Q>, ) -> Result<Self::QueryResp, Self::Error> {
        let response = self.contract.query(deps.clone(), env.clone(), msg)?;
        Ok(QueryResp::Result(response))
    }
}
