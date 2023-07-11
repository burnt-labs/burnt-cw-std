use std::{cell::RefCell, rc::Rc};

use burnt_glue::response::Response;
use cosmwasm_std::{
    BankMsg, Coin, CosmosMsg, CustomMsg, Deps, DepsMut, Env, MessageInfo, Timestamp, Uint64,
};
use cw721_base::{state::TokenInfo, MintMsg};
use cw_storage_plus::Item;
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

    pub fn add_primary_sale(
        &mut self,
        msg: CreatePrimarySale,
        deps: &mut DepsMut,
        env: Env,
        info: &MessageInfo,
    ) -> Result<Response, ContractError> {
        // can not add a sale that starts in the past
        if msg.start_time.lt(&Uint64::from(env.block.time.seconds())) {
            return Err(ContractError::InvalidPrimarySaleParamError(
                "start time".to_string(),
            ));
        } else if !msg.end_time.gt(&msg.start_time) {
            // cannot add a sale that ends before it starts
            return Err(ContractError::InvalidPrimarySaleParamError(
                "end time".to_string(),
            ));
        }

        // validate contract owner
        if info.sender
            != self
                .sellable
                .borrow()
                .ownable
                .borrow()
                .owner
                .load(deps.storage)?
        {
            return Err(ContractError::Unauthorized);
        }

        // make sure no active primary sale
        let start_time = Timestamp::from_seconds(msg.start_time.u64());
        let end_time = Timestamp::from_seconds(msg.end_time.u64());
        let mut primary_sales = self.primary_sales.load(deps.storage).unwrap_or(vec![]);
        for sale in &primary_sales {
            // can't add a sale that overlaps with the start or end of another sale
            if check_events_overlap(start_time, end_time, sale.start_time, sale.end_time) {
                return Err(ContractError::InvalidPrimarySaleParamError(
                    "overlap".to_string(),
                ));
            }
        }
        primary_sales.push(msg.into());
        self.primary_sales.save(deps.storage, &primary_sales)?;
        Ok(Response::default())
    }

    pub fn halt_sale(
        &mut self,
        deps: &mut DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        if info.sender
            != self
                .sellable
                .borrow()
                .ownable
                .borrow()
                .owner
                .load(deps.storage)?
        {
            return Err(ContractError::Unauthorized);
        }

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
        Err(ContractError::NoOngoingPrimarySaleError)
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
            if !sale.disabled // if the sale is not disabled
                && sale.end_time.gt(&env.block.time) // and the sale has not ended
                && (sale.tokens_minted.lt(&sale.total_supply) // and tokens haven't hit their supply cap
                    || sale.total_supply.eq(&Uint64::from(0_u8)))
            // or the supply cap is 0 (unlimited)
            {
                // check if enough fee was sent
                if info.funds.len() == 0 {
                    return Err(ContractError::NoFundsError);
                } else if info.funds.len() > 1 {
                    return Err(ContractError::MultipleFundsError);
                } else {
                    if info.funds[0].denom.ne(&sale.price[0].denom) {
                        return Err(ContractError::WrongFundError);
                    }
                    if info.funds[0].amount.lt(&info.funds[0].amount) {
                        return Err(ContractError::InsufficientFundsError);
                    }
                }
                // mint the item
                let mut response = self.mint(deps, env, &info, mint_msg).unwrap();
                sale.tokens_minted = sale.tokens_minted.checked_add(Uint64::from(1_u8)).unwrap();

                if sale.tokens_minted.eq(&sale.total_supply) {
                    sale.disabled = true;
                }
                // send funds to creator
                let ownable = &self.sellable.borrow().ownable;
                let message = BankMsg::Send {
                    to_address: ownable
                        .borrow()
                        .get_owner(&deps.as_ref())
                        .unwrap()
                        .to_string(),
                    amount: vec![Coin::new(
                        sale.price[0].amount.u128(),
                        sale.price[0].denom.clone(),
                    )],
                };
                let cosmos_msg = CosmosMsg::Bank(message);
                response = response.add_message(cosmos_msg);

                if sale.price[0].amount.lt(&info.funds[0].amount) {
                    // refund user back extra funds
                    let refund_amount = info.funds[0]
                        .amount
                        .checked_sub(sale.price[0].amount)
                        .unwrap();
                    let refund_message = BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin::new(refund_amount.u128(), sale.price[0].denom.clone())],
                    };
                    let refund_cosmos_msg = CosmosMsg::Bank(refund_message);
                    response = response.add_message(refund_cosmos_msg);
                }
                self.primary_sales.save(deps.storage, &primary_sales)?;
                return Ok(response);
            }
        }
        Err(ContractError::NoOngoingPrimarySaleError)
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

fn check_events_overlap(
    a_start: Timestamp,
    a_end: Timestamp,
    b_start: Timestamp,
    b_end: Timestamp,
) -> bool {
    // first event starts during second event
    if a_start.ge(&b_start) && a_start.le(&b_end) {
        return true;
    }

    // first event ends during second event
    if a_end.ge(&b_start) && a_end.le(&b_end) {
        return true;
    }

    // second event starts during first event
    if b_start.ge(&a_start) && b_start.le(&a_end) {
        return true;
    }

    // second event ends during first event
    if b_end.ge(&a_start) && b_end.le(&a_end) {
        return true;
    }

    false
}
