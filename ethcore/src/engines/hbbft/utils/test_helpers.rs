use crate::client::Client;
use crate::engines::signer::from_keypair;
use crate::miner::Miner;
use crate::test_helpers::generate_dummy_client_with_spec;
use ethereum_types::U256;
use ethkey::KeyPair;
use spec::Spec;
use std::sync::Arc;
use test_helpers::TestNotify;

pub fn hbbft_spec() -> Spec {
    Spec::load(
        &::std::env::temp_dir(),
        include_bytes!("../../../../res/honey_badger_bft.json") as &[u8],
    )
    .expect(concat!("Chain spec is invalid."))
}

pub fn hbbft_client() -> std::sync::Arc<Client> {
    let client = generate_dummy_client_with_spec(hbbft_spec);
    /// @todo Implement set_sync_provider function
    //client.set_sync_provider(Box::new(SyncProviderWrapper()));
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
