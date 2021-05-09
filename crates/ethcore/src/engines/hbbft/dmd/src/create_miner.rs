use ethstore::{KeyFile, SafeAccount};
use parity_crypto::publickey::{Generator, KeyPair, Random, Secret};
use std::{fs, num::NonZeroU32, path::Path};

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
    let acc = Random.generate();

    // Create "data" and "network" subfolders.
    let network_key_dir = Path::new("./data/network");
    fs::create_dir_all(network_key_dir).expect("Could not create network key directory");
    // Write the private key for the hbbft node
    fs::write(network_key_dir.join("key"), acc.secret().to_hex())
        .expect("Unable to write the network key file");

    // Create "keys" and "DPoSChain" subfolders.
    let accounts_dir = Path::new("./data/keys/DPoSChain");
    fs::create_dir_all(accounts_dir).expect("Could not create accounts directory");

    // Write JSON account.
    write_json_for_secret(
        acc.secret().clone(),
        accounts_dir
            .join("dmd_miner_key.json")
            .to_str()
            .expect("Could not convert the JSON account path to a string"),
    );
    fs::write("password.txt", "test").expect("Unable to write password.txt file");
    fs::write("public_key.txt", format!("{:?}", acc.public()))
        .expect("Unable to write password.txt file");

    println!("Miner address: {:?}", acc.address());
    println!("Miner public key: {:?}", acc.public());
}
