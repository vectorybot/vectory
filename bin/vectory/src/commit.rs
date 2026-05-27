//! Public prediction commit command.

use chrono::Utc;
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use twitter_api::{TwitterApi, TwitterClient};

use crate::config::{PlayerConfig, agent_dir};
use crate::prediction_commitment::PredictionCommitment;
use crate::wallet;

#[derive(Debug, Serialize, Deserialize)]
struct PredictionReceipt {
    round_id: String,
    protocol_version: String,
    chain_id: String,
    target_account_id: String,
    wallet: String,
    prediction_text: String,
    scoring_model_id: String,
    difficulty_bits: u32,
    pow_nonce: u64,
    pow_digest: String,
    signature: String,
    tweet_text: String,
    tweet_id: Option<String>,
    saved_at: String,
}

pub struct CommitOptions {
    pub target_account_id: String,
    pub prediction: String,
    pub scoring_model_id: String,
    pub difficulty_bits: u32,
    pub chain_id: String,
    pub protocol_version: String,
    pub post: bool,
    pub tweet_id: Option<String>,
}

pub async fn commit(
    config: Option<&PlayerConfig>,
    agent_name: &str,
    options: CommitOptions,
) -> Result<()> {
    let wallet = wallet::load_wallet(agent_name)?;
    let round_id = fetch_active_round_id(config).await?;

    let commitment = PredictionCommitment::mine(
        &options.protocol_version,
        &options.chain_id,
        &options.target_account_id,
        &wallet.address,
        &options.prediction,
        &options.scoring_model_id,
        options.difficulty_bits,
    );
    let signature = wallet.sign(commitment.canonical_payload.as_bytes());
    let tweet_text = tweet_text(&round_id, &commitment, &signature);

    if tweet_text.chars().count() > 280 {
        return Err(eyre::eyre!(
            "Prediction tweet is {} characters; shorten the prediction to fit 280 characters",
            tweet_text.chars().count()
        ));
    }

    let mut receipt = PredictionReceipt {
        round_id: round_id.clone(),
        protocol_version: commitment.protocol_version.clone(),
        chain_id: commitment.chain_id.clone(),
        target_account_id: commitment.target_account_id.clone(),
        wallet: commitment.wallet.clone(),
        prediction_text: commitment.prediction_text.clone(),
        scoring_model_id: commitment.scoring_model_id.clone(),
        difficulty_bits: commitment.difficulty_bits,
        pow_nonce: commitment.pow_nonce,
        pow_digest: commitment.pow_digest.clone(),
        signature,
        tweet_text,
        tweet_id: None,
        saved_at: Utc::now().to_rfc3339(),
    };

    if options.post {
        let client = twitter_client_for_post(config)?;
        let posted = if let Some(tweet_id) = options.tweet_id {
            client.quote_tweet(&receipt.tweet_text, &tweet_id).await?
        } else {
            client.post_tweet(&receipt.tweet_text).await?
        };
        receipt.tweet_id = Some(posted.tweet.id);
        println!("Posted: {}", posted.tweet.url);
    } else {
        println!("{}", receipt.tweet_text);
    }

    let path = save_receipt(agent_name, &receipt)?;
    println!("Saved prediction receipt to {}", path.display());
    println!("Wallet: {}", receipt.wallet);
    println!("PoW nonce: {}", receipt.pow_nonce);
    println!("PoW digest: {}", receipt.pow_digest);

    Ok(())
}

fn twitter_client_for_post(config: Option<&PlayerConfig>) -> Result<TwitterClient> {
    if let Some(config) = config {
        if config.twitter.is_some() {
            return config.twitter_client();
        }
    }

    TwitterClient::from_env().wrap_err(
        "Twitter credentials missing. Add a twitter block to config.yaml, set TWITTER_API_KEY/TWITTER_API_SECRET/TWITTER_ACCESS_TOKEN/TWITTER_ACCESS_TOKEN_SECRET, or run without --post and paste manually.",
    )
}

async fn fetch_active_round_id(config: Option<&PlayerConfig>) -> Result<String> {
    let url = config
        .and_then(|config| config.supabase_url())
        .or_else(|| std::env::var("SUPABASE_URL").ok())
        .ok_or_else(|| eyre::eyre!("No supabase_url in config or SUPABASE_URL env var"))?;

    let anon_key = config
        .and_then(|config| config.supabase_anon_key())
        .or_else(|| std::env::var("SUPABASE_ANON_KEY").ok())
        .ok_or_else(|| {
            eyre::eyre!("No supabase_anon_key in config or SUPABASE_ANON_KEY env var")
        })?;

    let resp = reqwest::Client::new()
        .get(format!(
            "{}/rest/v1/rounds?select=round_id,status,created_at&order=created_at.desc&limit=20",
            url
        ))
        .header("apikey", &anon_key)
        .header("Authorization", format!("Bearer {}", anon_key))
        .send()
        .await
        .wrap_err("Failed to query Supabase for active round")?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        return Err(eyre::eyre!("Supabase returned {}: {}", status, body));
    }

    active_round_id_from_rows(&body).ok_or_else(|| eyre::eyre!("No active round found in database"))
}

fn active_round_id_from_rows(rows: &serde_json::Value) -> Option<String> {
    let rows = rows.as_array()?;
    for row in rows {
        let status = row["status"].as_str().unwrap_or("").to_lowercase();
        if is_active_status(&status) {
            if let Some(round_id) = row["round_id"].as_str() {
                return Some(round_id.to_string());
            }
            if let Some(round_id) = row["round_id"].as_i64() {
                return Some(round_id.to_string());
            }
        }
    }
    None
}

fn is_active_status(status: &str) -> bool {
    matches!(
        status,
        "active"
            | "open"
            | "predictionopen"
            | "predictionsopen"
            | "predictions_open"
            | "commitmentsopen"
    )
}

fn tweet_text(round_id: &str, commitment: &PredictionCommitment, signature: &str) -> String {
    format!(
        "r:{}\nt:{}\np:{}\nw:{}\nm:{}\nn:{}\ns:{}",
        round_id,
        commitment.target_account_id,
        commitment.prediction_text,
        commitment.wallet,
        commitment.scoring_model_id,
        commitment.pow_nonce,
        signature
    )
}

fn save_receipt(agent_name: &str, receipt: &PredictionReceipt) -> Result<PathBuf> {
    let dir = agent_dir(agent_name).join("prediction-receipts");
    std::fs::create_dir_all(&dir)
        .wrap_err_with(|| format!("Failed to create {}", dir.display()))?;
    let path = dir.join(format!(
        "{}-{}.json",
        receipt.round_id,
        Utc::now().timestamp()
    ));
    let data = serde_json::to_string_pretty(receipt)?;
    std::fs::write(&path, data).wrap_err_with(|| format!("Failed to write {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tweet_text_uses_database_round_id() {
        let commitment = PredictionCommitment::new(
            "vectory-v1",
            "vectory-local",
            "12345",
            "vcty1abc",
            "prediction",
            "bge-m3",
            0,
            0,
        );

        let text = tweet_text("round-74", &commitment, "sig");

        assert!(text.starts_with("r:round-74\n"));
    }

    #[test]
    fn active_round_id_uses_first_open_database_round() {
        let rows = serde_json::json!([
            {"round_id": "old", "status": "complete"},
            {"round_id": "round-74", "status": "active"}
        ]);

        assert_eq!(active_round_id_from_rows(&rows).unwrap(), "round-74");
    }

    #[test]
    fn active_round_id_accepts_legacy_commitmentsopen_status() {
        let rows = serde_json::json!([
            {"round_id": 46, "status": "commitmentsopen"}
        ]);

        assert_eq!(active_round_id_from_rows(&rows).unwrap(), "46");
    }
}
