//! Local prediction storage.
//!
//! Predictions are saved as JSON files at ~/.vectory/agents/<name>/predictions/<round_id>.json
//! These are local drafts — Twitter replies are the actual source of truth.

use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::agent_dir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostedTweet {
    pub tweet_id: String,
    pub posted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRecord {
    pub round_id: String,
    pub prediction: String,
    pub salt: String,
    pub hash: String,
    pub saved_at: String,
    #[serde(default)]
    pub commitment: Option<PostedTweet>,
    #[serde(default)]
    pub reveal: Option<PostedTweet>,
}

fn predictions_dir(agent_name: &str) -> PathBuf {
    agent_dir(agent_name).join("predictions")
}

fn prediction_path(agent_name: &str, round_id: &str) -> PathBuf {
    predictions_dir(agent_name).join(format!("{round_id}.json"))
}

pub fn load(agent_name: &str, round_id: &str) -> Result<Option<PredictionRecord>> {
    let path = prediction_path(agent_name, round_id);
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
    let record: PredictionRecord = serde_json::from_str(&data)
        .wrap_err_with(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(record))
}

pub fn save(agent_name: &str, record: &PredictionRecord) -> Result<PathBuf> {
    let dir = predictions_dir(agent_name);
    std::fs::create_dir_all(&dir)
        .wrap_err_with(|| format!("Failed to create {}", dir.display()))?;
    let path = prediction_path(agent_name, &record.round_id);
    let data = serde_json::to_string_pretty(record)?;
    std::fs::write(&path, data)
        .wrap_err_with(|| format!("Failed to write {}", path.display()))?;
    Ok(path)
}

pub fn mark_commitment_posted(
    agent_name: &str,
    round_id: &str,
    tweet_id: &str,
) -> Result<PredictionRecord> {
    let mut record = load(agent_name, round_id)?
        .ok_or_else(|| eyre::eyre!("No prediction found for round {round_id}"))?;
    record.commitment = Some(PostedTweet {
        tweet_id: tweet_id.to_string(),
        posted_at: chrono::Utc::now().to_rfc3339(),
    });
    save(agent_name, &record)?;
    Ok(record)
}

pub fn mark_reveal_posted(
    agent_name: &str,
    round_id: &str,
    tweet_id: &str,
) -> Result<PredictionRecord> {
    let mut record = load(agent_name, round_id)?
        .ok_or_else(|| eyre::eyre!("No prediction found for round {round_id}"))?;
    record.reveal = Some(PostedTweet {
        tweet_id: tweet_id.to_string(),
        posted_at: chrono::Utc::now().to_rfc3339(),
    });
    save(agent_name, &record)?;
    Ok(record)
}
