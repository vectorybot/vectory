//! Query the local $VCTY ledger for the player's wallet balance.

use eyre::{Result, WrapErr};
use serde::Deserialize;

use crate::config::PlayerConfig;
use crate::wallet;

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    address: String,
    balance: u64,
    nonce: u32,
    #[serde(default)]
    exists: Option<bool>,
}

/// Fetch and print the balance for the agent's wallet from the ledger service.
pub async fn check_balance(config: &PlayerConfig, agent: &str) -> Result<()> {
    let wallet = wallet::load_wallet(agent).wrap_err_with(|| {
        format!(
            "Failed to load wallet for agent `{}`. Run `vectory --agent {} wallet create` first.",
            agent, agent
        )
    })?;

    let base = config.ledger_url_or_default();
    let url = format!("{}/balance/{}", base.trim_end_matches('/'), wallet.address);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .wrap_err_with(|| format!("Failed to reach ledger at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(eyre::eyre!(
            "Ledger returned HTTP {} for {}: {}",
            status,
            url,
            body
        ));
    }

    let parsed: BalanceResponse = resp
        .json()
        .await
        .wrap_err("Failed to parse ledger /balance response as JSON")?;

    println!("Address: {}", parsed.address);
    // The ledger reports balance=0,nonce=0,exists=false for unknown addresses.
    // Treat `exists: Some(false)` as "account not yet seen by ledger" so the
    // player isn't confused by a brand-new wallet.
    if matches!(parsed.exists, Some(false)) {
        println!("Balance: 0 $VCTY (account not yet seen by ledger)");
        println!("Nonce:   0");
    } else {
        println!("Balance: {} $VCTY", parsed.balance);
        println!("Nonce:   {}", parsed.nonce);
    }

    Ok(())
}
