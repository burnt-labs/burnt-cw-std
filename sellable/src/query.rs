use std::borrow::Borrow;

use cosmwasm_std::{CustomMsg, Deps, Order, StdResult};
use cw721_base::state::TokenInfo;
use cw_storage_plus::Bound;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{msg::SellableTrait, Sellable};

const DEFAULT_LIMIT: u32 = 500;
const MAX_LIMIT: u32 = 10000;

pub fn listed_tokens
<
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
    C: CustomMsg,
    Q: CustomMsg,
    E: CustomMsg,
>(
    deps: &Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    sellable_module: &Sellable<T, C, E, Q>,
) -> StdResult<ListedTokensResponse<T>> {
    let contract = &sellable_module.tokens.try_borrow().unwrap().contract;

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let token_vec = contract
        .tokens
        .range(deps.storage, start, None, Order::Ascending)
        .flat_map(|result| match result {
            Ok(pair) => Some(pair),
            _ => None,
        })
        .take(limit)
        .collect();

    Ok(ListedTokensResponse { tokens: token_vec })
}

#[derive(Serialize, Clone, Deserialize, PartialEq, JsonSchema, Debug)]
pub struct ListedTokensResponse<T> {
    /// Contains all token_ids in lexicographical ordering
    /// If there are more than `limit`, use `start_from` in future queries
    /// to achieve pagination.
    pub tokens: Vec<(String, TokenInfo<T>)>,
}
