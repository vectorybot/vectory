//! Native Vectory wallet storage and signing.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use bech32::{Bech32, Hrp};
#[cfg(test)]
use ed25519_dalek::{Signature, Verifier};
use ed25519_dalek::{Signer, SigningKey};
use eyre::{Result, WrapErr};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::agent_dir;

const ADDRESS_HRP: &str = "vcty";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub private_key_hex: String,
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
        // Signatures use URL_SAFE_NO_PAD, not bech32. See docs/adr/0002.
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    }

    #[cfg(test)]
    pub fn verify(&self, message: &[u8], signature: &str) -> bool {
        // Signature encoding must match sign() — see docs/adr/0002.
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
        let address = encode_address(&public_key_bytes);
        Self {
            private_key_hex,
            address,
        }
    }

    fn signing_key(&self) -> SigningKey {
        let bytes = hex::decode(&self.private_key_hex).expect("wallet private key hex");
        let private_key: [u8; 32] = bytes.try_into().expect("wallet private key length");
        SigningKey::from_bytes(&private_key)
    }
}

// Address encoding is bech32 with HRP "vcty". See docs/adr/0001.
fn encode_address(public_key_bytes: &[u8; 32]) -> String {
    let hrp = Hrp::parse(ADDRESS_HRP).expect("static hrp is valid");
    bech32::encode::<Bech32>(hrp, public_key_bytes).expect("bech32 encoding of 32-byte pubkey")
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
    let stored: Wallet = serde_json::from_str(&data)
        .wrap_err_with(|| format!("Failed to parse {}", path.display()))?;

    // Re-derive address from private key. Migrates pre-bech32 wallets transparently:
    // if the on-disk address no longer matches the canonical encoding, rewrite the file.
    let bytes = hex::decode(&stored.private_key_hex)
        .wrap_err_with(|| format!("Wallet private key hex in {} is malformed", path.display()))?;
    let private_key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| eyre::eyre!("Wallet private key in {} must be 32 bytes", path.display()))?;
    let canonical = Wallet::from_signing_key(SigningKey::from_bytes(&private_key));

    if canonical.address != stored.address {
        save_wallet(agent_name, &canonical)?;
    }
    Ok(canonical)
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
    fn wallet_address_is_lowercase_bech32_with_vcty1_prefix() {
        let wallet = Wallet::from_private_key_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();

        assert!(wallet.address.starts_with("vcty1"));
        assert_eq!(wallet.address, wallet.address.to_lowercase());
        // 32-byte payload encodes to 52 base32 chars + 6-char checksum + "vcty1" = 63 chars.
        assert_eq!(wallet.address.len(), 63);
        // bech32 alphabet excludes hyphens and underscores entirely.
        assert!(!wallet.address.contains('-'));
        assert!(!wallet.address.contains('_'));

        let loaded = Wallet::from_private_key_hex(&wallet.private_key_hex).unwrap();
        assert_eq!(loaded.address, wallet.address);
    }

    #[test]
    fn wallet_address_round_trips_through_bech32_decode() {
        let wallet = Wallet::from_private_key_hex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        )
        .unwrap();

        let (hrp, data) = bech32::decode(&wallet.address).expect("decode");
        assert_eq!(hrp.as_str(), "vcty");
        assert_eq!(data.len(), 32);
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
