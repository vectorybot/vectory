//! Fetch the validator's public key from the local ledger and save it into the
//! agent's config.yaml as the trusted validator_pubkey.

use eyre::{Result, WrapErr, eyre};
use serde::Deserialize;

use crate::config::PlayerConfig;

#[derive(Debug, Deserialize)]
struct PubkeyResponse {
    pubkey_base64: String,
}

pub async fn fetch_validator_info(config: &PlayerConfig, agent: &str) -> Result<()> {
    let base = config.ledger_url_or_default();
    let url = format!("{}/pubkey", base.trim_end_matches('/'));

    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .wrap_err_with(|| format!("Failed to reach ledger at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(eyre!("Ledger returned HTTP {} for {}: {}", status, url, body));
    }

    let fetched: PubkeyResponse = resp
        .json()
        .await
        .wrap_err("Failed to parse /pubkey response as JSON")?;

    // If a pubkey is already configured and differs from the fetched value, refuse to
    // overwrite — a changing validator pubkey is a security event the operator
    // must investigate (key rotation, wrong ledger URL, or attack).
    if let Some(existing) = config.validator_pubkey.as_ref() {
        if existing != &fetched.pubkey_base64 {
            return Err(eyre!(
                "Validator pubkey mismatch — refusing to overwrite.\n  configured: {}\n  fetched:    {}\nIf this is intentional, remove `validator_pubkey:` from your config.yaml and run validator-info again.",
                existing,
                fetched.pubkey_base64
            ));
        }
        println!("Validator pubkey already configured and matches the ledger.");
        println!("Pubkey: {}", fetched.pubkey_base64);
        return Ok(());
    }

    PlayerConfig::set_validator_pubkey_in_file(agent, &fetched.pubkey_base64)?;
    println!(
        "Validator pubkey saved to ~/.vectory/agents/{}/config.yaml",
        agent
    );
    println!("Pubkey: {}", fetched.pubkey_base64);
    Ok(())
}
