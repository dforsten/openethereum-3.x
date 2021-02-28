mod contracts;
mod hbbft_engine;
mod utils;

pub use self::hbbft_engine::HoneyBadgerBFT;
use ethkey::Public;

#[derive(
    Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, Debug,
)]
pub struct NodeId(pub Public);

#[cfg(test)]
mod tests {

    use super::utils::test_helpers::create_hbbft_client;
    use ethkey::{KeyPair, Secret};
    use std::str::FromStr;

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
