// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! A signer used by Engines which need to sign messages.

use ethereum_types::{Address, H256};
use ethkey::{self, crypto::ecies, Public, Signature};

/// Everything that an Engine needs to sign messages.
pub trait EngineSigner: Send + Sync {
    /// Sign a consensus message hash.
    fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error>;

    /// Signing address
    fn address(&self) -> Address;

    /// Decrypt a message that was encrypted to this signer's key.
    fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::Error>;

    /// The signer's public key, if available.
    fn public(&self) -> Option<Public>;
}

/// Creates a new `EngineSigner` from given key pair.
pub fn from_keypair(keypair: ethkey::KeyPair) -> Box<dyn EngineSigner> {
    Box::new(Signer(keypair))
}

struct Signer(ethkey::KeyPair);

impl EngineSigner for Signer {
    fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error> {
        ethkey::sign(self.0.secret(), &hash)
    }

    fn address(&self) -> Address {
        self.0.address()
    }

    fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::Error> {
        ecies::decrypt(self.0.secret(), auth_data, cipher).map_err(|e| match e {
            ethkey::crypto::Error::Secp(e) => ethkey::Error::InvalidSecret,
            ethkey::crypto::Error::Io(e) => ethkey::Error::Io(e),
            ethkey::crypto::Error::InvalidMessage => ethkey::Error::InvalidMessage,
            ethkey::crypto::Error::Symm(_) => ethkey::Error::InvalidSecret,
        })
    }

    fn public(&self) -> Option<Public> {
        Some(*self.0.public())
    }
}

#[cfg(test)]
mod test_signer {
    use std::sync::Arc;

    use accounts::{self, AccountProvider, SignError};
    use ethkey::Password;

    use super::*;

    impl EngineSigner for (Arc<AccountProvider>, Address, Password) {
        fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error> {
            match self.0.sign(self.1, Some(self.2.clone()), hash) {
                Err(SignError::NotUnlocked) => unreachable!(),
                Err(SignError::NotFound) => Err(ethkey::Error::InvalidAddress),
                Err(SignError::SStore(accounts::Error::EthKey(err))) => Err(err),
                Err(SignError::SStore(accounts::Error::EthKeyCrypto(err))) => {
                    warn!("Low level crypto error: {:?}", err);
                    Err(ethkey::Error::InvalidSecret)
                }
                Err(SignError::SStore(err)) => {
                    warn!("Error signing for engine: {:?}", err);
                    Err(ethkey::Error::InvalidSignature)
                }
                Ok(ok) => Ok(ok),
            }
        }

        fn address(&self) -> Address {
            self.1
        }

        fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::Error> {
            self.0
                .decrypt(self.1, Some(self.2.clone()), auth_data, cipher)
                .map_err(|e| ethkey::Error::Custom(e.to_string()))
        }

        fn public(&self) -> Option<Public> {
            self.0.account_public(self.1, &self.2).ok()
        }
    }
}
