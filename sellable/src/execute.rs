use std::{cell::RefCell, ops::Sub, rc::Rc};

use crate::{errors::ContractError, RSellable, Sellable};
use allowable::Allowable;
use burnt_glue::response::Response;
use cosmwasm_std::{BankMsg, Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Order, Uint128};
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
        allowable_module: Rc<RefCell<Allowable<'a>>>,
        ownable_module: Rc<RefCell<Ownable<'a>>>,
        listed_tokens: Map<'a, &'a str, Coin>,
    ) -> Self {
        Self {
            tokens: tokens_module,
            allowable: allowable_module,
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
        let ownable = &self.ownable.borrow();
        check_ownable(&deps.as_ref(), &env, &info, ownable)?;

        for (token_id, price) in listings {
            if price.amount > Uint128::new(0) {
                if self.listed_tokens.may_load(deps.storage, &token_id).is_ok() {
                    return Err(ContractError::TokenAlreadyListed);
                } else if let Ok(Some(_)) = self
                    .tokens
                    .borrow()
                    .contract
                    .tokens
                    .may_load(deps.storage, &token_id)
                {
                    self.listed_tokens
                        .save(deps.storage, token_id.as_str(), &price)?;
                } else {
                    return Err(ContractError::TokenIDNotFoundError);
                }
            } else {
                return Err(ContractError::InvalidListingPrice);
            }
        }
        Ok(Response::new().add_attribute("method", "list"))
    }

    pub fn try_delist(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // Check that the token is still listed
        self.listed_tokens
            .load(deps.storage, &token_id)
            .map_err(|_| ContractError::NoListedTokensError)?;

        let listed_token = self
            .tokens
            .borrow()
            .contract
            .tokens
            .load(deps.storage, &token_id)
            .map_err(|_| ContractError::TokenIDNotFoundError)?;
        if listed_token.owner.eq(&info.sender) {
            self.listed_tokens.remove(deps.storage, &token_id);
            Ok(Response::new().add_attribute("delist", token_id))
        } else {
            Err(ContractError::Unauthorized)
        }
    }

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut sorted_tokens = self
            .listed_tokens
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
        let lowest_listed_token = sorted_tokens.get(0).unwrap();

        self.try_buy_token(deps, info, lowest_listed_token.clone().0)
    }

    pub fn try_buy_token(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        let allowable = &self.allowable.borrow();
        check_is_allowed(&deps.as_ref(), allowable, &info)?;

        // check if enough fee was sent
        match info.funds.as_slice() {
            [fund] => self
                .listed_tokens
                .load(deps.storage, token_id.as_str())
                .map_err(|_| ContractError::NoListedTokensError)
                .and_then(|price| {
                    if fund.denom.ne(&price.denom) {
                        Err(ContractError::WrongFundError)
                    } else if fund.amount.ge(&price.amount) {
                        let token_metadata = self
                            .tokens
                            .borrow()
                            .contract
                            .tokens
                            .load(deps.storage, &token_id)
                            .map_err(|_| ContractError::NoMetadataPresent)?;
                        self.tokens
                            .borrow_mut()
                            .contract
                            .tokens
                            .update::<_, ContractError>(deps.storage, token_id.as_str(), |old| {
                                let mut token_info = old.unwrap();
                                token_info.owner = info.sender.clone();
                                Ok(token_info)
                            })?;
                        self.listed_tokens.remove(deps.storage, &token_id);

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

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let allowable = &self.allowable.borrow();
        check_is_allowed(&deps.as_ref(), allowable, &info)?;

        let denom_name: String;
        if let Some(denom) = self.tokens.borrow().name.clone() {
            denom_name = denom;
        } else {
            return Err(ContractError::NoTokensDefined);
        }
        let contract = &self.tokens.borrow_mut().contract;
        let maybe_coin = info.funds.iter().find(|&coin| coin.denom.eq(&denom_name));

        if let Some(coin) = maybe_coin {
            let limit = (coin.amount.u128() as u64).into();

            let mut sorted_tokens = self
                .listed_tokens
                .range(deps.storage, None, None, Order::Descending)
                .map(|t| t.unwrap())
                .collect::<Vec<(String, Coin)>>();
            sorted_tokens.sort_unstable_by_key(|t| t.1.amount);
            if sorted_tokens.is_empty() {
                return Err(ContractError::NoListedTokensError);
            }
            let lowest_listed_token = sorted_tokens.get(0).unwrap();
            let token_info = contract
                .tokens
                .load(deps.storage, lowest_listed_token.0.as_str())?;
            let lowest = Ok((
                lowest_listed_token.clone().0,
                token_info.owner,
                lowest_listed_token.1.amount,
            ));

            lowest
                .and_then(|l @ (_, _, lowest_price)| {
                    if lowest_price <= limit {
                        Ok(l)
                    } else {
                        Err(ContractError::LimitBelowLowestOffer {
                            limit,
                            lowest_price,
                        })
                    }
                })
                .and_then(|(lowest_token_id, lowest_token_owner, lowest_price)| {
                    contract.tokens.update::<_, ContractError>(
                        deps.storage,
                        lowest_token_id.as_str(),
                        |old| {
                            let mut token_info = old.unwrap();
                            token_info.owner = info.sender.clone();
                            Ok(token_info)
                        },
                    )?;
                    self.listed_tokens
                        .remove(deps.storage, lowest_token_id.as_str());

                    let payment_coin = Coin::new(lowest_price.into(), &denom_name);
                    let delta = limit - lowest_price;
                    let mut messages = vec![BankMsg::Send {
                        to_address: lowest_token_owner.to_string(),
                        amount: vec![payment_coin],
                    }];
                    if delta > Uint128::new(0) {
                        messages.push(BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![Coin::new(delta.into(), &denom_name)],
                        })
                    }

                    Ok(Response::new()
                        .add_attribute("method", "buy")
                        .add_messages(messages))
                })
        } else {
            Err(ContractError::NoFundsPresent)
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
        allowable_module: Rc<RefCell<Allowable<'a>>>,
        ownable_module: Rc<RefCell<Ownable<'a>>>,
        listed_tokens: Map<'a, &'a str, Coin>,
        redeemable_module: Rc<RefCell<Redeemable<'a>>>,
    ) -> Self {
        Self {
            tokens: token_module,
            allowable: allowable_module,
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
        let ownable = &self.ownable.borrow();
        let redeemable = &self.redeemable.borrow();

        check_ownable(&deps.as_ref(), &env, &info, ownable)?;
        for (token_id, price) in &listings {
            if self.listed_tokens.may_load(deps.storage, &token_id).is_ok() {
                return Err(ContractError::TokenAlreadyListed);
            } else if price.amount > Uint128::new(0) {
                if self
                    .tokens
                    .borrow()
                    .contract
                    .tokens
                    .may_load(deps.storage, token_id)
                    .unwrap()
                    .is_some()
                {
                    check_redeemable(&deps.as_ref(), &env, &info, token_id, redeemable)?;
                    self.listed_tokens
                        .save(deps.storage, token_id.as_str(), price)?;
                } else {
                    return Err(ContractError::TokenIDNotFoundError);
                }
            } else {
                return Err(ContractError::InvalidListingPrice);
            }
        }
        Ok(Response::new().add_attribute("list", json!(listings).to_string()))
    }

    pub fn try_delist(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // Check that the token is still listed
        self.listed_tokens
            .load(deps.storage, &token_id)
            .map_err(|_| ContractError::NoListedTokensError)?;

        let listed_token = self
            .tokens
            .borrow()
            .contract
            .tokens
            .load(deps.storage, &token_id)
            .map_err(|_| ContractError::TokenIDNotFoundError)?;
        if listed_token.owner.eq(&info.sender) {
            self.listed_tokens.remove(deps.storage, &token_id);
            Ok(Response::new().add_attribute("delist", token_id))
        } else {
            Err(ContractError::Unauthorized)
        }
    }

    pub fn try_buy_token(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // check if enough fee was sent
        match info.funds.as_slice() {
            [fund] => self
                .listed_tokens
                .load(deps.storage, token_id.as_str())
                .map_err(|_| ContractError::NoListedTokensError)
                .and_then(|price| {
                    if fund.denom.ne(&price.denom) {
                        Err(ContractError::WrongFundError)
                    } else if fund.amount.ge(&price.amount) {
                        let redeemable = &self.redeemable.borrow();
                        check_redeemable(&deps.as_ref(), env, &info, &token_id, redeemable)?;
                        let token_metadata = self
                            .tokens
                            .borrow()
                            .contract
                            .tokens
                            .load(deps.storage, &token_id)
                            .map_err(|_| ContractError::NoMetadataPresent)?;
                        self.tokens
                            .borrow_mut()
                            .contract
                            .tokens
                            .update::<_, ContractError>(deps.storage, token_id.as_str(), |old| {
                                let mut token_info = old.unwrap();
                                token_info.owner = info.sender.clone();
                                Ok(token_info)
                            })?;
                        self.listed_tokens.remove(deps.storage, &token_id);

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

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        env: &Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut sorted_tokens = self
            .listed_tokens
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
        let lowest_listed_token = sorted_tokens.get(0).unwrap();

        self.try_buy_token(deps, env, info, lowest_listed_token.clone().0)
    }
}

fn check_is_allowed(
    deps: &Deps,
    allowable: &Allowable,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    if !allowable.is_allowed(deps, info.sender.clone())? {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

fn check_ownable(
    deps: &Deps,
    _env: &Env,
    info: &MessageInfo,
    ownable: &Ownable,
) -> Result<(), ContractError> {
    if !ownable.is_owner(deps, &info.sender)? {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

fn check_redeemable(
    deps: &Deps,
    _env: &Env,
    _info: &MessageInfo,
    token_id: &String,
    redeemable: &Redeemable,
) -> Result<(), ContractError> {
    // confirm token aren't locked or redeemed
    let locked_tokens = redeemable.locked_items.load(deps.storage)?;
    if locked_tokens.contains(token_id) {
        return Err(ContractError::TicketRedeemed);
    }
    Ok(())
}
