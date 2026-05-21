//! Public prediction commitments with proof-of-work.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct PredictionCommitment {
    pub protocol_version: String,
    pub chain_id: String,
    pub target_account_id: String,
    pub wallet: String,
    pub prediction_text: String,
    pub scoring_model_id: String,
    pub difficulty_bits: u32,
    pub pow_nonce: u64,
    pub canonical_payload: String,
    pub pow_digest: String,
}

impl PredictionCommitment {
    pub fn new(
        protocol_version: &str,
        chain_id: &str,
        target_account_id: &str,
        wallet: &str,
        prediction_text: &str,
        scoring_model_id: &str,
        difficulty_bits: u32,
        pow_nonce: u64,
    ) -> Self {
        let canonical_payload = canonical_payload(
            protocol_version,
            chain_id,
            target_account_id,
            wallet,
            prediction_text,
            scoring_model_id,
        );
        let pow_digest = digest_for_nonce(&canonical_payload, pow_nonce);
        Self {
            protocol_version: protocol_version.to_string(),
            chain_id: chain_id.to_string(),
            target_account_id: target_account_id.to_string(),
            wallet: wallet.to_string(),
            prediction_text: prediction_text.to_string(),
            scoring_model_id: scoring_model_id.to_string(),
            difficulty_bits,
            pow_nonce,
            canonical_payload,
            pow_digest,
        }
    }

    pub fn mine(
        protocol_version: &str,
        chain_id: &str,
        target_account_id: &str,
        wallet: &str,
        prediction_text: &str,
        scoring_model_id: &str,
        difficulty_bits: u32,
    ) -> Self {
        let mut nonce = 0;
        loop {
            let commitment = Self::new(
                protocol_version,
                chain_id,
                target_account_id,
                wallet,
                prediction_text,
                scoring_model_id,
                difficulty_bits,
                nonce,
            );
            if commitment.pow_digest_has_leading_zero_bits() {
                return commitment;
            }
            nonce += 1;
        }
    }

    pub fn pow_digest_has_leading_zero_bits(&self) -> bool {
        digest_has_leading_zero_bits(
            &digest_bytes(&self.canonical_payload, self.pow_nonce),
            self.difficulty_bits,
        )
    }
}

fn canonical_payload(
    protocol_version: &str,
    chain_id: &str,
    target_account_id: &str,
    wallet: &str,
    prediction_text: &str,
    scoring_model_id: &str,
) -> String {
    format!(
        "protocol_version:{protocol_version}\nchain_id:{chain_id}\ntarget_account_id:{target_account_id}\nwallet:{wallet}\nprediction_text:{prediction_text}\nscoring_model_id:{scoring_model_id}"
    )
}

fn digest_for_nonce(canonical_payload: &str, nonce: u64) -> String {
    URL_SAFE_NO_PAD.encode(digest_bytes(canonical_payload, nonce))
}

fn digest_bytes(canonical_payload: &str, nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(canonical_payload.as_bytes());
    hasher.update(b"\nnonce:");
    hasher.update(nonce.to_string().as_bytes());
    hasher.finalize().into()
}

fn digest_has_leading_zero_bits(bytes: &[u8], difficulty_bits: u32) -> bool {
    let full_zero_bytes = (difficulty_bits / 8) as usize;
    let remaining_bits = difficulty_bits % 8;

    for byte in bytes.iter().take(full_zero_bytes) {
        if *byte != 0 {
            return false;
        }
    }

    if remaining_bits == 0 {
        return true;
    }

    let Some(byte) = bytes.get(full_zero_bytes) else {
        return false;
    };
    let mask = 0xff << (8 - remaining_bits);
    byte & mask == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_payload_uses_only_eventual_chain_fields() {
        let commitment = PredictionCommitment::new(
            "vectory-v1",
            "vectory-local",
            "12345",
            "vec1abc",
            "the target will discuss open source models",
            "bge-m3",
            0,
            0,
        );

        assert_eq!(
            commitment.canonical_payload,
            "protocol_version:vectory-v1\nchain_id:vectory-local\ntarget_account_id:12345\nwallet:vec1abc\nprediction_text:the target will discuss open source models\nscoring_model_id:bge-m3"
        );
    }

    #[test]
    fn mined_commitment_satisfies_difficulty() {
        let commitment = PredictionCommitment::mine(
            "vectory-v1",
            "vectory-local",
            "12345",
            "vec1abc",
            "the target will discuss open source models",
            "bge-m3",
            8,
        );

        assert!(commitment.pow_digest_has_leading_zero_bits());
    }
}
