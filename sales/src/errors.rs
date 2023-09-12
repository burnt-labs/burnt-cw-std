use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("StdError {0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Ongoing Primary Sale")]
    OngoingPrimarySaleError,

    #[error("No Active Primary Sale")]
    NoOngoingPrimarySaleError,

    #[error("Token Module Error {0}")]
    TokenModuleError(cw721_base::ContractError),

    #[error("No Funds")]
    NoFundsError,

    #[error("Multiple Funds")]
    MultipleFundsError,

    #[error("Wrong Fund Denom")]
    WrongFundError,

    #[error("Insufficient Funds")]
    InsufficientFundsError,

    #[error("Invalid Primary Sale parameter {0}")]
    InvalidPrimarySaleParamError(String),
}
