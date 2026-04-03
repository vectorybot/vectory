//! Commitment generation and posting.

use eyre::{Result, WrapErr};
use sha2::{Digest, Sha256};
use twitter_api::TwitterApi;

use crate::config::PlayerConfig;
use crate::predictions::{self, PredictionRecord};

/// Compute SHA-256 commitment hash: hex(SHA256(prediction || salt))
pub fn compute_hash(prediction: &str, salt: &str) -> String {
    let input = format!("{}{}", prediction, salt);
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

/// Generate a random 16-character alphanumeric salt.
pub fn generate_salt() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

/// Generate commitment, save locally, and post as a quote tweet.
pub async fn commit(
    config: &PlayerConfig,
    agent_name: &str,
    round_id: &str,
    prediction: &str,
    salt: Option<&str>,
    tweet_id: &str,
) -> Result<()> {
    // Check for existing prediction
    if let Some(existing) = predictions::load(agent_name, round_id)? {
        if existing.commitment.is_some() {
            println!("Already committed to round {}:", round_id);
            println!("  hash: {}", existing.hash);
            println!("  tweet: {}", existing.commitment.unwrap().tweet_id);
            return Ok(());
        }
        println!("Draft exists for round {} but not yet posted. Re-posting.", round_id);
    }

    let salt = salt.map(String::from).unwrap_or_else(generate_salt);
    let hash = compute_hash(prediction, &salt);

    // Save prediction locally first
    let record = PredictionRecord {
        round_id: round_id.to_string(),
        prediction: prediction.to_string(),
        salt: salt.clone(),
        hash: hash.clone(),
        saved_at: chrono::Utc::now().to_rfc3339(),
        commitment: None,
        reveal: None,
    };
    let path = predictions::save(agent_name, &record)?;
    println!("Prediction saved to {}", path.display());

    // Build commitment tweet text
    let text = format!(
        "hash:{}\naddress:{}",
        hash, config.game.base_wallet_address
    );

    println!("Posting commitment for round {}...", round_id);
    let client = config.twitter_client();
    // Try quote tweet first, fall back to reply if quote is blocked
    let result = match client.quote_tweet(&text, tweet_id).await {
        Ok(r) => r,
        Err(twitter_api::TwitterError::ApiError { status: 403, .. }) => {
            println!("Quote tweet blocked, trying reply...");
            client
                .reply_to_tweet(&text, tweet_id)
                .await
                .wrap_err("Failed to post commitment (both quote and reply failed)")?
        }
        Err(e) => return Err(e).wrap_err("Failed to post commitment tweet"),
    };

    // Mark as posted
    predictions::mark_commitment_posted(agent_name, round_id, &result.tweet.id)?;

    println!("Commitment posted!");
    println!("  tweet: {}", result.tweet.url);
    println!("  hash: {}", hash);

    Ok(())
}
