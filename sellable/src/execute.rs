use std::{cell::RefCell, ops::Sub, rc::Rc};

use crate::{errors::ContractError, sellable_module::SellableModule, RSellable, Sellable};
use burnt_glue::response::Response;
use cosmwasm_std::{BankMsg, Coin, CustomMsg, DepsMut, Env, MessageInfo, Order, Uint128};
use cw_storage_plus::Map;
use ownable::Ownable;
use redeemable::Redeemable;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use token::Tokens;

impl<'a, T, C, E, Q> Sellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn new(
        tokens_module: Rc<RefCell<Tokens<'a, T, C, E, Q>>>,
        ownable_module: Rc<RefCell<Ownable<'a>>>,
        listed_tokens: Map<'a, &'a str, Coin>,
    ) -> Self {
        Self {
            tokens: tokens_module,
            ownable: ownable_module,
            listed_tokens,
        }
    }

    pub fn try_list(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        listings: schemars::Map<String, Coin>,
    ) -> Result<Response, ContractError> {
        list_helper(self, deps, env, info, listings)
    }

    pub fn try_delist(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        delist_helper(self, deps, info, token_id)
    }

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: MessageInfo,
        token_id: Option<String>,
    ) -> Result<Response, ContractError> {
        if let Some(token_id) = token_id {
            buy_specific_item_helper(self, deps, env, info, token_id)
        } else {
            let token_id = get_lowest_item_helper(self, deps)?;
            buy_specific_item_helper(self, deps, env, info, token_id)
        }
    }
}

impl<'a, T, C, E, Q> RSellable<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn new(
        token_module: Rc<RefCell<Tokens<'a, T, C, E, Q>>>,
        ownable_module: Rc<RefCell<Ownable<'a>>>,
        listed_tokens: Map<'a, &'a str, Coin>,
        redeemable_module: Rc<RefCell<Redeemable<'a>>>,
    ) -> Self {
        Self {
            tokens: token_module,
            ownable: ownable_module,
            listed_tokens,
            redeemable: redeemable_module,
        }
    }

    pub fn try_list(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
        listings: schemars::Map<String, Coin>,
    ) -> Result<Response, ContractError> {
        list_helper(self, deps, env, info, listings)
    }

    pub fn try_delist(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        delist_helper(self, deps, info, token_id)
    }

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: MessageInfo,
        token_id: Option<String>,
    ) -> Result<Response, ContractError> {
        if let Some(token_id) = token_id {
            buy_specific_item_helper(self, deps, env, info, token_id)
        } else {
            let token_id = get_lowest_item_helper(self, deps)?;
            buy_specific_item_helper(self, deps, env, info, token_id)
        }
    }
}

fn list_helper<T, C, E, Q>(
    sellable_module: &dyn SellableModule<T, C, E, Q>,
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    listings: schemars::Map<String, Coin>,
) -> Result<Response, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    let ownable = sellable_module.get_ownable_module();
    let redeemable = sellable_module.get_redeemable_module();
    let listed_tokens = sellable_module.get_listed_tokens();
    let tokens = sellable_module.get_token_module();

    sellable_module.check_ownable(&deps.as_ref(), &env, &info, &ownable.borrow())?;

    for (token_id, price) in &listings {
        if listed_tokens
            .may_load(deps.storage, token_id)
            .unwrap()
            .is_some()
        {
            return Err(ContractError::TokenAlreadyListed);
        } else if price.amount > Uint128::new(0) {
            if tokens
                .borrow()
                .contract
                .tokens
                .may_load(deps.storage, token_id)
                .unwrap()
                .is_some()
            {
                if let Some(redeemable_module) = &redeemable {
                    sellable_module.check_redeemable(
                        &deps.as_ref(),
                        &env,
                        &info,
                        token_id,
                        &redeemable_module.borrow(),
                    )?;
                }

                listed_tokens.save(deps.storage, token_id.as_str(), price)?;
            } else {
                return Err(ContractError::TokenIDNotFoundError);
            }
        } else {
            return Err(ContractError::InvalidListingPrice);
        }
    }
    Ok(Response::new().add_attribute("list", json!(listings).to_string()))
}

fn delist_helper<T, C, E, Q>(
    sellable_module: &dyn SellableModule<T, C, E, Q>,
    deps: &mut DepsMut,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    let listed_tokens = sellable_module.get_listed_tokens();
    let tokens = sellable_module.get_token_module();
    // Check that the token is still listed
    listed_tokens
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::NoListedTokensError)?;

    let listed_token = tokens
        .borrow()
        .contract
        .tokens
        .load(deps.storage, &token_id)
        .map_err(|_| ContractError::TokenIDNotFoundError)?;
    if listed_token.owner.eq(&info.sender) {
        listed_tokens.remove(deps.storage, &token_id);
        Ok(Response::new().add_attribute("delist", token_id))
    } else {
        Err(ContractError::Unauthorized)
    }
}

fn get_lowest_item_helper<T, C, E, Q>(
    sellable_module: &dyn SellableModule<T, C, E, Q>,
    deps: &mut DepsMut,
) -> Result<String, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    let listed_tokens = sellable_module.get_listed_tokens();
    let mut sorted_tokens = listed_tokens
        .range(deps.storage, None, None, Order::Descending)
        .map(|t| t.unwrap())
        .collect::<Vec<(String, Coin)>>();
    if sorted_tokens.is_empty() {
        return Err(ContractError::NoListedTokensError);
    }
    sorted_tokens.sort_by(|a, b| {
        if a.1.amount == b.1.amount {
            a.1.denom.cmp(&b.1.denom)
        } else {
            a.1.amount.cmp(&b.1.amount)
        }
    });
    //  We only need the token index
    Ok(sorted_tokens.get(0).unwrap().0.clone())
}

fn buy_specific_item_helper<T, C, E, Q>(
    sellable_module: &dyn SellableModule<T, C, E, Q>,
    deps: &mut DepsMut,
    env: &Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    let redeemable = sellable_module.get_redeemable_module();
    let listed_tokens = sellable_module.get_listed_tokens();
    let tokens = sellable_module.get_token_module();

    match info.funds.as_slice() {
        [fund] => listed_tokens
            .load(deps.storage, token_id.as_str())
            .map_err(|_| ContractError::NoListedTokensError)
            .and_then(|price| {
                if fund.denom.ne(&price.denom) {
                    Err(ContractError::WrongFundError)
                } else if fund.amount.ge(&price.amount) {
                    if let Some(redeemable_module) = &redeemable {
                        sellable_module.check_redeemable(
                            &deps.as_ref(),
                            env,
                            &info,
                            &token_id,
                            &redeemable_module.borrow(),
                        )?;
                    }
                    let token_metadata = tokens
                        .borrow()
                        .contract
                        .tokens
                        .load(deps.storage, &token_id)
                        .map_err(|_| ContractError::NoMetadataPresent)?;
                    tokens
                        .borrow_mut()
                        .contract
                        .tokens
                        .update::<_, ContractError>(deps.storage, token_id.as_str(), |old| {
                            let mut token_info = old.unwrap();
                            token_info.owner = info.sender.clone();
                            Ok(token_info)
                        })?;
                    listed_tokens.remove(deps.storage, &token_id);

                    let delta = fund.amount.sub(price.amount);
                    let mut messages = vec![BankMsg::Send {
                        to_address: token_metadata.owner.to_string(),
                        amount: vec![price.clone()],
                    }];
                    if !delta.is_zero() {
                        messages.push(BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![Coin::new(delta.u128(), &price.denom)],
                        })
                    }

                    return Ok(Response::new().add_messages(messages));
                } else {
                    return Err(ContractError::InsufficientFundsError {
                        fund: fund.amount,
                        seat_price: price.amount,
                    });
                }
            }),
        [] => Err(ContractError::NoFundsPresent),
        _ => Err(ContractError::MultipleFundsError),
    }
}
