use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub allowed_addrs: Vec<Addr>,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddAllowedAddrs { addresses: Vec<Addr> },
    RemoveAllowedAddrs { addresses: Vec<Addr> },
    ClearAllAllowedAddrs {},
    SetEnabled { enabled: bool },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // IsAllowed returns whether or not the address is allowed or the allowlist is disabled
    IsAllowed { address: Addr },
    // IsAllowedAddr returns whether or not the address is in the allowlist
    IsAllowedAddr(Addr),
    // IsEnabled returns whether or not the contract is enforcing allowability check
    IsEnabled {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    // IsAllowed returns true if the address exists in the allowlist or the allowlist isnt enabled
    IsAllowed { is_allowed: bool },
    // IsAllowedAddr returns true if the address exists in the allowlist
    IsAllowedAddr(bool),
    // IsEnabled returns true if the contract is using an allowlist
    IsEnabled { is_enabled: bool },
}
