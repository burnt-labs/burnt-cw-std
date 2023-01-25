use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Ongoing Primary Sale Error")]
    OngoingPrimarySaleError,

    #[error("No Active Primary Sale Error")]
    NoOngoingPrimarySaleError,

    #[error("Token Module Error")]
    TokenModuleError(cw721_base::ContractError),

    #[error("Multiple Funds Error")]
    MultipleFundsError,

    #[error("Wrong Fund Denom Error")]
    WrongFundError,

    #[error("Insufficient Funds Error")]
    InsufficientFundsError
}
