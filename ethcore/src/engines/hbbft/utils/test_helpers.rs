use client::traits::Nonce;
use client::{BlockQueueInfo, ChainSyncing, Client};
use engines::signer::from_keypair;
use ethereum_types::{Address, U256};
use ethkey::KeyPair;
use miner::Miner;
use miner::MinerService;
use spec::Spec;
use std::sync::Arc;
use test_helpers::generate_dummy_client_with_spec;
use test_helpers::TestNotify;
use types::ids::BlockId;
use types::transaction::{Action, SignedTransaction, Transaction, TypedTransaction};

pub fn hbbft_spec() -> Spec {
    Spec::load(
        &::std::env::temp_dir(),
        include_bytes!("../../../../res/honey_badger_bft.json") as &[u8],
    )
    .expect(concat!("Chain spec is invalid."))
}

struct SyncProviderWrapper();
impl ChainSyncing for SyncProviderWrapper {
    fn is_major_syncing(&self, queue_info: BlockQueueInfo) -> bool {
        false
    }
}

pub fn hbbft_client() -> std::sync::Arc<Client> {
    let client = generate_dummy_client_with_spec(hbbft_spec);
    /// @todo Implement set_sync_provider function
    client.set_sync_provider(Box::new(SyncProviderWrapper()));
    client
}

pub struct HbbftTestClient {
    pub client: Arc<Client>,
    pub notify: Arc<TestNotify>,
    pub miner: Arc<Miner>,
    pub keypair: KeyPair,
    pub nonce: U256,
}

pub fn create_hbbft_client(keypair: KeyPair) -> HbbftTestClient {
    let client = hbbft_client();
    let miner = client.miner();
    let engine = client.engine();
    let signer = from_keypair(keypair.clone());
    engine.set_signer(signer);
    engine.register_client(Arc::downgrade(&client) as _);
    let notify = Arc::new(TestNotify::default());
    client.add_notify(notify.clone());

    HbbftTestClient {
        client,
        notify,
        miner,
        keypair,
        nonce: U256::from(0),
    }
}

impl HbbftTestClient {
    pub fn transfer_to(&mut self, receiver: &Address, amount: &U256) {
        let transaction = create_transfer(&self.keypair, receiver, amount, &self.nonce);
        self.nonce += U256::from(1);
        self.miner
            .import_own_transaction(self.client.as_ref(), transaction.into(), false)
            .unwrap();
    }

    pub fn address(&self) -> Address {
        self.keypair.address()
    }

    pub fn call_as(
        &mut self,
        caller: &KeyPair,
        receiver: &Address,
        abi_call: ethabi::Bytes,
        amount: &U256,
    ) {
        let cur_nonce = self
            .client
            .nonce(
                &caller.address(),
                BlockId::Number(self.client.chain().best_block_number()),
            )
            .expect("Nonce for the current best block must always succeed");
        let transaction = create_call(caller, receiver, abi_call, amount, &cur_nonce);
        self.miner
            .import_claimed_local_transaction(self.client.as_ref(), transaction.into(), false)
            .unwrap();
    }
}

pub fn create_transfer(
    keypair: &KeyPair,
    receiver: &Address,
    amount: &U256,
    nonce: &U256,
) -> SignedTransaction {
    TypedTransaction::Legacy(Transaction {
        action: Action::Call(receiver.clone()),
        value: amount.clone(),
        data: vec![],
        gas: U256::from(100_000),
        gas_price: "10000000000".into(),
        nonce: *nonce,
    })
    .sign(keypair.secret(), None)
}

pub fn create_call(
    keypair: &KeyPair,
    receiver: &Address,
    abi_call: ethabi::Bytes,
    amount: &U256,
    nonce: &U256,
) -> SignedTransaction {
    TypedTransaction::Legacy(Transaction {
        action: Action::Call(receiver.clone()),
        value: amount.clone(),
        data: abi_call,
        gas: U256::from(900_000),
        gas_price: "10000000000".into(),
        nonce: *nonce,
    })
    .sign(keypair.secret(), None)
}
