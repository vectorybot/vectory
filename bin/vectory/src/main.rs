mod balance;
mod commit;
mod config;
mod predict;
mod prediction_commitment;
mod predictions;
mod reveal;
mod rounds;
mod validator_info;
mod verify;
mod verify_receipt;
mod wallet;

use clap::{Parser, Subcommand};
use eyre::Result;
use std::path::PathBuf;
use twitter_api::TwitterApi;

#[derive(Parser)]
#[command(name = "vectory", about = "Vectory player CLI")]
struct Cli {
    /// Agent name (loads config from ~/.vectory/agents/<name>/config.yaml)
    #[arg(long, short)]
    agent: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check for active rounds from the validator
    Rounds,

    /// Manage native Vectory wallet
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
    },

    /// Make a public prediction commitment with proof-of-work
    Predict {
        /// Target Twitter account id or handle
        #[arg(long)]
        target_account_id: String,
        /// Public prediction text
        #[arg(long)]
        prediction: String,
        /// Scoring model id
        #[arg(long, default_value = "bge-m3")]
        scoring_model_id: String,
        /// PoW leading zero bits
        #[arg(long, default_value_t = 16)]
        difficulty_bits: u32,
        /// Vectory chain id
        #[arg(long, default_value = "vectory-local")]
        chain_id: String,
        /// Prediction protocol version
        #[arg(long, default_value = "vectory-v1")]
        protocol_version: String,
        /// Post through Twitter API credentials instead of printing tweet text
        #[arg(long)]
        post: bool,
        /// Tweet ID to quote when posting
        #[arg(long)]
        tweet_id: Option<String>,
    },

    /// Commit a prediction to a round
    Commit {
        /// Round ID
        #[arg(long)]
        round_id: String,
        /// Your prediction text
        #[arg(long)]
        prediction: String,
        /// Custom salt (auto-generated if omitted)
        #[arg(long)]
        salt: Option<String>,
        /// Tweet ID to quote (announcement tweet)
        #[arg(long)]
        tweet_id: String,
    },

    /// Reveal your prediction for a round
    Reveal {
        /// Round ID
        #[arg(long)]
        round_id: String,
        /// Tweet ID to quote (reveals-open tweet)
        #[arg(long)]
        tweet_id: String,
    },

    /// Show round results from Supabase
    Results {
        /// Round ID
        round_id: String,
    },

    /// Show a saved prediction
    Show {
        /// Round ID
        round_id: String,
    },

    /// Verify that a round was scored correctly
    Verify {
        /// Round ID
        round_id: String,
    },

    /// Compute a commitment hash without posting
    Hash {
        /// Prediction text
        prediction: String,
        /// Salt (auto-generated if omitted)
        #[arg(long)]
        salt: Option<String>,
    },

    /// Post a standalone tweet
    Tweet {
        /// Tweet text
        text: String,
    },

    /// Post a quote tweet
    Quote {
        /// Tweet ID to quote
        tweet_id: String,
        /// Quote text
        text: String,
    },

    /// Reply to a tweet
    Reply {
        /// Tweet ID to reply to
        tweet_id: String,
        /// Reply text
        text: String,
    },

    /// Show this agent's $VEC balance from the local ledger
    Balance,

    /// Verify a signed $VEC ledger receipt against the trusted validator pubkey
    VerifyReceipt {
        /// Path to the JSON receipt (signed transaction file)
        #[arg(long)]
        tx_file: PathBuf,
    },

    /// Fetch and save the validator's public key from the local ledger
    ValidatorInfo,
}

#[derive(Subcommand)]
enum WalletCommand {
    /// Create a native Vectory wallet for this agent
    Create,
    /// Show this agent's native Vectory wallet address
    Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Rounds => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            rounds::check_rounds(&config).await?;
        }

        Command::Wallet { command } => match command {
            WalletCommand::Create => {
                let wallet = wallet::create_wallet(&cli.agent)?;
                println!("Created native Vectory wallet");
                println!("Address: {}", wallet.address);
            }
            WalletCommand::Address => {
                let wallet = wallet::load_wallet(&cli.agent)?;
                println!("{}", wallet.address);
            }
        },

        Command::Predict {
            target_account_id,
            prediction,
            scoring_model_id,
            difficulty_bits,
            chain_id,
            protocol_version,
            post,
            tweet_id,
        } => {
            let config = config::PlayerConfig::load_if_exists(&cli.agent)?;
            predict::predict(
                config.as_ref(),
                &cli.agent,
                predict::PredictOptions {
                    target_account_id,
                    prediction,
                    scoring_model_id,
                    difficulty_bits,
                    chain_id,
                    protocol_version,
                    post,
                    tweet_id,
                },
            )
            .await?;
        }

        Command::Commit {
            round_id,
            prediction,
            salt,
            tweet_id,
        } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            commit::commit(
                &config,
                &cli.agent,
                &round_id,
                &prediction,
                salt.as_deref(),
                &tweet_id,
            )
            .await?;
        }

        Command::Reveal { round_id, tweet_id } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            reveal::reveal(&config, &cli.agent, &round_id, &tweet_id).await?;
        }

        Command::Results { round_id } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            rounds::check_results(&config, &round_id).await?;
        }

        Command::Show { round_id } => match predictions::load(&cli.agent, &round_id)? {
            Some(record) => {
                println!("{}", serde_json::to_string_pretty(&record)?);
            }
            None => {
                println!("No saved prediction for round {}", round_id);
            }
        },

        Command::Verify { round_id } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            verify::verify(&config, &round_id).await?;
        }

        Command::Hash { prediction, salt } => {
            let salt = salt.unwrap_or_else(commit::generate_salt);
            let hash = commit::compute_hash(&prediction, &salt);
            println!("prediction: {}", prediction);
            println!("salt:       {}", salt);
            println!("hash:       {}", hash);
        }

        Command::Tweet { text } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            let client = config.twitter_client()?;
            let result = client.post_tweet(&text).await?;
            println!("Posted: {}", result.tweet.url);
        }

        Command::Quote { tweet_id, text } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            let client = config.twitter_client()?;
            let result = client.quote_tweet(&text, &tweet_id).await?;
            println!("Posted: {}", result.tweet.url);
        }

        Command::Reply { tweet_id, text } => {
            let config = config::PlayerConfig::load(&cli.agent)?;
            let client = config.twitter_client()?;
            let result = client.reply_to_tweet(&text, &tweet_id).await?;
            println!("Posted: {}", result.tweet.url);
        }

        Command::Balance => {
            let config = config::PlayerConfig::load_if_exists(&cli.agent)?
                .unwrap_or_default();
            balance::check_balance(&config, &cli.agent).await?;
        }

        Command::VerifyReceipt { tx_file } => {
            let config = config::PlayerConfig::load_if_exists(&cli.agent)?
                .unwrap_or_default();
            verify_receipt::verify_receipt(&config, &cli.agent, &tx_file)?;
        }

        Command::ValidatorInfo => {
            let config = config::PlayerConfig::load_if_exists(&cli.agent)?
                .unwrap_or_default();
            validator_info::fetch_validator_info(&config, &cli.agent).await?;
        }
    }

    Ok(())
}
