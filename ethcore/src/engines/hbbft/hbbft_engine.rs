use client::EngineClient;
use engines::signer::EngineSigner;
use engines::{Engine, ForkChoice};
use error::Error;
use ethjson::spec::hbbft::HbbftParams;
use machine::EthereumMachine;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use types::header::{ExtendedHeader, Header};

pub struct HoneyBadgerBFT {
    client: Arc<RwLock<Option<Weak<dyn EngineClient>>>>,
    signer: Arc<RwLock<Option<Box<dyn EngineSigner>>>>,
    machine: EthereumMachine,
    params: HbbftParams,
}

impl HoneyBadgerBFT {
    pub fn new(params: HbbftParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
        let engine = Arc::new(HoneyBadgerBFT {
            client: Arc::new(RwLock::new(None)),
            signer: Arc::new(RwLock::new(None)),
            machine,
            params,
        });

        Ok(engine)
    }
}

impl Engine<EthereumMachine> for HoneyBadgerBFT {
    fn name(&self) -> &str {
        "HoneyBadgerBFT"
    }
    fn machine(&self) -> &EthereumMachine {
        &self.machine
    }
    fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
        Ok(())
    }
    fn fork_choice(&self, new: &ExtendedHeader, best: &ExtendedHeader) -> ForkChoice {
        // Forks should never, ever happen with HBBFT.
        ForkChoice::New
    }
}
