use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Generic(String),

    #[error("Mismatched asset type sent and distributed")]
    MismatchedAssetType {},

    #[error("Mismatched asset amount sent and distributed")]
    MismatchedAssetAmount {},

    #[error("Duplicate recipient in list")]
    DuplicateRecipient {},
}
