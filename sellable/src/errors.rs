use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Redeemed")]
    TicketRedeemed,

    #[error("Locked")]
    TicketLocked,

    #[error("Missing required metadata")]
    NoMetadataPresent,

    #[error("Token already listed")]
    TokenAlreadyListed,

    #[error("Invalid listing price")]
    InvalidListingPrice,

    #[error("No tokens listed for sale")]
    NoListedTokensError,

    #[error("Limit of {limit} below lowest offer of {lowest_price}")]
    LimitBelowLowestOffer {
        limit: Uint128,
        lowest_price: Uint128,
    },

    #[error("Funds of {fund} below seat price of {seat_price}")]
    InsufficientFundsError { fund: Uint128, seat_price: Uint128 },

    #[error("No relevant funds present in transaction")]
    NoFundsPresent,

    #[error("Multiple funds present")]
    MultipleFundsError,

    #[error("Wrong fund in transaction")]
    WrongFundError,

    #[error("Token ID not found")]
    TokenIDNotFoundError,

    #[error("{0}")]
    BaseError(#[from] cw721_base::ContractError),
}
