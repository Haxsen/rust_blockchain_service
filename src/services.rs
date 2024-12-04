use ethers::prelude::*;
use std::sync::Arc;
use crate::utils::{get_stake_contract_addr, initialize_client};

abigen!(
    StakeContract,
    "./abi/StakeContract.abi", // Path to the ABI for interacting with the smart contract
    event_derives(serde::Deserialize, serde::Serialize)
);

// Deposit to contract function
pub async fn deposit_to_contract(privatekey: String, amount: U256) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = initialize_client(privatekey)?;
    let contract_addr: Address = get_stake_contract_addr();
    let contract = StakeContract::new(contract_addr, Arc::new(client));
    let tx = contract.deposit().value(amount).send().await?.await?;
    let tr = serde_json::to_string(&tx)?;
    Ok(Some(tr))
}

// Withdraw from contract function
pub async fn withdraw_from_contract(privatekey: String, amount: U256) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = initialize_client(privatekey)?;
    let contract_addr: Address = get_stake_contract_addr();
    let contract = StakeContract::new(contract_addr, Arc::new(client));
    let tx = contract.withdraw(amount).send().await?.await?;
    let tr = serde_json::to_string(&tx)?;
    Ok(Some(tr))
}

// Check contract balance function
pub async fn check_contract_balance(privatekey: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = initialize_client(privatekey)?;
    let contract_addr: Address = get_stake_contract_addr();
    let contract = StakeContract::new(contract_addr, Arc::new(client));
    let value = contract.check_balance().call().await?;
    Ok(Some(value.to_string()))
}
