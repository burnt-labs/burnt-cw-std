use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllowableError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    // #[error("{0}")]
    // SerdeJson(#[from] serde_json::Error),
    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
