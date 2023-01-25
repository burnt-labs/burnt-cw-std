use std::{cell::RefCell, rc::Rc};

use burnt_glue::response::Response;
use cosmwasm_std::{Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Timestamp, Uint64};
use cw721_base::MintMsg;
use cw_storage_plus::Item;
use ownable::Ownable;
use sellable::Sellable;
use serde::{de::DeserializeOwned, Serialize};

use crate::{errors::ContractError, state::PrimarySale, Sales};

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
        total_supply: Uint64,
        start_time: Uint64,
        end_time: Uint64,
        price: Coin,
        deps: &mut DepsMut,
        env: Env,
        info: &MessageInfo,
    ) -> Result<Response, ContractError> {
        check_ownable(
            &deps.as_ref(),
            &env,
            &info,
            &self.sellable.borrow().ownable.borrow(),
        )?;
        // make sure no active primary sale
        let mut primary_sales = self.primary_sales.load(deps.storage).unwrap();
        for sale in &primary_sales {
            if !sale.disabled || sale.end_time.gt(&env.block.time) {
                return Err(ContractError::OngoingPrimarySale);
            }
        }
        let primary_sale = PrimarySale {
            total_supply,
            tokens_minted: Uint64::from(0 as u8),
            start_time: Timestamp::from_seconds(start_time.u64()),
            end_time: Timestamp::from_seconds(end_time.u64()),
            price,
            disabled: false,
        };
        primary_sales.push(primary_sale);
        self.primary_sales.save(deps.storage, &primary_sales)?;
        return Ok(Response::default());
    }

    pub fn halt_sale(&mut self, deps: &mut DepsMut, env: Env) -> Result<Response, ContractError> {
        let mut primary_sales = self.primary_sales.load(deps.storage)?;

        for sale in primary_sales.iter_mut() {
            if !sale.disabled || sale.end_time.gt(&env.block.time) {
                sale.disabled = true;
                self.primary_sales.save(deps.storage, &primary_sales)?;
                return Ok(Response::default());
            }
        }
        return Err(ContractError::NoOngoingPrimarySale);
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
                    // buy the item
                    self.sellable
                        .borrow_mut()
                        .tokens
                        .borrow_mut()
                        .contract
                        .mint(deps.branch(), env.clone(), info.clone(), mint_msg)
                        .map_err(|err| ContractError::TokenModuleError(err))?;

                    sale.tokens_minted = sale
                        .tokens_minted
                        .checked_add(Uint64::from(1 as u8))
                        .unwrap();

                    if sale.tokens_minted.eq(&sale.total_supply) {
                        sale.disabled = true;
                    }
                    self.primary_sales.save(deps.storage, &primary_sales)?;
                    return Ok(Response::default());
                }
            }
        }
        return Err(ContractError::NoOngoingPrimarySale);
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
    return Err(ContractError::Unauthorized);
}
