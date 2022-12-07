use std::{cell::RefCell, rc::Rc};

use crate::{errors::ContractError, RSellable, Sellable};
use cosmwasm_std::{
    BankMsg, Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, Uint64,
};
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
        listed_tokens: Map<'a, &'a str, Uint64>,
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
        listings: schemars::Map<String, Uint64>,
    ) -> Result<Response, ContractError> {
        let ownable = &self.ownable.borrow();
        check_ownable(&deps.as_ref(), &env, &info, ownable)?;

        for (token_id, price) in listings {
            if price > Uint64::new(0) {
                self.listed_tokens
                    .save(deps.storage, token_id.as_str(), &price)?;
            }
        }
        Ok(Response::new().add_attribute("method", "list"))
    }

    pub fn try_buy(&mut self, deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
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

            let sorted_tokens = self
                .listed_tokens
                .range(deps.storage, None, None, Order::Descending)
                .map(|t| t.unwrap())
                .collect::<Vec<(String, Uint64)>>();
            if sorted_tokens.len() == 0 {
                return Err(ContractError::NoListedTokensError);
            }
            let lowest_listed_token = sorted_tokens.get(sorted_tokens.len() - 1).unwrap();
            let token_info = contract
                .tokens
                .load(deps.storage, lowest_listed_token.0.as_str())?;
            // TODO: In-efficient access. Get rid of cloning
            let lowest = Ok((
                lowest_listed_token.clone().0,
                token_info.owner,
                lowest_listed_token.1,
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

                    let payment_coin = Coin::new(lowest_price.u64() as u128, &denom_name);
                    let delta = limit - lowest_price;
                    let mut messages = vec![BankMsg::Send {
                        to_address: lowest_token_owner.to_string(),
                        amount: vec![payment_coin],
                    }];
                    if delta.u64() > 0 {
                        messages.push(BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![Coin::new(delta.u64() as u128, &denom_name)],
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
        listed_tokens: Map<'a, &'a str, Uint64>,
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
        listings: schemars::Map<String, Uint64>,
    ) -> Result<Response, ContractError> {
        let ownable = &self.ownable.borrow();
        let redeemable = &self.redeemable.borrow();

        check_ownable(&deps.as_ref(), &env, &info, ownable)?;
        for (token_id, price) in listings {
            if price > Uint64::new(0) {
                check_redeemable(&deps.as_ref(), &env, &info, &token_id, redeemable)?;
                self.listed_tokens
                    .save(deps.storage, token_id.as_str(), &price)?;
            }
        }
        Ok(Response::new().add_attribute("method", "list"))
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
            let limit = (coin.amount.u128() as u64).into();

            let sorted_tokens = self
                .listed_tokens
                .range(deps.storage, None, None, Order::Descending)
                .map(|t| t.unwrap())
                .collect::<Vec<(String, Uint64)>>();

            if sorted_tokens.len() == 0 {
                return Err(ContractError::NoListedTokensError);
            }
            let lowest_listed_token = sorted_tokens.get(sorted_tokens.len() - 1).unwrap();

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
                lowest_listed_token.1,
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

                    let payment_coin = Coin::new(lowest_price.u64() as u128, &denom_name);
                    let delta = limit - lowest_price;
                    let mut messages = vec![BankMsg::Send {
                        to_address: lowest_token_owner.to_string(),
                        amount: vec![payment_coin],
                    }];
                    if delta.u64() > 0 {
                        messages.push(BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![Coin::new(delta.u64() as u128, &denom_name)],
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
    let locked_tokens = redeemable.locked_tokens.load(deps.storage)?;
    if locked_tokens.contains(token_id) {
        return Err(ContractError::TicketRedeemed);
    }
    return Ok(());
}
