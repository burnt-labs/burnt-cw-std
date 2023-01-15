use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
  pub number: i64
}

#[cw_serde]
pub enum ExecuteMsg {
  RegisterAddress(i64)
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  #[returns(Vec<String>)]
  ListAddresses {}
}
