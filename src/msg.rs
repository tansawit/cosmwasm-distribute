use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128};
use cw20::Cw20ReceiveMsg;


/// ## Description
/// This structure stores the basic settings for creating a new contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

/// ## Description
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Receive calls a hook message after receiving CW20 asset.
    Receive(Cw20ReceiveMsg),
    /// Distribute native SDK tokens
    DistributeNative {
        /// Coin denom to send
        denom: String,
        /// List of individual recipient addresses and amount
        recipients: Vec<Recipient>,
    },
}

/// ## Description
/// This structure stores the recipient structure of the distribution
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Recipient {
    /// Address of the individual recipient
    pub recipient: String,
    /// Amount of assets the individual recipient will receive
    pub amount: Uint128,
}

/// ## Description
/// This structure describes the possible hook messages for CW20 contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DistributeCw20 {
        /// Address of CW20 token contract to send
        asset_token: String,
        /// List of individual recipient addresses and amount
        recipients: Vec<Recipient>,
    },
}

/// ## Description
/// A struct used for migrating contracts.
/// Currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

