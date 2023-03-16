use std::{cell::RefCell, ops::Sub, rc::Rc};

use crate::{errors::ContractError, RSellable, Sellable};
use burnt_glue::response::Response;
use cosmwasm_std::{BankMsg, Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Order, Uint128};
use cw_storage_plus::Map;
use ownable::Ownable;
use redeemable::Redeemable;
use serde::{de::DeserializeOwned, Serialize};
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
        let ownable = &self.ownable.borrow();
        check_ownable(&deps.as_ref(), &env, &info, ownable)?;

        for (token_id, price) in listings {
            if price.amount > Uint128::new(0) {
                if let Ok(Some(_)) = self
                    .tokens
                    .borrow()
                    .contract
                    .tokens
                    .may_load(deps.storage, &token_id)
                {
                    self.listed_tokens
                        .save(deps.storage, token_id.as_str(), &price)?;
                } else {
                    return Err(ContractError::NoMetadataPresent);
                }
            }
        }
        Ok(Response::new().add_attribute("method", "list"))
    }

    pub fn try_buy_token(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response, ContractError> {
        // check if enough fee was sent
        match info.funds.as_slice() {
            [fund] => {
                let token = self
                    .listed_tokens
                    .load(deps.storage, token_id.as_str())
                    .map_err(|_| ContractError::NoListedTokensError);
                match token {
                    Ok(price) => {
                        if fund.denom.ne(&price.denom) {
                            return Err(ContractError::WrongFundError);
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
                                .update::<_, ContractError>(
                                    deps.storage,
                                    token_id.as_str(),
                                    |old| {
                                        let mut token_info = old.unwrap();
                                        token_info.owner = info.sender.clone();
                                        Ok(token_info)
                                    },
                                )?;
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
                            // TODO: Send royalties to minter
                            return Ok(Response::new().add_messages(messages));
                        } else {
                            return Err(ContractError::InsufficientFundsError {
                                fund: fund.amount,
                                seat_price: price.amount,
                            });
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            [] => return Err(ContractError::NoFundsPresent),
            _ => return Err(ContractError::MultipleFundsError),
        }
    }

    pub fn try_buy(
        &mut self,
        deps: &mut DepsMut,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let denom_name: String;
        if let Some(denom) = self.tokens.borrow().name.clone() {
            denom_name = denom;
        } else {
            return Err(ContractError::NoFundsPresent);
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
            if sorted_tokens.len() == 0 {
                return Err(ContractError::NoListedTokensError);
            }
            let lowest_listed_token = sorted_tokens.get(0).unwrap();
            let token_info = contract
                .tokens
                .load(deps.storage, lowest_listed_token.0.as_str())?;
            // TODO: In-efficient access. Get rid of cloning
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
                            lowest_price: lowest_price,
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
            return Err(ContractError::NoFundsPresent);
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
        let ownable = &self.ownable.borrow();
        let redeemable = &self.redeemable.borrow();

        check_ownable(&deps.as_ref(), &env, &info, ownable)?;
        for (token_id, price) in listings {
            if price.amount > Uint128::new(0) {
                if let Some(_) = self
                    .tokens
                    .borrow()
                    .contract
                    .tokens
                    .may_load(deps.storage, &token_id)
                    .unwrap()
                {
                    check_redeemable(&deps.as_ref(), &env, &info, &token_id, redeemable)?;
                    self.listed_tokens
                        .save(deps.storage, token_id.as_str(), &price)?;
                } else {
                    return Err(ContractError::NoMetadataPresent);
                }
            }
        }
        Ok(Response::new().add_attribute("method", "list"))
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
            [fund] => {
                let token = self
                    .listed_tokens
                    .load(deps.storage, token_id.as_str())
                    .map_err(|_| ContractError::NoListedTokensError);
                match token {
                    Ok(price) => {
                        if fund.denom.ne(&price.denom) {
                            return Err(ContractError::WrongFundError);
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
                                .update::<_, ContractError>(
                                    deps.storage,
                                    token_id.as_str(),
                                    |old| {
                                        let mut token_info = old.unwrap();
                                        token_info.owner = info.sender.clone();
                                        Ok(token_info)
                                    },
                                )?;
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
                    }
                    Err(err) => return Err(err),
                }
            }
            [] => return Err(ContractError::NoFundsPresent),
            _ => return Err(ContractError::MultipleFundsError),
        }
    }

    pub fn try_buy(
        &mut self,
        deps: DepsMut,
        env: &Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let denom_name: String;
        if let Some(denom) = self.tokens.borrow().name.clone() {
            denom_name = denom;
        } else {
            return Err(ContractError::NoFundsPresent);
        }
        let contract = &self.tokens.borrow_mut().contract;
        let redeemable = &self.redeemable.borrow();

        let maybe_coin = info.funds.iter().find(|&coin| coin.denom.eq(&denom_name));

        if let Some(coin) = maybe_coin {
            let limit = coin.amount;

            let mut sorted_tokens = self
                .listed_tokens
                .range(deps.storage, None, None, Order::Descending)
                .map(|t| t.unwrap())
                .collect::<Vec<(String, Coin)>>();
            sorted_tokens.sort_unstable_by_key(|t| t.1.amount);
            if sorted_tokens.len() == 0 {
                return Err(ContractError::NoListedTokensError);
            }
            let lowest_listed_token = sorted_tokens.get(0).unwrap();

            check_redeemable(
                &deps.as_ref(),
                env,
                &info,
                &lowest_listed_token.0,
                redeemable,
            )?;
            let token_info = contract
                .tokens
                .load(deps.storage, lowest_listed_token.0.as_str())?;
            // TODO: Get rid of cloning
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
                            lowest_price: lowest_price,
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
            return Err(ContractError::NoFundsPresent);
        }
    }
}

fn check_ownable(
    deps: &Deps,
    _env: &Env,
    info: &MessageInfo,
    ownable: &Ownable,
) -> Result<(), ContractError> {
    if ownable.is_owner(deps, &info.sender)? {
        return Ok(());
    }
    return Ok(());
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
    return Ok(());
}
