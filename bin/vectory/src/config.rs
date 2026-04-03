//! Player configuration — YAML loading with environment variable expansion.

use eyre::{Result, WrapErr};
use serde::Deserialize;
use std::path::PathBuf;
use twitter_api::{TwitterClient, TwitterConfig};

#[derive(Deserialize)]
pub struct PlayerConfig {
    pub name: String,
    pub handle: String,
    pub twitter: TwitterCreds,
    pub game: GameSettings,
}

#[derive(Deserialize)]
pub struct TwitterCreds {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
    #[serde(default)]
    pub twitterapi_dot_io_api_key: Option<String>,
}

#[derive(Deserialize)]
pub struct GameSettings {
    pub validator_username: String,
    pub base_wallet_address: String,
    #[serde(default)]
    pub supabase_url: Option<String>,
    #[serde(default)]
    pub supabase_anon_key: Option<String>,
}

/// Resolve the agent directory: ~/.vectory/agents/{name}/
pub fn agent_dir(name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".vectory")
        .join("agents")
        .join(name)
}

impl PlayerConfig {
    /// Load a player config from `~/.vectory/agents/{name}/config.yaml`,
    /// expanding `${VAR_NAME}` patterns from environment variables.
    pub fn load(name: &str) -> Result<Self> {
        let dir = agent_dir(name);
        let config_path = dir.join("config.yaml");
        let raw = std::fs::read_to_string(&config_path)
            .wrap_err_with(|| format!("Failed to read {}", config_path.display()))?;

        let expanded = expand_env_vars(&raw);

        serde_yaml::from_str(&expanded)
            .wrap_err_with(|| format!("Failed to parse {}", config_path.display()))
    }

    /// Build a `TwitterClient` from config credentials.
    pub fn twitter_client(&self) -> TwitterClient {
        TwitterClient::new(TwitterConfig {
            api_key: self.twitter.api_key.clone(),
            api_secret: self.twitter.api_secret.clone(),
            access_token: self.twitter.access_token.clone(),
            access_token_secret: self.twitter.access_token_secret.clone(),
        })
    }

    /// Resolve twitterapi.io key from config first, then environment.
    pub fn twitterapi_io_api_key(&self) -> Result<String> {
        self.twitter
            .twitterapi_dot_io_api_key
            .clone()
            .or_else(|| std::env::var("TWITTERAPI_DOT_IO_API_KEY").ok())
            .ok_or_else(|| {
                eyre::eyre!(
                    "twitterapi.io API key missing. Set twitter.twitterapi_dot_io_api_key in config.yaml or TWITTERAPI_DOT_IO_API_KEY env var"
                )
            })
    }
}

/// Replace `${VAR_NAME}` patterns with values from environment variables.
fn expand_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            let mut found_close = false;
            for c in chars.by_ref() {
                if c == '}' {
                    found_close = true;
                    break;
                }
                var_name.push(c);
            }
            if found_close && !var_name.is_empty() {
                match std::env::var(&var_name) {
                    Ok(val) => result.push_str(&val),
                    Err(_) => {
                        result.push_str("${");
                        result.push_str(&var_name);
                        result.push('}');
                    }
                }
            } else {
                result.push_str("${");
                result.push_str(&var_name);
            }
        } else {
            result.push(ch);
        }
    }

    result
}
