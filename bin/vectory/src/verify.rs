//! Score verification — independently verify that a round was scored correctly.
//!
//! Fetches round data from Supabase and recomputes:
//! 1. Commitment hashes (SHA-256 of prediction + salt)
//! 2. Cosine similarities from embeddings
//! 3. Softmax score distribution

use eyre::{Result, WrapErr};
use sha2::{Digest, Sha256};

use crate::config::PlayerConfig;

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

fn softmax(scores: &[f64], temperature: f64) -> Vec<f64> {
    if scores.is_empty() {
        return vec![];
    }
    let temp = if temperature == 0.0 { 1.0 } else { temperature };
    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = scores
        .iter()
        .map(|s| ((s - max_score) / temp).exp())
        .collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|e| e / sum).collect()
}

/// Parse a pgvector string like "[0.1,0.2,0.3]" into Vec<f64>.
fn parse_pgvector(s: &str) -> Option<Vec<f64>> {
    let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return None;
    }
    let vec: Vec<f64> = trimmed
        .split(',')
        .filter_map(|v| v.trim().parse::<f64>().ok())
        .collect();
    if vec.is_empty() { None } else { Some(vec) }
}

struct SupabaseClient {
    url: String,
    anon_key: String,
    client: reqwest::Client,
}

impl SupabaseClient {
    fn new(url: String, anon_key: String) -> Self {
        Self {
            url,
            anon_key,
            client: reqwest::Client::new(),
        }
    }

    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(format!("{}/rest/v1/{}", self.url, path))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .wrap_err("Supabase request failed")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            return Err(eyre::eyre!("Supabase returned {}: {}", status, body));
        }
        Ok(body)
    }
}

pub async fn verify(config: &PlayerConfig, round_id: &str) -> Result<()> {
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

    let db = SupabaseClient::new(url, anon_key);

    // 1. Fetch round data
    let rounds = db
        .get(&format!(
            "rounds?round_id=eq.{}&select=round_id,status,captured_tweet_embedding,softmax_temperature",
            round_id
        ))
        .await?;

    let round = rounds
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| eyre::eyre!("Round '{}' not found", round_id))?;

    let target_embedding = round["captured_tweet_embedding"]
        .as_str()
        .and_then(parse_pgvector);

    let temperature: f64 = round["softmax_temperature"]
        .as_f64()
        .or_else(|| {
            round["softmax_temperature"]
                .as_str()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(1.0);

    // 2. Fetch commitments
    let commitments = db
        .get(&format!(
            "commitments?round_id=eq.{}&select=player,hash",
            round_id
        ))
        .await?;

    let commitment_map: std::collections::HashMap<String, String> = commitments
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| {
            let player = c["player"].as_str()?.to_string();
            let hash = c["hash"].as_str()?.to_string();
            Some((player, hash))
        })
        .collect();

    // 3. Fetch reveals
    let reveals = db
        .get(&format!(
            "reveals?round_id=eq.{}&select=player,guess,salt,hash_valid,embedding,cosine_similarity,softmax_score",
            round_id
        ))
        .await?;

    let empty = vec![];
    let reveals_arr = reveals.as_array().unwrap_or(&empty);

    if reveals_arr.is_empty() {
        println!("No reveals found for round {}. Nothing to verify.", round_id);
        return Ok(());
    }

    println!("Verifying round {}...\n", round_id);

    // 4. Verify each reveal
    let mut all_hashes_ok = true;
    let mut all_cosine_ok = true;
    let mut all_cosine_scores = Vec::new();
    let mut all_stored_softmax = Vec::new();

    for reveal in reveals_arr {
        let player = reveal["player"].as_str().unwrap_or("unknown");
        let guess = reveal["guess"].as_str().unwrap_or("");
        let salt = reveal["salt"].as_str().unwrap_or("");

        // Commitment hash verification
        let expected_hash = {
            let input = format!("{}{}", guess, salt);
            let hash = Sha256::digest(input.as_bytes());
            hex::encode(hash)
        };
        let stored_hash = commitment_map.get(player);
        let hash_ok = stored_hash.map(|h| h == &expected_hash).unwrap_or(false);
        if !hash_ok {
            all_hashes_ok = false;
        }

        print!("  {} — hash: {}", player, if hash_ok { "OK" } else { "MISMATCH" });

        // Cosine similarity verification
        let reveal_embedding = reveal["embedding"].as_str().and_then(parse_pgvector);

        match (&target_embedding, &reveal_embedding) {
            (Some(target), Some(rev)) => {
                let recomputed = cosine_similarity(target, rev);
                let stored = reveal["cosine_similarity"]
                    .as_f64()
                    .or_else(|| {
                        reveal["cosine_similarity"]
                            .as_str()
                            .and_then(|s| s.parse().ok())
                    })
                    .unwrap_or(0.0);
                let cosine_ok = (recomputed - stored).abs() < 1e-6;
                if !cosine_ok {
                    all_cosine_ok = false;
                }
                print!(
                    " | cosine: {} ({:.6} stored vs {:.6} recomputed)",
                    if cosine_ok { "OK" } else { "MISMATCH" },
                    stored,
                    recomputed
                );
                all_cosine_scores.push(recomputed);
                all_stored_softmax.push(
                    reveal["softmax_score"]
                        .as_f64()
                        .or_else(|| {
                            reveal["softmax_score"]
                                .as_str()
                                .and_then(|s| s.parse().ok())
                        })
                        .unwrap_or(0.0),
                );
            }
            _ => {
                print!(" | cosine: skipped (embeddings not available)");
            }
        }

        println!();
    }

    // 5. Verify softmax distribution
    let softmax_ok = if !all_cosine_scores.is_empty() {
        let recomputed = softmax(&all_cosine_scores, temperature);
        let ok = recomputed
            .iter()
            .zip(all_stored_softmax.iter())
            .all(|(r, s)| (r - s).abs() < 1e-6);
        if ok {
            println!("\n  Softmax distribution: OK");
        } else {
            println!("\n  Softmax distribution: MISMATCH");
            for (i, (r, s)) in recomputed.iter().zip(all_stored_softmax.iter()).enumerate() {
                println!("    player {}: stored={:.6} recomputed={:.6}", i, s, r);
            }
        }
        ok
    } else {
        println!("\n  Softmax: skipped (no embeddings)");
        true
    };

    // 6. Overall verdict
    println!();
    if all_hashes_ok && all_cosine_ok && softmax_ok {
        println!("VERIFIED — all checks passed for round {}", round_id);
    } else {
        println!("MISMATCH DETECTED in round {}", round_id);
        if !all_hashes_ok {
            println!("  - Commitment hash verification failed");
        }
        if !all_cosine_ok {
            println!("  - Cosine similarity verification failed");
        }
        if !softmax_ok {
            println!("  - Softmax distribution verification failed");
        }
    }

    Ok(())
}
