//! Player configuration — YAML loading with environment variable expansion.

use eyre::{Result, WrapErr};
use serde::Deserialize;
use std::path::PathBuf;
use twitter_api::{TwitterClient, TwitterConfig};

#[derive(Deserialize, Default)]
#[allow(dead_code)]
pub struct PlayerConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub handle: String,
    #[serde(default)]
    pub twitter: Option<TwitterCreds>,
    #[serde(default)]
    pub game: GameSettings,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct TwitterCreds {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
    #[serde(default)]
    pub twitterapi_dot_io_api_key: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct GameSettings {
    #[serde(default)]
    pub validator_username: String,
    #[serde(default)]
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

    pub fn load_if_exists(name: &str) -> Result<Option<Self>> {
        let dir = agent_dir(name);
        let config_path = dir.join("config.yaml");
        if !config_path.exists() {
            return Ok(None);
        }
        Self::load(name).map(Some)
    }

    /// Build a `TwitterClient` from config credentials.
    pub fn twitter_client(&self) -> Result<TwitterClient> {
        let twitter = self.twitter.as_ref().ok_or_else(|| {
            eyre::eyre!(
                "Twitter credentials missing. Add a twitter block to config.yaml to post through the API, or run without --post and paste manually."
            )
        })?;
        Ok(TwitterClient::new(TwitterConfig {
            api_key: twitter.api_key.clone(),
            api_secret: twitter.api_secret.clone(),
            access_token: twitter.access_token.clone(),
            access_token_secret: twitter.access_token_secret.clone(),
        }))
    }

    pub fn supabase_url(&self) -> Option<String> {
        self.game
            .supabase_url
            .clone()
            .or_else(|| std::env::var("SUPABASE_URL").ok())
    }

    pub fn supabase_anon_key(&self) -> Option<String> {
        self.game
            .supabase_anon_key
            .clone()
            .or_else(|| std::env::var("SUPABASE_ANON_KEY").ok())
    }

    pub fn validator_username(&self) -> Result<&str> {
        if self.game.validator_username.is_empty() {
            return Err(eyre::eyre!(
                "validator_username missing in config.yaml; required for rounds"
            ));
        }
        Ok(&self.game.validator_username)
    }

    pub fn base_wallet_address(&self) -> Result<&str> {
        if self.game.base_wallet_address.is_empty() {
            return Err(eyre::eyre!(
                "base_wallet_address missing in config.yaml; required for legacy commit"
            ));
        }
        Ok(&self.game.base_wallet_address)
    }

    /// Resolve twitterapi.io key from config first, then environment.
    #[allow(dead_code)]
    pub fn twitterapi_io_api_key(&self) -> Result<String> {
        self.twitter
            .as_ref()
            .and_then(|twitter| twitter.twitterapi_dot_io_api_key.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manual_config_without_twitter_credentials() {
        let raw = r#"
game:
  supabase_url: "https://example.supabase.co"
  supabase_anon_key: "anon"
"#;

        let config: PlayerConfig = serde_yaml::from_str(raw).unwrap();

        assert!(config.twitter.is_none());
        assert_eq!(
            config.supabase_url().unwrap(),
            "https://example.supabase.co"
        );
        assert_eq!(config.supabase_anon_key().unwrap(), "anon");
    }

    #[test]
    fn twitter_client_errors_without_twitter_credentials() {
        let config: PlayerConfig = serde_yaml::from_str("game: {}\n").unwrap();

        assert!(config.twitter_client().is_err());
    }
}
