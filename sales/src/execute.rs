use std::{cell::RefCell, rc::Rc};

use burnt_glue::response::Response;
use cosmwasm_std::{BankMsg, Coin, CosmosMsg, CustomMsg, Deps, DepsMut, Env, MessageInfo, Uint64};
use cw721_base::{state::TokenInfo, MintMsg};
use cw_storage_plus::Item;
use ownable::Ownable;
use sellable::Sellable;
use serde::{de::DeserializeOwned, Serialize};

use crate::{errors::ContractError, msg::CreatePrimarySale, PrimarySale, Sales};

impl<'a, T, C, E, Q> Sales<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn new(
        sellable_module: Rc<RefCell<Sellable<'a, T, C, E, Q>>>,
        primary_sales: Item<'a, Vec<PrimarySale>>,
    ) -> Self {
        Self {
            sellable: sellable_module,
            primary_sales,
        }
    }

    pub fn add_primary_sales(
        &mut self,
        msg: CreatePrimarySale,
        deps: &mut DepsMut,
        env: Env,
        info: &MessageInfo,
    ) -> Result<Response, ContractError> {
        // basic validation on CreatePrimarySale struct
        if msg.start_time.lt(&Uint64::from(env.block.time.seconds())) {
            return Err(ContractError::InvalidPrimarySaleParamError(
                "start time".to_string(),
            ));
        }
        let ownable = &self.sellable.borrow().ownable;
        assert_owner(&deps.as_ref(), &env, &info, &ownable.borrow())?;
        // make sure no active primary sale
        let mut primary_sales = self.primary_sales.load(deps.storage).unwrap_or(vec![]);
        for sale in &primary_sales {
            if msg.start_time.le(&Uint64::from(sale.end_time.seconds())) {
                return Err(ContractError::InvalidPrimarySaleParamError(
                    "start time".to_string(),
                ));
            }
        }
        primary_sales.push(msg.into());
        self.primary_sales.save(deps.storage, &primary_sales)?;
        return Ok(Response::default());
    }

    pub fn halt_sale(&mut self, deps: &mut DepsMut, env: Env) -> Result<Response, ContractError> {
        let mut primary_sales = self.primary_sales.load(deps.storage)?;

        for sale in primary_sales.iter_mut() {
            if (!sale.disabled && sale.end_time.gt(&env.block.time))
                || sale.end_time.gt(&env.block.time)
            {
                sale.disabled = true;
                self.primary_sales.save(deps.storage, &primary_sales)?;
                return Ok(Response::default());
            }
        }
        return Err(ContractError::NoOngoingPrimarySaleError);
    }

    pub fn buy_item(
        &mut self,
        env: Env,
        deps: &mut DepsMut,
        info: MessageInfo,
        mint_msg: MintMsg<T>,
    ) -> Result<Response, ContractError> {
        // get current active sale
        let mut primary_sales = self.primary_sales.load(deps.storage).unwrap();

        for sale in primary_sales.iter_mut() {
            if !sale.disabled && sale.end_time.gt(&env.block.time) {
                if sale.tokens_minted.lt(&sale.total_supply) {
                    // check if enough fee was sent
                    let ownable = &self.sellable.borrow().ownable;
                    let paying_fund: &Coin;
                    if info.funds.len() > 1 {
                        return Err(ContractError::MultipleFundsError);
                    } else if sale.price.contains(&info.funds[0]) {
                        return Err(ContractError::WrongFundError);
                    } else {
                        paying_fund = sale
                            .price
                            .iter()
                            .find(|coin| coin.denom == info.funds[0].denom)
                            .unwrap();
                        if paying_fund.amount.gt(&info.funds[0].amount) {
                            return Err(ContractError::InsufficientFundsError);
                        }
                    }
                    // mint the item
                    let mut response = self.mint(deps, env, &info, mint_msg).unwrap();
                    sale.tokens_minted = sale
                        .tokens_minted
                        .checked_add(Uint64::from(1 as u8))
                        .unwrap();

                    if sale.tokens_minted.eq(&sale.total_supply) {
                        sale.disabled = true;
                    }
                    // send funds to creator
                    let message = BankMsg::Send {
                        to_address: ownable
                            .borrow()
                            .get_owner(&deps.as_ref())
                            .unwrap()
                            .to_string(),
                        amount: vec![Coin::new(
                            paying_fund.amount.u128(),
                            paying_fund.denom.clone(),
                        )],
                    };
                    let cosmos_msg = CosmosMsg::Bank(message);
                    response = response.add_message(cosmos_msg);

                    if paying_fund.amount.lt(&info.funds[0].amount) {
                        // refund user back extra funds
                        let refund_amount = info.funds[0]
                            .amount
                            .checked_sub(paying_fund.amount)
                            .unwrap();
                        let refund_message = BankMsg::Send {
                            to_address: info.sender.to_string(),
                            amount: vec![Coin::new(
                                refund_amount.u128(),
                                paying_fund.denom.clone(),
                            )],
                        };
                        let refund_cosmos_msg = CosmosMsg::Bank(refund_message);
                        response = response.add_message(refund_cosmos_msg);
                    }
                    self.primary_sales.save(deps.storage, &primary_sales)?;
                    return Ok(response);
                }
            }
        }
        return Err(ContractError::NoOngoingPrimarySaleError);
    }

    pub fn mint(
        &self,
        deps: &mut DepsMut,
        _env: Env,
        info: &MessageInfo,
        msg: MintMsg<T>,
    ) -> Result<Response, ContractError> {
        // create the token
        let token = TokenInfo {
            owner: deps.api.addr_validate(&msg.owner)?,
            approvals: vec![],
            token_uri: msg.token_uri,
            extension: msg.extension,
        };
        {
            self.sellable
                .borrow()
                .tokens
                .borrow_mut()
                .contract
                .tokens
                .update(deps.storage, &msg.token_id, |old| match old {
                    Some(_) => Err(ContractError::TokenModuleError(
                        cw721_base::ContractError::Claimed {},
                    )),
                    None => Ok(token),
                })?;
        }
        self.sellable
            .borrow()
            .tokens
            .borrow_mut()
            .contract
            .increment_tokens(deps.storage)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", &info.sender)
            .add_attribute("owner", msg.owner)
            .add_attribute("token_id", msg.token_id))
    }
}

fn assert_owner(
    deps: &Deps,
    _env: &Env,
    info: &MessageInfo,
    ownable: &Ownable,
) -> Result<(), ContractError> {
    if ownable.is_owner(deps, &info.sender)? {
        return Ok(());
    }
    return Err(ContractError::Unauthorized);
}
