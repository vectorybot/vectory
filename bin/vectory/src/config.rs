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
    /// Optional override for the local $VCTY ledger HTTP URL.
    /// Defaults to `http://127.0.0.1:3000` when unset.
    #[serde(default)]
    pub ledger_url: Option<String>,
    /// Trusted validator ed25519 public key, base64 STANDARD-encoded (32 bytes).
    /// Populated by `vectory --agent <name> validator-info`.
    #[serde(default)]
    pub validator_pubkey: Option<String>,
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
        // Resolution order: config.yaml field → SUPABASE_URL env var →
        // compile-time `option_env!` baked into the release binary.
        // The baked default is set by CI from a repo secret at build time,
        // so end users running the released `vectory` binary need no config.
        self.game
            .supabase_url
            .clone()
            .or_else(|| std::env::var("SUPABASE_URL").ok())
            .or_else(|| option_env!("SUPABASE_URL").map(String::from))
    }

    pub fn supabase_anon_key(&self) -> Option<String> {
        // Same resolution order as `supabase_url` — see comment above.
        self.game
            .supabase_anon_key
            .clone()
            .or_else(|| std::env::var("SUPABASE_ANON_KEY").ok())
            .or_else(|| option_env!("SUPABASE_ANON_KEY").map(String::from))
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

    /// Resolve the ledger base URL, falling back to the local default when unset.
    pub fn ledger_url_or_default(&self) -> String {
        self.ledger_url
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
    }

    /// Persist `validator_pubkey` into the on-disk config.yaml, preserving every
    /// other field. We round-trip through `serde_yaml::Value` so unknown / future
    /// keys aren't dropped (the strongly-typed `PlayerConfig` would discard
    /// anything it doesn't know about).
    pub fn set_validator_pubkey_in_file(name: &str, pubkey_b64_std: &str) -> Result<()> {
        let dir = agent_dir(name);
        std::fs::create_dir_all(&dir)
            .wrap_err_with(|| format!("Failed to create {}", dir.display()))?;
        let config_path = dir.join("config.yaml");

        // It's fine for config.yaml not to exist yet — a brand-new agent who
        // only has a wallet still wants to be able to fetch the validator
        // pubkey. Create with an empty mapping in that case.
        let raw = match std::fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Err(eyre::eyre!(
                    "Failed to read {}: {}",
                    config_path.display(),
                    e
                ));
            }
        };

        let mut doc: serde_yaml::Value = if raw.trim().is_empty() {
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
        } else {
            serde_yaml::from_str(&raw)
                .wrap_err_with(|| format!("Failed to parse {}", config_path.display()))?
        };

        let mapping = doc.as_mapping_mut().ok_or_else(|| {
            eyre::eyre!(
                "Expected {} to be a YAML mapping at the top level",
                config_path.display()
            )
        })?;

        mapping.insert(
            serde_yaml::Value::String("validator_pubkey".to_string()),
            serde_yaml::Value::String(pubkey_b64_std.to_string()),
        );

        let serialized = serde_yaml::to_string(&doc)
            .wrap_err("Failed to serialize updated config.yaml")?;
        std::fs::write(&config_path, serialized)
            .wrap_err_with(|| format!("Failed to write {}", config_path.display()))?;
        Ok(())
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
