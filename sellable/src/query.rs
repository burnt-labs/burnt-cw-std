use cosmwasm_std::{CustomMsg, Deps, Order, StdResult, Uint64};
use cw721_base::state::TokenInfo;
use cw_storage_plus::Bound;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{RSellable, Sellable};

const DEFAULT_LIMIT: u32 = 500;
const MAX_LIMIT: u32 = 10000;

impl<'a, T, C, E, Q> Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn listed_tokens(
        &self,
        deps: &Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<ListedTokensResponse<T>> {
        let contract = &self.tokens.try_borrow().unwrap().contract;
        let listed_tokens = self.listed_tokens.load(deps.storage)?;

        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let token_vec = contract
            .tokens
            .range(deps.storage, start, None, Order::Ascending)
            .flat_map(|result| match result {
                Ok(pair) => {
                    if listed_tokens.contains_key(&pair.0) {
                        if *(listed_tokens.get(&pair.0).unwrap()) > Uint64::new(0) {
                            return Some(pair);
                        }
                    }
                    return None;
                }
                _ => None,
            })
            .take(limit)
            .collect();

        Ok(ListedTokensResponse { tokens: token_vec })
    }
}

impl<'a, T, C, E, Q> RSellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn listed_tokens(
        &self,
        deps: &Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<ListedTokensResponse<T>> {
        let contract = &self.tokens.try_borrow().unwrap().contract;
        let listed_tokens = self.listed_tokens.load(deps.storage)?;

        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let token_vec = contract
            .tokens
            .range(deps.storage, start, None, Order::Ascending)
            .flat_map(|result| match result {
                Ok(pair) => {
                    if listed_tokens.contains_key(&pair.0) {
                        if *(listed_tokens.get(&pair.0).unwrap()) > Uint64::new(0) {
                            return Some(pair);
                        }
                    }
                    return None;
                }
                _ => None,
            })
            .take(limit)
            .collect();

        Ok(ListedTokensResponse { tokens: token_vec })
    }
}

#[derive(Serialize, Clone, Deserialize, PartialEq, JsonSchema, Debug)]
pub struct ListedTokensResponse<T> {
    /// Contains all token_ids in lexicographical ordering
    /// If there are more than `limit`, use `start_from` in future queries
    /// to achieve pagination.
    pub tokens: Vec<(String, TokenInfo<T>)>,
}
