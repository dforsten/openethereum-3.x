mod utils;
mod hbbft_engine;

pub use self::hbbft_engine::HoneyBadgerBFT;

#[cfg(test)]
mod tests {

    use ethkey::{KeyPair, Secret};
    use std::str::FromStr;
    use super::utils::test_helpers::create_hbbft_client;

    lazy_static! {
        static ref MASTER_OF_CEREMONIES_KEYPAIR: KeyPair = KeyPair::from_secret(
            Secret::from_str("18f059a4d72d166a96c1edfb9803af258a07b5ec862a961b3a1d801f443a1762")
                .expect("Secret from hex string must succeed")
        )
        .expect("KeyPair generation from secret must succeed");
    }

    #[test]
    fn test_miner_transaction_injection() {
        let mut test_data = create_hbbft_client(MASTER_OF_CEREMONIES_KEYPAIR.clone());
    }
}
