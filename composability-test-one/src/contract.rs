#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg, to_binary, CosmosMsg, Empty, Order, Addr};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::PEOPLE_WHO_SAID_FOUR;
use composability_test_two::msg::ExecuteMsg as CompanionExecuteMsg;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:composability-test-one";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

const COMPANION_ADDR: &str = "burnt1rdnpk3jp6z96zgxn8dx36dh4r6tx0crlr6ajvqjt2mhgrr5gqtushx6cfv";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let companion_msg = CompanionExecuteMsg::ValidateNumber(msg.number);
    let companion_msg_bytes = to_binary(&companion_msg)?;
    let wasm_msg = WasmMsg::Execute {
        contract_addr: COMPANION_ADDR.to_string(),
        msg: companion_msg_bytes,
        funds: Vec::new()
    };
    let cosmos_msg = CosmosMsg::Wasm(wasm_msg);
    PEOPLE_WHO_SAID_FOUR.save(deps.storage, info.sender, &Empty::default())?;

    Ok(Response::new().add_message(cosmos_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterAddress(number) => {
            let companion_msg = CompanionExecuteMsg::ValidateNumber(number);
            let companion_msg_bytes = to_binary(&companion_msg)?;
            let wasm_msg = WasmMsg::Execute {
                contract_addr: COMPANION_ADDR.to_string(),
                msg: companion_msg_bytes,
                funds: Vec::new()
            };
            let cosmos_msg = CosmosMsg::Wasm(wasm_msg);
            PEOPLE_WHO_SAID_FOUR.save(deps.storage, info.sender, &Empty::default())?;

            Ok(Response::new().add_message(cosmos_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    let iter = PEOPLE_WHO_SAID_FOUR.keys(deps.storage, None, None, Order::Ascending);
    let addrs: Vec<String> = iter.flat_map(|result| result.map(|addr| addr.to_string())).collect();
    let payload = to_binary(&addrs)?;

    Ok(payload)
}

#[cfg(test)]
mod tests {}
