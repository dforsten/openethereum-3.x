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

    #[test]
    fn test_staking_account_creation() {
        // Create Master of Ceremonies
        let mut moc = create_hbbft_client(MASTER_OF_CEREMONIES_KEYPAIR.clone());

        // Verify the master of ceremony is funded.
        assert!(moc.balance(&moc.address()) > U256::from(10000000));

        // Create a potential validator.
        let miner_1 =
            create_hbbft_client(Random.generate().expect("KeyPair generation must succeed."));

        // Verify the pending validator is unfunded.
        assert_eq!(moc.balance(&miner_1.address()), U256::from(0));

        // Verify that we actually start at block 0.
        assert_eq!(moc.client.chain().best_block_number(), 0);

        let transaction_funds = U256::from(9000000000000000000u64);

        // Inject a transaction, with instant sealing a block will be created right away.
        moc.transfer_to(&miner_1.address(), &transaction_funds);

        // Expect a new block to be created.
        assert_eq!(moc.client.chain().best_block_number(), 1);

        // Verify the pending validator is now funded.
        assert_eq!(moc.balance(&miner_1.address()), transaction_funds);

        // Create staking address
        let staker_1 = create_staker(&mut moc, &miner_1, transaction_funds);

        // Expect two new blocks to be created, one for the transfer of staking funds,
        // one for registering the staker as pool.
        assert_eq!(moc.client.chain().best_block_number(), 3);

        // Expect one transaction in the block.
        let block = moc
            .client
            .block(BlockId::Number(3))
            .expect("Block must exist");
        // @todo Investigate why this block has two transactions - we only expect one.
        //assert_eq!(block.transactions_count(), 1);

        assert_ne!(
            mining_by_staking_address(moc.client.as_ref(), &staker_1.address())
                .expect("Constant call must succeed."),
            Address::zero()
        );

        // Check if the staking pool is active.
        assert_eq!(
            is_pool_active(moc.client.as_ref(), staker_1.address())
                .expect("Pool active query must succeed."),
            true
        );
    }
}
