pub mod errors;
pub mod execute;
pub mod msg;
pub mod query;
mod test;

use std::{cell::RefCell, rc::Rc};

use cw_storage_plus::Item;
use schemars::JsonSchema;
use sellable::Sellable;

use cosmwasm_std::{CustomMsg, Deps, DepsMut, Env, MessageInfo};
use errors::ContractError;
use msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryResp};
use serde::de::DeserializeOwned;
use serde::Serialize;

use burnt_glue::module::Module;
use burnt_glue::response::Response;
use cosmwasm_std::{Coin, Timestamp, Uint64};
use serde::Deserialize;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct PrimarySale {
    pub total_supply: Uint64,
    pub tokens_minted: Uint64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub price: Vec<Coin>,
    pub disabled: bool, // is sale still on ?
}

pub struct Sales<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub sellable: Rc<RefCell<Sellable<'a, T, C, E, Q>>>,
    pub primary_sales: Item<'a, Vec<PrimarySale>>,
}

impl<'a, T, C, E, Q> Default for Sales<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    fn default() -> Self {
        Self {
            sellable: Rc::new(RefCell::new(Sellable::default())),
            primary_sales: Item::new("primary_sales"),
        }
    }
}

impl<'a, T, C, E, Q> Module for Sales<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg + DeserializeOwned,
    E: CustomMsg + DeserializeOwned,
    C: CustomMsg + DeserializeOwned,
{
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg<T>;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp;
    type Error = ContractError;

    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, Self::Error> {
        if let Some(sale) = msg.sales {
            self.add_primary_sales(sale, deps, env.clone(), info)?;
        } else {
            self.primary_sales.save(deps.storage, &vec![])?;
        }
        Ok(Response::default())
    }

    fn execute(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T>,
    ) -> Result<Response, Self::Error> {
        match msg {
            ExecuteMsg::PrimarySale(msg) => self.add_primary_sales(msg, deps, env, &info),

            ExecuteMsg::HaltSale {} => self.halt_sale(deps, env),

            ExecuteMsg::BuyItem(mint_msg) => self.buy_item(env, deps, info, mint_msg),
        }
    }

    fn query(&self, deps: &Deps, env: Env, msg: QueryMsg) -> Result<Self::QueryResp, Self::Error> {
        match msg {
            QueryMsg::ActivePrimarySale {} => Ok(self.active_primary_sales(deps, env)?),
            QueryMsg::PrimarySales {} => Ok(self.primary_sales(deps)?),
        }
    }
}
