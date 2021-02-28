use client::EngineClient;
use engines::hbbft::contracts::validator_set::{get_validator_pubkeys, ValidatorType};
use engines::hbbft::utils::bound_contract::{BoundContract, CallError};
use engines::hbbft::NodeId;
use engines::EngineSigner;
use ethereum_types::{Address, H512, U256};
use ethkey::Public;
use hbbft::crypto::{PublicKeySet, SecretKeyShare};
use hbbft::sync_key_gen::{
    Ack, AckOutcome, Error, Part, PartOutcome, PubKeyMap, PublicKey, SecretKey, SyncKeyGen,
};
use hbbft::util::max_faulty;
use hbbft::NetworkInfo;
use itertools::Itertools;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use types::ids::BlockId;

use_contract!(
    key_history_contract,
    "res/contracts/key_history_contract.json"
);

lazy_static! {
    static ref KEYGEN_HISTORY_ADDRESS: Address =
        Address::from_str("7000000000000000000000000000000000000001").unwrap();
}

macro_rules! call_const_key_history {
	($c:ident, $x:ident $(, $a:expr )*) => {
		$c.call_const(key_history_contract::functions::$x::call($($a),*))
	};
}

#[derive(Clone)]
pub struct PublicWrapper {
    pub inner: Public,
}

#[derive(Clone)]
pub struct KeyPairWrapper {
    pub inner: Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
}

impl<'a> PublicKey for PublicWrapper {
    type Error = ethkey::crypto::Error;
    type SecretKey = KeyPairWrapper;
    fn encrypt<M: AsRef<[u8]>, R: rand_065::Rng>(
        &self,
        msg: M,
        _rng: &mut R,
    ) -> Result<Vec<u8>, Self::Error> {
        ethkey::crypto::ecies::encrypt(&self.inner, b"", msg.as_ref())
    }
}

impl<'a> SecretKey for KeyPairWrapper {
    type Error = ethkey::Error;
    fn decrypt(&self, ct: &[u8]) -> Result<Vec<u8>, Self::Error> {
        self.inner
            .read()
            .as_ref()
            .ok_or(ethkey::Error::InvalidSecret)
            .expect("Signer must be set!")
            .decrypt(b"", ct)
    }
}

pub fn engine_signer_to_synckeygen<'a>(
    signer: &Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
    pub_keys: PubKeyMap<Public, PublicWrapper>,
) -> Result<(SyncKeyGen<Public, PublicWrapper>, Option<Part>), Error> {
    let wrapper = KeyPairWrapper {
        inner: signer.clone(),
    };
    let public = match signer.read().as_ref() {
        Some(signer) => signer
            .public()
            .expect("Signer's public key must be available!"),
        None => Public::from(H512::new()),
    };
    let mut rng = rand_065::thread_rng();
    let num_nodes = pub_keys.len();
    SyncKeyGen::new(public, wrapper, pub_keys, max_faulty(num_nodes), &mut rng)
}

pub fn synckeygen_to_network_info(
    synckeygen: &SyncKeyGen<Public, PublicWrapper>,
    pks: PublicKeySet,
    sks: Option<SecretKeyShare>,
) -> Option<NetworkInfo<NodeId>> {
    let pub_keys = synckeygen
        .public_keys()
        .keys()
        .map(|p| NodeId(*p))
        .collect::<Vec<_>>();
    println!("Creating Network Info");
    println!("pub_keys: {:?}", pub_keys);
    println!(
        "pks: {:?}",
        (0..(pub_keys.len()))
            .map(|i| pks.public_key_share(i))
            .collect::<Vec<_>>()
    );
    let sks = sks.unwrap();
    println!("sks.public_key_share: {:?}", sks.public_key_share());
    println!("sks.reveal: {:?}", sks.reveal());

    Some(NetworkInfo::new(
        NodeId(synckeygen.our_id().clone()),
        sks,
        pks,
        pub_keys,
    ))
}

pub fn has_part_of_address_data(
    client: &dyn EngineClient,
    address: Address,
) -> Result<bool, CallError> {
    let c = BoundContract::bind(client, BlockId::Latest, *KEYGEN_HISTORY_ADDRESS);
    let serialized_part = call_const_key_history!(c, parts, address)?;
    //println!("Part for address {}: {:?}", address, serialized_part);
    Ok(!serialized_part.is_empty())
}

pub fn part_of_address(
    client: &dyn EngineClient,
    address: Address,
    vmap: &BTreeMap<Address, Public>,
    skg: &mut SyncKeyGen<Public, PublicWrapper>,
    block_id: BlockId,
) -> Result<Option<Ack>, CallError> {
    let c = BoundContract::bind(client, block_id, *KEYGEN_HISTORY_ADDRESS);
    let serialized_part = call_const_key_history!(c, parts, address)?;
    //println!("Part for address {}: {:?}", address, serialized_part);
    if serialized_part.is_empty() {
        return Err(CallError::ReturnValueInvalid);
    }
    let deserialized_part: Part = bincode::deserialize(&serialized_part).unwrap();
    let mut rng = rand_065::thread_rng();
    let outcome = skg
        .handle_part(vmap.get(&address).unwrap(), deserialized_part, &mut rng)
        .unwrap();

    match outcome {
        PartOutcome::Invalid(_) => Err(CallError::ReturnValueInvalid),
        PartOutcome::Valid(ack) => Ok(ack),
    }
}

pub fn has_acks_of_address_data(
    client: &dyn EngineClient,
    address: Address,
) -> Result<bool, CallError> {
    let c = BoundContract::bind(client, BlockId::Latest, *KEYGEN_HISTORY_ADDRESS);
    let serialized_length = call_const_key_history!(c, get_acks_length, address)?;
    Ok(serialized_length.low_u64() != 0)
}

pub fn acks_of_address(
    client: &dyn EngineClient,
    address: Address,
    vmap: &BTreeMap<Address, Public>,
    skg: &mut SyncKeyGen<Public, PublicWrapper>,
    block_id: BlockId,
) -> Result<(), CallError> {
    let c = BoundContract::bind(client, block_id, *KEYGEN_HISTORY_ADDRESS);
    let serialized_length = call_const_key_history!(c, get_acks_length, address)?;

    // println!(
    // 	"Acks for address {} is of size: {:?}",
    // 	address, serialized_length
    // );
    for n in 0..serialized_length.low_u64() {
        let serialized_ack = call_const_key_history!(c, acks, address, n)?;
        //println!("Ack #{} for address {}: {:?}", n, address, serialized_ack);
        if serialized_ack.is_empty() {
            return Err(CallError::ReturnValueInvalid);
        }
        let deserialized_ack: Ack = bincode::deserialize(&serialized_ack).unwrap();
        let outcome = skg
            .handle_ack(vmap.get(&address).unwrap(), deserialized_ack)
            .unwrap();
        if let AckOutcome::Invalid(fault) = outcome {
            panic!("Expected Ack Outcome to be valid. {}", fault);
        }
    }

    Ok(())
}

/// Read available keygen data from the blockchain and initialize a SyncKeyGen instance with it.
pub fn initialize_synckeygen(
    client: &dyn EngineClient,
    signer: &Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
    block_id: BlockId,
    validator_type: ValidatorType,
) -> Result<SyncKeyGen<Public, PublicWrapper>, CallError> {
    let vmap = get_validator_pubkeys(&*client, block_id, validator_type)?;
    let pub_keys: BTreeMap<_, _> = vmap
        .values()
        .map(|p| (*p, PublicWrapper { inner: p.clone() }))
        .collect();

    // if synckeygen creation fails then either signer or validator pub keys are problematic.
    // Todo: We should expect up to f clients to write invalid pub keys. Report and re-start pending validator set selection.
    let (mut synckeygen, _) = engine_signer_to_synckeygen(signer, Arc::new(pub_keys))
        .map_err(|_| CallError::ReturnValueInvalid)?;

    for v in vmap.keys().sorted() {
        part_of_address(&*client, *v, &vmap, &mut synckeygen, block_id)?;
    }
    for v in vmap.keys().sorted() {
        acks_of_address(&*client, *v, &vmap, &mut synckeygen, block_id)?;
    }

    Ok(synckeygen)
}
