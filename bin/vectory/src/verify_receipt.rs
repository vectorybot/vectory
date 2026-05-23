//! Verify a signed $VEC ledger receipt against the trusted validator pubkey.

use base64::{Engine as _, engine::general_purpose};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use eyre::{Result, WrapErr, eyre};
use serde::Deserialize;
use std::path::Path;

use crate::config::PlayerConfig;
use crate::wallet;

#[derive(Debug, Deserialize)]
struct Receipt {
    sender: String,
    receiver: String,
    amount: u64,
    nonce: u32,
    signature: String,     // base64 STANDARD, 64 bytes
    sender_pubkey: String, // base64 STANDARD, 32 bytes
}

pub fn verify_receipt(config: &PlayerConfig, agent: &str, tx_file: &Path) -> Result<()> {
    let trusted_pubkey = config.validator_pubkey.as_ref().ok_or_else(|| {
        eyre!(
            "No trusted validator_pubkey configured. Run `vectory --agent {} validator-info` first to fetch and save the validator's public key.",
            agent
        )
    })?;

    let wallet = wallet::load_wallet(agent).wrap_err_with(|| {
        format!(
            "Failed to load wallet for agent `{}`. Run `vectory --agent {} wallet create` first.",
            agent, agent
        )
    })?;

    let raw = std::fs::read_to_string(tx_file)
        .wrap_err_with(|| format!("Failed to read receipt file {}", tx_file.display()))?;
    let receipt: Receipt = serde_json::from_str(&raw)
        .wrap_err_with(|| format!("Receipt at {} is not valid JSON", tx_file.display()))?;

    // Check 1: receipt was signed by the trusted validator pubkey.
    if &receipt.sender_pubkey != trusted_pubkey {
        return Err(eyre!(
            "Receipt sender_pubkey does not match trusted validator_pubkey.\n  receipt:  {}\n  trusted:  {}",
            receipt.sender_pubkey,
            trusted_pubkey
        ));
    }

    // Check 2: receiver is the player's own wallet.
    if receipt.receiver != wallet.address {
        return Err(eyre!(
            "Receipt receiver is not your wallet address.\n  receiver: {}\n  yours:    {}",
            receipt.receiver,
            wallet.address
        ));
    }

    // Check 3: ed25519 signature verifies over the canonical bytes.
    let pubkey_bytes = general_purpose::STANDARD
        .decode(&receipt.sender_pubkey)
        .wrap_err("sender_pubkey is not valid base64 STANDARD")?;
    if pubkey_bytes.len() != 32 {
        return Err(eyre!(
            "sender_pubkey must decode to 32 bytes, got {}",
            pubkey_bytes.len()
        ));
    }
    let mut pk_arr = [0u8; 32];
    pk_arr.copy_from_slice(&pubkey_bytes);
    let verifying_key =
        VerifyingKey::from_bytes(&pk_arr).wrap_err("sender_pubkey is not a valid ed25519 key")?;

    let sig_bytes = general_purpose::STANDARD
        .decode(&receipt.signature)
        .wrap_err("signature is not valid base64 STANDARD")?;
    if sig_bytes.len() != 64 {
        return Err(eyre!(
            "signature must decode to 64 bytes, got {}",
            sig_bytes.len()
        ));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    let canonical = format!(
        "{}|{}|{}|{}",
        receipt.sender, receipt.receiver, receipt.amount, receipt.nonce
    );
    verifying_key
        .verify(canonical.as_bytes(), &signature)
        .wrap_err("ed25519 signature does not verify over the canonical bytes")?;

    println!("✓ Receipt verified");
    println!("  From:    {}", receipt.sender);
    println!(
        "  To:      {} (matches your wallet)",
        receipt.receiver
    );
    println!("  Amount:  {} $VEC", receipt.amount);
    println!("  Nonce:   {}", receipt.nonce);
    println!("  Signer:  validator (matches trusted pubkey)");
    Ok(())
}
