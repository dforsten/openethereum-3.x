use super::contracts::staking::tests::{create_staker, is_pool_active};
use super::contracts::staking::{get_posdao_epoch, start_time_of_next_phase_transition};
use super::contracts::validator_set::{is_pending_validator, mining_by_staking_address};
use super::contribution::unix_now_secs;
use super::test::test_helpers::create_hbbft_client;
use client::traits::BlockInfo;
use crypto::publickey::{Generator, KeyPair, Random, Secret};
use ethereum_types::{Address, U256};
use std::str::FromStr;
use types::ids::BlockId;

pub mod test_helpers;

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
}

#[test]
fn test_staking_account_creation() {
    // Create Master of Ceremonies
    let mut moc = create_hbbft_client(MASTER_OF_CEREMONIES_KEYPAIR.clone());

    // Verify the master of ceremony is funded.
    assert!(moc.balance(&moc.address()) > U256::from(10000000));

    // Create a potential validator.
    let miner_1 = create_hbbft_client(Random.generate());

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
    assert_eq!(block.transactions_count(), 2);

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

#[test]
fn test_epoch_transition() {
    // Create Master of Ceremonies
    let mut moc = create_hbbft_client(MASTER_OF_CEREMONIES_KEYPAIR.clone());
    // To avoid performing external transactions with the MoC we create and fund a random address.
    let transactor: KeyPair = Random.generate();

    let genesis_transition_time = start_time_of_next_phase_transition(moc.client.as_ref())
        .expect("Constant call must succeed");

    // Genesis block is at time 0, current unix time must be much larger.
    assert!(genesis_transition_time.as_u64() < unix_now_secs());

    // We should not be in the pending validator set at the genesis block.
    assert!(!is_pending_validator(moc.client.as_ref(), &moc.address())
        .expect("Constant call must succeed"));

    // Fund the transactor.
    // Also triggers the creation of a block.
    // This implicitly calls the block reward contract, which should trigger a phase transition
    // since we already verified that the genesis transition time threshold has been reached.
    let transaction_funds = U256::from(9000000000000000000u64);
    moc.transfer_to(&transactor.address(), &transaction_funds);

    // Expect a new block to be created.
    assert_eq!(moc.client.chain().best_block_number(), 1);

    // Now we should be part of the pending validator set.
    assert!(is_pending_validator(moc.client.as_ref(), &moc.address())
        .expect("Constant call must succeed"));

    // Check if we are still in the first epoch.
    assert_eq!(
        get_posdao_epoch(moc.client.as_ref(), BlockId::Latest).expect("Constant call must succeed"),
        U256::from(0)
    );

    // First the validator realizes it is in the next validator set and sends his part.
    moc.create_some_transaction(Some(&transactor));

    // The part will be included in the block triggered by this transaction, but not part of the global state yet,
    // so it sends the transaction another time.
    moc.create_some_transaction(Some(&transactor));

    // Now the part is part of the global chain state, and we send our acks.
    moc.create_some_transaction(Some(&transactor));

    // The acks will be included in the block triggered by this transaction, but not part of the global state yet.
    moc.create_some_transaction(Some(&transactor));

    // Now the acks are part of the global block state, and the key generation is complete and the next epoch begins
    moc.create_some_transaction(Some(&transactor));

    // At this point we should be in the new epoch.
    assert_eq!(
        get_posdao_epoch(moc.client.as_ref(), BlockId::Latest).expect("Constant call must succeed"),
        U256::from(1)
    );

    // Let's do another one to check if the transition to the new honey badger and keys works.
    moc.create_some_transaction(Some(&transactor));
}
