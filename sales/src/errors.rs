use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Ongoing Primary Sale")]
    OngoingPrimarySale,

    #[error("No Active Primary Sale")]
    NoOngoingPrimarySale,

    #[error("Token Module Error")]
    TokenModuleError(cw721_base::ContractError),
}
