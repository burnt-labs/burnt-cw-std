use crate::{errors::ContractError, msg::SellableTrait, Sellable};
use cosmwasm_std::{CustomMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, Uint64};
use cw721_base::Cw721Contract;
use ownable::Ownable;
use schemars::Map;
use serde::{de::DeserializeOwned, Serialize};

pub fn try_list<
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
    C: CustomMsg,
    Q: CustomMsg,
    E: CustomMsg,
>(
    deps: &mut DepsMut,
    env: Env,
    info: MessageInfo,
    listings: Map<String, Uint64>,
    sellable_module: &mut Sellable<T, C, E, Q>,
) -> Result<Response, ContractError> {
    let contract = &mut sellable_module.tokens.contract;
    let ownable = &mut sellable_module.ownable;
    for (token_id, price) in listings.iter() {
        verify_token(&deps.as_ref(), &env, &info, token_id, contract, ownable)?;
        contract
            .tokens
            .update::<_, ContractError>(deps.storage, token_id, |old| {
                old.ok_or(StdError::not_found("SellableToken").into())
                    .map(|old| {
                        let opt_price = if (*price) > Uint64::new(0) {
                            Some(*price)
                        } else {
                            None
                        };
                        // TODO: get rid of this unwrap
                        let meta = &old.extension;
                        meta.set_list_price(opt_price.unwrap());
                        old
                    })
            })?;
    }

    Ok(Response::new().add_attribute("method", "list"))
}

fn verify_token<
    T: Serialize + DeserializeOwned + Clone + SellableTrait,
    C: CustomMsg,
    Q: CustomMsg,
    E: CustomMsg,
>(
    deps: &Deps,
    _env: &Env,
    info: &MessageInfo,
    token_id: &String,
    contract: &mut Cw721Contract<T, C, E, Q>,
    ownable: &mut Ownable,
) -> Result<(), ContractError> {
    let token = contract.tokens.load(deps.storage, token_id)?;
    // confirm token aren't locked or redeemed
    let metadata = token.extension;
    if metadata.get_redeemed() {
        return Err(ContractError::TicketRedeemed);
    } else if metadata.get_locked() {
        return Err(ContractError::TicketLocked);
    }
    if ownable.is_owner(deps, &info.sender)? {
        return Ok(());
    }
    return Ok(());
}
