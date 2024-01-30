use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InvalidParam")]
    InvalidParam,

    #[error("Config is invalid")]
    InvalidConfig,

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("ConfigAlreadyExists")]
    ConfigAlreadyExists,

    #[error("SaleNoActive")]
    SaleNoActive,
}
