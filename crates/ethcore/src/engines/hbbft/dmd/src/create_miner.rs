use ethstore::{KeyFile, SafeAccount};
use parity_crypto::publickey::{Address, Generator, KeyPair, Public, Random, Secret};
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

fn create_account() -> (Secret, Public, Address) {
    let acc = Random.generate();
    (
        acc.secret().clone(),
        acc.public().clone(),
        acc.address().clone(),
    )
}

fn write_json_for_secret(secret: Secret, filename: &str) {
    let json_key: KeyFile = SafeAccount::create(
        &KeyPair::from_secret(secret).unwrap(),
        [0u8; 16],
        &"test".into(),
        NonZeroU32::new(10240).expect("We know 10240 is not zero."),
        "Test".to_owned(),
        "{}".to_owned(),
    )
    .expect("json key object creation should succeed")
    .into();

    let serialized_json_key =
        serde_json::to_string(&json_key).expect("json key object serialization should succeed");
    fs::write(filename, serialized_json_key).expect("Unable to write json key file");
}

pub fn create_miner() {
    println!("Creating dmd v4 miner...");
    let acc = create_account();

    // Create "data" and "network" subfolders.
    let network_key_dir = Path::new("./data/network");
    fs::create_dir_all(network_key_dir).expect("Could not create network key directory");
    // Write the private key for the hbbft node
    fs::write(network_key_dir.join("key"), acc.0.to_hex())
        .expect("Unable to write the network key file");

    // Create "keys" and "DPoSChain" subfolders.
    let accounts_dir = Path::new("./data/keys/DPoSChain");
    fs::create_dir_all(accounts_dir).expect("Could not create accounts directory");

    // Write JSON account.
    write_json_for_secret(
        acc.0,
        accounts_dir
            .join("dmd_miner_key.json")
            .to_str()
            .expect("Could not convert the JSON account path to a string"),
    );
    fs::write("password.txt", "test").expect("Unable to write password.txt file");
}
