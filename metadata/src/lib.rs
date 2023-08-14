use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, StdResult};
use cosmwasm_std::{Event, StdError};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

use burnt_glue::module::Module;
use burnt_glue::response::Response;
use ownable::Ownable;

pub struct Metadata<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub metadata: Item<'a, T>,
    ownable: Rc<RefCell<Ownable<'a>>>,
}

impl<'a, T> Metadata<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(metadata: Item<'a, T>, ownable: Rc<RefCell<Ownable<'a>>>) -> Self {
        Self { metadata, ownable }
    }
}

impl<'a, T> Default for Metadata<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self {
            metadata: Item::new("metadata"),
            ownable: Rc::new(RefCell::new(Ownable::default())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg<T> {
    pub metadata: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T> {
    SetMetadata(T),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetMetadata {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp<T> {
    Metadata(T),
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl<'a, T> Metadata<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn get_metadata(&self, deps: &Deps) -> StdResult<T> {
        self.metadata.load(deps.storage)
    }

    pub fn set_metadata(&self, deps: &mut DepsMut, meta: &T) -> StdResult<()> {
        self.metadata.save(deps.storage, meta)
    }
}

impl<'a, T> Module for Metadata<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    type InstantiateMsg = InstantiateMsg<T>;
    type ExecuteMsg = ExecuteMsg<T>;
    type QueryMsg = QueryMsg;
    type QueryResp = QueryResp<T>;
    type Error = MetadataError;

    fn instantiate(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Self::Error> {
        self.metadata.save(deps.storage, &msg.metadata)?;
        let resp = Response::new()
            .add_event(Event::new("metadata-instantiate"))
            .add_attributes(
                vec![
                    ("contract_address", env.contract.address.to_string()),
                    ("owner", info.sender.to_string()),
                    ("metadata", serde_json::to_string(&msg.metadata).unwrap()),
                ]
                .into_iter(),
            );
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
            ExecuteMsg::SetMetadata(meta) => {
                let owner_module = self.ownable.borrow();
                let loaded_owner = owner_module.get_owner(&deps.as_ref()).unwrap();
                if info.sender != loaded_owner {
                    Err(MetadataError::Unauthorized {})
                } else {
                    self.metadata.save(deps.storage, &meta).unwrap();
                    let resp = Response::new()
                        .add_event(Event::new("metadata-set_metadata"))
                        .add_attributes(vec![
                            ("contract_address", env.contract.address.to_string()),
                            ("owner", info.sender.to_string()),
                            ("metadata", serde_json::to_string(&meta).unwrap()),
                        ]);
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
            QueryMsg::GetMetadata {} => {
                let loaded_metadata = self.metadata.load(deps.storage).unwrap();
                let resp = QueryResp::Metadata(loaded_metadata);
                Ok(resp)
            }
        }
    }
}
