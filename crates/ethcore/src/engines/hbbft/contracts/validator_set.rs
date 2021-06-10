use client::traits::{EngineClient, TransactionRequest};
use crypto::publickey::Public;
use engines::hbbft::utils::bound_contract::{BoundContract, CallError};
use ethereum_types::{Address, U256};
use std::{collections::BTreeMap, str::FromStr};
use types::ids::BlockId;
use client::BlockChainClient;

use_contract!(
    validator_set_hbbft,
    "res/contracts/validator_set_hbbft.json"
);

lazy_static! {
    static ref VALIDATOR_SET_ADDRESS: Address =
        Address::from_str("1000000000000000000000000000000000000001").unwrap();
}

macro_rules! call_const_validator {
	($c:ident, $x:ident $(, $a:expr )*) => {
		$c.call_const(validator_set_hbbft::functions::$x::call($($a),*))
	};
}

macro_rules! call_const_validator_no_args {
	($c:ident, $x:ident) => {
		$c.call_const(validator_set_hbbft::functions::$x::call())
	};
}


pub enum ValidatorType {
    Current,
    Pending,
}

pub fn get_validator_pubkeys(
    client: &dyn EngineClient,
    block_id: BlockId,
    validator_type: ValidatorType,
) -> Result<BTreeMap<Address, Public>, CallError> {
    let c = BoundContract::bind(client, block_id, *VALIDATOR_SET_ADDRESS);
    let validators = match validator_type {
        ValidatorType::Current => call_const_validator!(c, get_validators)?,
        ValidatorType::Pending => call_const_validator!(c, get_pending_validators)?,
    };
    let mut validator_map = BTreeMap::new();
    for v in validators {
        let pubkey = call_const_validator!(c, get_public_key, v)?;

        if pubkey.len() != 64 {
            return Err(CallError::ReturnValueInvalid);
        }
        let pubkey = Public::from_slice(&pubkey);

        //println!("Validator {:?} with public key {}", v, pubkey);
        validator_map.insert(v, pubkey);
    }
    Ok(validator_map)
}

#[cfg(test)]
pub fn mining_by_staking_address(
    client: &dyn EngineClient,
    staking_address: &Address,
) -> Result<Address, CallError> {
    let c = BoundContract::bind(client, BlockId::Latest, *VALIDATOR_SET_ADDRESS);
    call_const_validator!(c, mining_by_staking_address, staking_address.clone())
}

pub fn staking_by_mining_address(
 	client: &dyn EngineClient,
 	mining_address: &Address,
 ) -> Result<Address, CallError> {
 	let c = BoundContract::bind(client, BlockId::Latest, *VALIDATOR_SET_ADDRESS);
 	call_const_validator!(c, staking_by_mining_address, mining_address.clone())
 }

pub fn is_pending_validator(
    client: &dyn EngineClient,
    staking_address: &Address,
) -> Result<bool, CallError> {
    let c = BoundContract::bind(client, BlockId::Latest, *VALIDATOR_SET_ADDRESS);
    call_const_validator!(c, is_pending_validator, staking_address.clone())
}

pub fn get_validator_available_since(client: &dyn EngineClient, address: &Address)
	-> Result<U256, CallError> {
	let c = BoundContract::bind(client, BlockId::Latest, *VALIDATOR_SET_ADDRESS);
	call_const_validator!(c, validator_available_since, address.clone())
}

pub fn get_pending_validators(client: &dyn EngineClient) -> Result<Vec<Address>, CallError> {
    let c = BoundContract::bind(client, BlockId::Latest, *VALIDATOR_SET_ADDRESS);
    call_const_validator!(c, get_pending_validators)
}

pub fn send_tx_announce_availability(full_client: &dyn BlockChainClient, address: &Address) {

	let sendData =  validator_set_hbbft::functions::announce_availability::call();

	let transaction = TransactionRequest::call(*VALIDATOR_SET_ADDRESS, sendData.0)
		.gas(U256::from(250_000))
		.nonce(full_client.next_nonce(&address))
		.gas_price(U256::from(10000000000u64));

	full_client
		.transact_silently(transaction)
		.map_err(|_| CallError::ReturnValueInvalid);
}
