mod block_reward_hbbft;
mod contracts;
mod contribution;
mod hbbft_engine;
mod hbbft_state;
mod sealing;
mod utils;

pub use self::hbbft_engine::HoneyBadgerBFT;
use ethkey::Public;
use std::fmt;

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NodeId(pub Public);

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:6}", hex_fmt::HexFmt(&self.0))
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "NodeId({})", self.0)
    }
}

#[cfg(test)]
mod tests {

    use super::utils::test_helpers::create_hbbft_client;
    use client::traits::BlockInfo;
    use engines::hbbft::contracts::staking::tests::{create_staker, is_pool_active};
    use engines::hbbft::contracts::validator_set::{
        is_pending_validator, mining_by_staking_address,
    };
    use ethereum_types::{Address, U256};
    use ethkey::{Generator, KeyPair, Random, Secret};
    use std::str::FromStr;
    use types::ids::BlockId;

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

        // Verify that we actually start at block 0.
        assert_eq!(test_data.client.chain().best_block_number(), 0);

        // Inject a transaction, with instant sealing a block will be created right away.
        test_data.create_some_transaction(None);

        // Expect a new block to be created.
        assert_eq!(test_data.client.chain().best_block_number(), 1);

        // Expect one transaction in the block.
        let block = test_data
            .client
            .block(BlockId::Number(1))
            .expect("Block 1 must exist");
        assert_eq!(block.transactions_count(), 1);

        // Inject a transaction, with instant sealing a block will be created right away.
        test_data.create_some_transaction(None);

        // Expect a new block to be created.
        assert_eq!(test_data.client.chain().best_block_number(), 2);
    }
}
