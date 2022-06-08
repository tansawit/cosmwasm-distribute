#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    coins, from_binary, to_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg,
    Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, Recipient};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "nebula-airdrop";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

/// ## Description
/// Exposes all the execute functions available in the contract.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **msg** is an object of type [`ExecuteMsg`].
///
/// ## Commands
/// - **ExecuteMsg::Receive (msg)** Receives CW20 tokens and executes a hook message.
///
/// - **ExecuteMsg::DistributeNative {
///             denom,
///             recipients,
///         }** Distributes native tokenss.
///
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::DistributeNative {
            denom,
            recipients,
        } => try_distribute_native(deps, info, denom, recipients),
    }
}


/// ## Description
/// Receives CW20 tokens and executes a hook message.
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **cw20_msg** is an object of type [`Cw20ReceiveMsg`] which is a hook message to be executed.
pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeCw20 {
            asset_token,
            recipients,
        }) => {
            if info.sender.to_string() != asset_token {
                return Err(ContractError::MismatchedAssetType {});
            };
            try_distribute_cw20(deps, cw20_msg.amount, asset_token, recipients)
        }
        Err(_) => Err(ContractError::Generic("invalid cw20 hook message".to_string())),
    }
}

/// ## Description
/// Handles distribution of CW20 tokens
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **amount** is an object of type [`Uint128`] which is the amount of tokens to be distributed.
///
/// - **asset_token** is an object of type [`String`] which is the contract address of the CW20 token to distribute.
///
/// - **recipients** is an object of type [`Vec<Recipient>`] which is the list of recipient address and amount to distribute to.
pub fn try_distribute_cw20(
    deps: DepsMut,
    amount: Uint128,
    asset_token: String,
    recipients: Vec<Recipient>,
) -> Result<Response, ContractError> {
    // validate sent coin amount matches sum(recipient amounts)
    let sum_recipient_amount: Uint128 =
        recipients.iter().fold(Uint128::zero(), |sum, recipient| sum + recipient.amount);
    if amount != sum_recipient_amount {
        return Err(ContractError::MismatchedAssetAmount {});
    }

    // check for duplicate recipient address
    if (1..recipients.len()).any(|i| recipients[i..].contains(&recipients[i - 1])) {
        return Err(ContractError::DuplicateRecipient {});
    }

    // construct transfer messsage vector
    let mut transfer_msgs: Vec<SubMsg> = vec![];
    for recipient in recipients.iter() {
        deps.api.addr_validate(&recipient.recipient)?;

        transfer_msgs.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: asset_token.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.recipient.clone(),
                amount: recipient.amount,
            })
            .unwrap(),
        }))
    }
    Ok(Response::new().add_submessages(transfer_msgs))
}

/// ## Description
/// Handles distribution of native Cosmos SDK coins
///
/// ## Params
/// - **deps** is an object of type [`DepsMut`].
///
/// - **info** is an object of type [`MessageInfo`].
///
/// - **denom** is an object of type [`String`] which is the denomination of the native token to distribute.
///
/// - **recipients** is an object of type [`Vec<Recipient>`] which is the list of recipient address and amount to distribute to.
pub fn try_distribute_native(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    recipients: Vec<Recipient>,
) -> Result<Response, ContractError> {
    // validate sent coin denom
    let mut amount = Uint128::zero();
    for coin in info.funds.iter() {
        if coin.denom != denom {
            return Err(ContractError::MismatchedAssetType {});
        } else {
            amount = coin.amount;
        }
    }
    let sum_recipient_amount: Uint128 =
        recipients.iter().fold(Uint128::zero(), |sum, recipient| sum + recipient.amount);

    // validate sent coin amount matches sum(recipient amounts)
    if amount != sum_recipient_amount {
        return Err(ContractError::MismatchedAssetAmount {});
    }

    // check for duplicate recipient address
    if (1..recipients.len()).any(|i| recipients[i..].contains(&recipients[i - 1])) {
        return Err(ContractError::DuplicateRecipient {});
    }

    // construct transfer messsage vector
    let mut transfer_msgs: Vec<SubMsg> = vec![];
    for recipient in recipients.iter() {
        deps.api.addr_validate(&recipient.recipient)?;

        transfer_msgs.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.recipient.clone(),
            amount: coins(recipient.amount.into(), denom.clone()),
        })))
    }

    Ok(Response::new().add_submessages(transfer_msgs))
}

/// ## Description
/// Exposes the migrate functionality in the contract.
///
/// ## Params
/// - **_deps** is an object of type [`DepsMut`].
///
/// - **_env** is an object of type [`Env`].
///
/// - **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
