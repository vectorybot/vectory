//! Vectory core types
//!
//! Data models for rounds, commitments, reveals, and scoring.

pub mod spec;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Current phase of a round
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RoundStatus {
    /// Round announced, accepting commitments
    CommitmentsOpen,
    /// Commitment deadline passed, waiting for target tweet
    CommitmentsClosed,
    /// Target captured, accepting reveals
    RevealsOpen,
    /// Reveal deadline passed, ready for scoring
    RevealsClosed,
    /// Computing scores and rankings
    Scoring,
    /// Results announced, payouts recorded
    Complete,
    /// Round cancelled
    Cancelled,
}

/// A prediction round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub round_id: String,
    pub status: RoundStatus,
    pub created_at: DateTime<Utc>,

    /// The target tweet to predict
    pub target: Target,

    /// Entry fee per player in USDC
    pub entry_fee: f64,

    /// Tweet ID of the round announcement
    pub announcement_tweet_id: Option<String>,

    /// Embedding model used (e.g. "BAAI/bge-m3")
    pub embedding_model: String,

    /// Git commit hash of the model revision at time of embedding
    pub embedding_revision: Option<String>,

    /// Expected embedding dimensions (e.g. 1024)
    pub embedding_dimensions: u32,

    /// Temperature parameter used in softmax scoring
    pub softmax_temperature: f64,

    /// Collected commitments
    pub commitments: Vec<Commitment>,

    /// Collected reveals (after target captured)
    pub reveals: Vec<Reveal>,

    /// Final results (after scoring)
    pub results: Option<Results>,
}

/// What players are predicting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    /// Twitter account to monitor
    pub account: String,

    /// Predict first tweet after this timestamp
    pub after_timestamp: DateTime<Utc>,

    /// The captured tweet (once found)
    pub tweet: Option<CapturedTweet>,
}

/// A captured target tweet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedTweet {
    pub tweet_id: String,
    pub text: String,
    pub posted_at: DateTime<Utc>,
    pub captured_at: DateTime<Utc>,
    /// Raw BGE-M3 embedding vector (1024-dim)
    pub embedding: Vec<f64>,
}

/// A player's commitment (hash of prediction + salt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub player: String,
    pub hash: String,
    /// USDC payout address on Base
    pub address: Option<String>,
    /// Tweet ID of commitment reply
    pub tweet_id: String,
    pub submitted_at: DateTime<Utc>,
}

/// A player's revealed prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reveal {
    pub player: String,
    pub guess: String,
    pub salt: String,
    /// Whether the reveal hash matches the commitment
    pub hash_valid: bool,
    /// Raw BGE-M3 embedding vector (1024-dim)
    pub embedding: Vec<f64>,
    /// Cosine similarity to target (raw, independent score)
    pub cosine_similarity: f64,
    /// Softmax score (competitive, sums to 100%)
    pub softmax_score: f64,
    /// Tweet ID of reveal reply
    pub tweet_id: String,
    pub submitted_at: DateTime<Utc>,
}

/// Final round results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Results {
    pub rankings: Vec<RankedPlayer>,
    pub announced_at: DateTime<Utc>,
    /// Tweet ID of the results announcement
    pub announcement_tweet_id: Option<String>,
    pub payout_currency: String,
    pub payout_network: String,
}

/// A player's final ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedPlayer {
    pub rank: usize,
    pub player: String,
    pub guess: String,
    pub cosine_similarity: f64,
    pub softmax_score: f64,
    pub payout: f64,
    pub address: Option<String>,
    pub paid: bool,
    pub tx_hash: Option<String>,
}

impl Round {
    pub fn new(target_account: String, after_timestamp: DateTime<Utc>, entry_fee: f64) -> Self {
        let round_id = Uuid::new_v4().as_simple().to_string();
        Self {
            round_id,
            status: RoundStatus::CommitmentsOpen,
            created_at: Utc::now(),
            target: Target {
                account: target_account,
                after_timestamp,
                tweet: None,
            },
            entry_fee,
            announcement_tweet_id: None,
            embedding_model: "BAAI/bge-m3".to_string(),
            embedding_revision: None,
            embedding_dimensions: 1024,
            softmax_temperature: 1.0,
            commitments: Vec::new(),
            reveals: Vec::new(),
            results: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_status_serializes_to_db_values() {
        let cases = vec![
            (RoundStatus::CommitmentsOpen, "commitmentsopen"),
            (RoundStatus::CommitmentsClosed, "commitmentsclosed"),
            (RoundStatus::RevealsOpen, "revealsopen"),
            (RoundStatus::RevealsClosed, "revealsclosed"),
            (RoundStatus::Scoring, "scoring"),
            (RoundStatus::Complete, "complete"),
            (RoundStatus::Cancelled, "cancelled"),
        ];
        for (variant, expected) in cases {
            let serialized = serde_json::to_value(&variant).unwrap();
            assert_eq!(
                serialized, expected,
                "RoundStatus::{:?} serialized to {:?}, expected {:?}",
                variant, serialized, expected
            );
        }
    }

    #[test]
    fn round_status_deserializes_from_db_values() {
        let cases = vec![
            ("commitmentsopen", RoundStatus::CommitmentsOpen),
            ("commitmentsclosed", RoundStatus::CommitmentsClosed),
            ("revealsopen", RoundStatus::RevealsOpen),
            ("revealsclosed", RoundStatus::RevealsClosed),
            ("scoring", RoundStatus::Scoring),
            ("complete", RoundStatus::Complete),
            ("cancelled", RoundStatus::Cancelled),
        ];
        for (input, expected) in cases {
            let json = format!("\"{}\"", input);
            let deserialized: RoundStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, expected, "Failed to deserialize {:?}", input);
        }
    }
}
