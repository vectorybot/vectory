use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Lightweight round spec — frozen at announcement time as a contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundSpec {
    pub schema_version: u32,
    pub round_id: String,
    pub frozen_at: String,
    pub spec_hash: String,
    pub parameters: RoundParameters,
    pub state_order: Vec<String>,
    pub scoring: ScoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundParameters {
    pub target_account: String,
    pub entry_fee: f64,
    pub commitment_window_minutes: i64,
    pub reveal_window_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoringConfig {
    pub embedding_model: String,
    pub embedding_revision: String,
    pub embedding_dimensions: u32,
    pub similarity_metric: String,
    pub softmax_temperature: f64,
    pub distribution_method: String,
}

/// The editable template that gets frozen into a RoundSpec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecTemplate {
    pub schema_version: u32,
    pub default_entry_fee: f64,
    pub default_commitment_window_minutes: i64,
    pub default_reveal_window_minutes: i64,
    pub state_order: Vec<String>,
    pub scoring: ScoringConfig,
}

impl RoundSpec {
    /// Compute the SHA-256 hash of the spec (excluding the spec_hash field itself).
    pub fn compute_hash(&self) -> String {
        let mut hashable = self.clone();
        hashable.spec_hash = String::new();
        let json = serde_json::to_string(&hashable).expect("spec serialization");
        let hash = Sha256::digest(json.as_bytes());
        format!("sha256:{}", hex::encode(hash))
    }

    /// Create a frozen spec from a template and round-specific parameters.
    pub fn freeze(
        template: &SpecTemplate,
        round_id: String,
        target_account: String,
        entry_fee: Option<f64>,
        commitment_window_minutes: Option<i64>,
        reveal_window_minutes: Option<i64>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let mut spec = RoundSpec {
            schema_version: template.schema_version,
            round_id,
            frozen_at: now,
            spec_hash: String::new(),
            parameters: RoundParameters {
                target_account,
                entry_fee: entry_fee.unwrap_or(template.default_entry_fee),
                commitment_window_minutes: commitment_window_minutes
                    .unwrap_or(template.default_commitment_window_minutes),
                reveal_window_minutes: reveal_window_minutes
                    .unwrap_or(template.default_reveal_window_minutes),
            },
            state_order: template.state_order.clone(),
            scoring: template.scoring.clone(),
        };
        spec.spec_hash = spec.compute_hash();
        spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_template() -> SpecTemplate {
        SpecTemplate {
            schema_version: 1,
            default_entry_fee: 0.0,
            default_commitment_window_minutes: 120,
            default_reveal_window_minutes: 120,
            state_order: vec![
                "commitmentsopen".into(),
                "commitmentsclosed".into(),
                "revealsopen".into(),
                "revealsclosed".into(),
                "scoring".into(),
                "complete".into(),
            ],
            scoring: ScoringConfig {
                embedding_model: "BAAI/bge-m3".into(),
                embedding_revision: "pinned".into(),
                embedding_dimensions: 1024,
                similarity_metric: "cosine".into(),
                softmax_temperature: 1.0,
                distribution_method: "softmax_proportional".into(),
            },
        }
    }

    #[test]
    fn freeze_produces_valid_spec_with_hash() {
        let template = test_template();
        let spec = RoundSpec::freeze(&template, "12".into(), "@naval".into(), None, None, None);
        assert_eq!(spec.round_id, "12");
        assert_eq!(spec.parameters.target_account, "@naval");
        assert_eq!(spec.parameters.entry_fee, 0.0);
        assert_eq!(spec.parameters.commitment_window_minutes, 120);
        assert!(spec.spec_hash.starts_with("sha256:"));
        assert_eq!(spec.spec_hash, spec.compute_hash());
    }

    #[test]
    fn freeze_overrides_defaults() {
        let template = test_template();
        let spec = RoundSpec::freeze(
            &template,
            "13".into(),
            "@elonmusk".into(),
            Some(5.0),
            Some(60),
            Some(30),
        );
        assert_eq!(spec.parameters.entry_fee, 5.0);
        assert_eq!(spec.parameters.commitment_window_minutes, 60);
        assert_eq!(spec.parameters.reveal_window_minutes, 30);
    }

    #[test]
    fn spec_hash_is_deterministic() {
        let template = test_template();
        let spec1 = RoundSpec {
            schema_version: 1,
            round_id: "12".into(),
            frozen_at: "2026-03-27T14:00:00Z".into(),
            spec_hash: String::new(),
            parameters: RoundParameters {
                target_account: "@naval".into(),
                entry_fee: 0.0,
                commitment_window_minutes: 120,
                reveal_window_minutes: 120,
            },
            state_order: template.state_order.clone(),
            scoring: template.scoring.clone(),
        };
        let mut spec2 = spec1.clone();
        assert_eq!(spec1.compute_hash(), spec2.compute_hash());
        spec2.parameters.entry_fee = 1.0;
        assert_ne!(spec1.compute_hash(), spec2.compute_hash());
    }

    #[test]
    fn spec_round_trips_through_json() {
        let template = test_template();
        let spec = RoundSpec::freeze(&template, "42".into(), "@test".into(), None, None, None);
        let json = serde_json::to_string(&spec).unwrap();
        let decoded: RoundSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, decoded);
    }

    #[test]
    fn template_deserializes_from_json_file() {
        let raw = include_str!("../../../spec/template.json");
        let template: SpecTemplate = serde_json::from_str(raw).unwrap();
        assert_eq!(template.schema_version, 1);
        assert!(!template.state_order.is_empty());
    }
}
