use cosmwasm_std::Addr;
use schemars::{JsonSchema};
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
    AddAllowedAddrs(Vec<Addr>),
    RemoveAllowedAddrs(Vec<Addr>),
    ClearAllowedAddrs(),
    SetEnabled(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // IsAllowed returns whether or not the address is currently allowed
    IsAllowed(Addr),
    // IsEnabled returns whether or not the contract is enforcing allowability check
    IsEnabled(),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResp {
    // IsAllowed returns true if the address exists in the allowlist
    IsAllowed(bool),
    // IsEnabled returns true if the contract is using an allowlist
    IsEnabled(bool),
}
