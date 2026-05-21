//! Native Vectory wallet storage and signing.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
#[cfg(test)]
use ed25519_dalek::{Signature, Verifier};
use ed25519_dalek::{Signer, SigningKey};
use eyre::{Result, WrapErr};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::agent_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub private_key_hex: String,
    pub public_key: String,
    pub address: String,
}

impl Wallet {
    pub fn create() -> Self {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        Self::from_signing_key(signing_key)
    }

    #[cfg(test)]
    pub fn from_private_key_hex(private_key_hex: &str) -> Result<Self> {
        let bytes = hex::decode(private_key_hex).wrap_err("Invalid wallet private key hex")?;
        let private_key: [u8; 32] = bytes
            .try_into()
            .map_err(|_| eyre::eyre!("Wallet private key must be 32 bytes"))?;
        let signing_key = SigningKey::from_bytes(&private_key);
        Ok(Self::from_signing_key(signing_key))
    }

    pub fn sign(&self, message: &[u8]) -> String {
        let signing_key = self.signing_key();
        let signature = signing_key.sign(message);
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    }

    #[cfg(test)]
    pub fn verify(&self, message: &[u8], signature: &str) -> bool {
        let Ok(bytes) = URL_SAFE_NO_PAD.decode(signature) else {
            return false;
        };
        let Ok(signature) = Signature::from_slice(&bytes) else {
            return false;
        };
        self.signing_key()
            .verifying_key()
            .verify(message, &signature)
            .is_ok()
    }

    fn from_signing_key(signing_key: SigningKey) -> Self {
        let private_key_hex = hex::encode(signing_key.to_bytes());
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let public_key = URL_SAFE_NO_PAD.encode(public_key_bytes);
        let address = format!("vec1{}", public_key);
        Self {
            private_key_hex,
            public_key,
            address,
        }
    }

    fn signing_key(&self) -> SigningKey {
        let bytes = hex::decode(&self.private_key_hex).expect("wallet private key hex");
        let private_key: [u8; 32] = bytes.try_into().expect("wallet private key length");
        SigningKey::from_bytes(&private_key)
    }
}

fn wallet_path(agent_name: &str) -> PathBuf {
    agent_dir(agent_name).join("wallet.json")
}

pub fn create_wallet(agent_name: &str) -> Result<Wallet> {
    let path = wallet_path(agent_name);
    if path.exists() {
        return Err(eyre::eyre!("Wallet already exists at {}", path.display()));
    }

    let wallet = Wallet::create();
    save_wallet(agent_name, &wallet)?;
    Ok(wallet)
}

pub fn load_wallet(agent_name: &str) -> Result<Wallet> {
    let path = wallet_path(agent_name);
    let data = std::fs::read_to_string(&path)
        .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
    serde_json::from_str(&data).wrap_err_with(|| format!("Failed to parse {}", path.display()))
}

fn save_wallet(agent_name: &str, wallet: &Wallet) -> Result<()> {
    let dir = agent_dir(agent_name);
    std::fs::create_dir_all(&dir)
        .wrap_err_with(|| format!("Failed to create {}", dir.display()))?;
    let path = wallet_path(agent_name);
    let data = serde_json::to_string_pretty(wallet)?;
    std::fs::write(&path, data).wrap_err_with(|| format!("Failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, permissions)
            .wrap_err_with(|| format!("Failed to protect {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_address_starts_with_vec1_and_round_trips() {
        let wallet = Wallet::from_private_key_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();

        assert!(wallet.address.starts_with("vec1"));
        assert!(wallet.address.len() > 20);

        let loaded = Wallet::from_private_key_hex(&wallet.private_key_hex).unwrap();
        assert_eq!(loaded.address, wallet.address);
    }

    #[test]
    fn wallet_signs_and_verifies_message() {
        let wallet = Wallet::from_private_key_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();

        let message = b"vectory prediction payload";
        let signature = wallet.sign(message);

        assert!(wallet.verify(message, &signature));
        assert!(!wallet.verify(b"different message", &signature));
    }
}
