//! Round checking — fetch active rounds from Twitter or Supabase.

use eyre::{Result, WrapErr};
use twitter_api::TwitterApi;

use crate::config::PlayerConfig;

/// Fetch recent tweets from the validator account and display round info.
pub async fn check_rounds(config: &PlayerConfig) -> Result<()> {
    let client = config.twitter_client();
    let username = &config.game.validator_username;

    println!("Checking @{} for active rounds...\n", username);

    let tweets = client
        .get_user_tweets(username, 10, false)
        .await
        .wrap_err_with(|| format!("Failed to fetch tweets from @{}", username))?;

    if tweets.is_empty() {
        println!("No recent tweets from @{}", username);
        return Ok(());
    }

    let mut found_rounds = false;
    for tweet in &tweets {
        let text_lower = tweet.text.to_lowercase();
        if text_lower.contains("#vectory") {
            found_rounds = true;
            let status = if text_lower.contains("#commitmentsopen") {
                "COMMITMENTS OPEN"
            } else if text_lower.contains("#commitmentsclosed") {
                "COMMITMENTS CLOSED"
            } else if text_lower.contains("#revealsopen") {
                "REVEALS OPEN"
            } else if text_lower.contains("#revealsclosed") {
                "REVEALS CLOSED"
            } else if text_lower.contains("#complete") {
                "COMPLETE"
            } else {
                "UNKNOWN"
            };

            let time = tweet
                .created_at
                .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_default();

            println!("[{}] {} (tweet: {})", status, time, tweet.id);
            // Print first line of tweet text for context
            if let Some(first_line) = tweet.text.lines().next() {
                println!("  {}", first_line);
            }
            println!();
        }
    }

    if !found_rounds {
        println!("No #vectory rounds found in recent tweets from @{}", username);
    }

    Ok(())
}

/// Fetch round results from Supabase (read-only, anon key).
pub async fn check_results(config: &PlayerConfig, round_id: &str) -> Result<()> {
    let url = config
        .game
        .supabase_url
        .clone()
        .or_else(|| std::env::var("SUPABASE_URL").ok())
        .ok_or_else(|| eyre::eyre!("No supabase_url in config or SUPABASE_URL env var"))?;

    let anon_key = config
        .game
        .supabase_anon_key
        .clone()
        .or_else(|| std::env::var("SUPABASE_ANON_KEY").ok())
        .ok_or_else(|| eyre::eyre!("No supabase_anon_key in config or SUPABASE_ANON_KEY env var"))?;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/rest/v1/rounds?round_id=eq.{}&select=*",
            url, round_id
        ))
        .header("apikey", &anon_key)
        .header("Authorization", format!("Bearer {}", anon_key))
        .send()
        .await
        .wrap_err("Failed to query Supabase")?;

    let body: serde_json::Value = resp.json().await?;

    if let Some(rounds) = body.as_array() {
        if rounds.is_empty() {
            println!("No round found with id {}", round_id);
        } else {
            println!("{}", serde_json::to_string_pretty(&rounds[0])?);
        }
    }

    Ok(())
}
