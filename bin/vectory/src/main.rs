mod commit;
mod config;
mod predictions;
mod reveal;
mod rounds;
mod verify;

use clap::{Parser, Subcommand};
use eyre::Result;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::PlayerConfig::load(&cli.agent)?;

    match cli.command {
        Command::Rounds => {
            rounds::check_rounds(&config).await?;
        }

        Command::Commit {
            round_id,
            prediction,
            salt,
            tweet_id,
        } => {
            commit::commit(&config, &cli.agent, &round_id, &prediction, salt.as_deref(), &tweet_id).await?;
        }

        Command::Reveal {
            round_id,
            tweet_id,
        } => {
            reveal::reveal(&config, &cli.agent, &round_id, &tweet_id).await?;
        }

        Command::Results { round_id } => {
            rounds::check_results(&config, &round_id).await?;
        }

        Command::Show { round_id } => {
            match predictions::load(&cli.agent, &round_id)? {
                Some(record) => {
                    println!("{}", serde_json::to_string_pretty(&record)?);
                }
                None => {
                    println!("No saved prediction for round {}", round_id);
                }
            }
        }

        Command::Verify { round_id } => {
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
            let client = config.twitter_client();
            let result = client.post_tweet(&text).await?;
            println!("Posted: {}", result.tweet.url);
        }

        Command::Quote { tweet_id, text } => {
            let client = config.twitter_client();
            let result = client.quote_tweet(&text, &tweet_id).await?;
            println!("Posted: {}", result.tweet.url);
        }

        Command::Reply { tweet_id, text } => {
            let client = config.twitter_client();
            let result = client.reply_to_tweet(&text, &tweet_id).await?;
            println!("Posted: {}", result.tweet.url);
        }
    }

    Ok(())
}
