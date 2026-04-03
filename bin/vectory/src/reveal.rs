//! Reveal posting — load saved prediction and post it publicly.

use eyre::{Result, WrapErr};
use twitter_api::TwitterApi;

use crate::config::PlayerConfig;
use crate::predictions;

/// Load saved prediction and post the reveal as a quote tweet.
pub async fn reveal(
    config: &PlayerConfig,
    agent_name: &str,
    round_id: &str,
    tweet_id: &str,
) -> Result<()> {
    let record = predictions::load(agent_name, round_id)?
        .ok_or_else(|| eyre::eyre!("No saved prediction for round {round_id}. Did you commit first?"))?;

    if record.reveal.is_some() {
        println!("Already revealed for round {}:", round_id);
        println!("  prediction: {}", record.prediction);
        println!("  tweet: {}", record.reveal.unwrap().tweet_id);
        return Ok(());
    }

    if record.commitment.is_none() {
        println!("Warning: no commitment tweet recorded for round {}. Revealing anyway.", round_id);
    }

    // Build reveal tweet text
    let text = format!("{}\nsalt:{}", record.prediction, record.salt);

    println!("Posting reveal for round {}...", round_id);
    let client = config.twitter_client();
    // Try quote tweet first, fall back to reply if quote is blocked
    let result = match client.quote_tweet(&text, tweet_id).await {
        Ok(r) => r,
        Err(twitter_api::TwitterError::ApiError { status: 403, .. }) => {
            println!("Quote tweet blocked, trying reply...");
            client
                .reply_to_tweet(&text, tweet_id)
                .await
                .wrap_err("Failed to post reveal (both quote and reply failed)")?
        }
        Err(e) => return Err(e).wrap_err("Failed to post reveal tweet"),
    };

    predictions::mark_reveal_posted(agent_name, round_id, &result.tweet.id)?;

    println!("Reveal posted!");
    println!("  tweet: {}", result.tweet.url);
    println!("  prediction: {}", record.prediction);

    Ok(())
}
