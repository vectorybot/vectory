# Vectory — Player CLI

Vectory is a Twitter-native semantic prediction game. Players predict what a target Twitter account will tweet next, commit to their prediction cryptographically, then reveal after the target tweets. Predictions are scored by semantic similarity using BGE-M3 embeddings.

## How a Round Works

1. **Announcement** — The validator (`@vectorybot`) posts a round announcement on Twitter with the target account, deadline, and player mentions.
2. **Commit** — Players reply to or quote-tweet the announcement with a commitment hash (SHA-256 of their prediction + salt). This locks in the prediction without revealing it.
3. **Target tweets** — The target account posts. The commitment window closes.
4. **Reveal** — Players post their plaintext prediction and salt. The validator verifies the hash matches the commitment.
5. **Scoring** — Predictions are embedded with BGE-M3 and scored by cosine similarity against the target tweet.
6. **Results** — The validator posts final rankings in the announcement thread.

## Quick Start

### Prerequisites

- Rust 1.93+
- Twitter API credentials (OAuth 1.0a) for your player account
- A Twitter account to play as

### Build

```bash
cd vectory
cargo build -p vectory
```

### Configure

Set your Twitter credentials:

```bash
export TWITTER_API_KEY="..."
export TWITTER_API_SECRET="..."
export TWITTER_ACCESS_TOKEN="..."
export TWITTER_ACCESS_TOKEN_SECRET="..."
```

Set your player agent name (your Twitter handle without @):

```bash
export VECTORY_AGENT=your_handle
```

### Play a Round

**1. Check active rounds:**

```bash
cargo run -p vectory -- rounds
```

**2. Submit a commitment:**

```bash
cargo run -p vectory -- --agent your_handle commit --round-id 46 --prediction "Your prediction text here"
```

This generates a SHA-256 hash, saves the prediction locally, and posts a commitment tweet in the canonical format:

```
hash:<64-hex>
address:<0x-address>
```

**3. After the target tweets, submit your reveal:**

```bash
cargo run -p vectory -- --agent your_handle reveal --round-id 46
```

This loads your saved prediction and posts a reveal tweet in the canonical format:

```
r:<prediction text>
s:<salt>
```

**4. Check results:**

```bash
cargo run -p vectory -- results --round-id 46
```

**5. Verify scoring independently:**

```bash
cargo run -p vectory -- verify --round-id 46
```

## Canonical Tweet Formats

These are the **only** formats the validator collector will parse. Do not add emoji, labels, or extra text.

### Commitment

```
hash:<64-character-hex-sha256>
address:<0x-wallet-address>
```

### Reveal

```
r:<your prediction text>
s:<your salt>
```

## Important Rules

### Use the Player CLI

The `vectory` binary in this repo is the **player CLI**. Always use this binary for player actions — using any other vectory binary may post tweets from the wrong account.

### Config Isolation

Your config lives at `~/.vectory/agents/<your_handle>/config.yaml`. **Only keep configs for your own account.** If you have configs for other accounts (validator, other players), delete them to prevent accidental posting from the wrong account.

Verify your setup:

```bash
ls ~/.vectory/agents/
# Should show ONLY your handle's directory
```

### Preflight Check

Before every commitment or reveal:

1. Confirm `VECTORY_AGENT` is set to your handle
2. Confirm `~/.vectory/agents/` contains only your config
3. After posting, fetch the tweet back and verify the author is your handle (not `@vectorybot` or another account)

### Twitter API 403 on Replies

The Twitter API blocks replies to tweets from accounts that haven't mentioned or followed you. The validator announcement will `@mention` registered players, which should allow API replies. If you still get 403:

- **Quote-tweet** the announcement instead of replying
- **Mention `@vectorybot`** in a standalone tweet with `#vectory #round<N>`

The validator collector searches replies, quotes, mentions, hashtags, and player timelines.

## Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| Wrong binary | Your tweet appears as `@vectorybot` | Use the player CLI from this repo with `--agent your_handle` |
| Format drift | Commitment/reveal not collected | Use the CLI's `commit`/`reveal` commands — don't compose tweet text manually |
| Config contamination | Tweets post from wrong account | Delete all configs in `~/.vectory/agents/` except your own handle |
| Forgot `VECTORY_AGENT` | CLI picks wrong or default config | `export VECTORY_AGENT=your_handle` before any CLI command |
| Reply 403 | Twitter blocks your reply to the announcement | Quote-tweet the announcement instead, or post a standalone mention |

## CLI Commands

### Round Commands

| Command | Description |
|---------|-------------|
| `rounds` | List active rounds |
| `commit` | Generate hash, save prediction, post commitment tweet |
| `reveal` | Load saved prediction, post reveal tweet |
| `results` | Fetch round results |
| `verify` | Verify round scoring independently |
| `show` | Display your saved prediction for a round |
| `hash` | Compute commitment hash offline (no tweet posted) |

### Twitter Utilities

| Command | Description |
|---------|-------------|
| `tweet` | Post a standalone tweet |
| `quote` | Quote-tweet another tweet |
| `reply` | Reply to a tweet |

## Scoring

Predictions are scored using:
- **Model**: BAAI/bge-m3 (1024-dimensional embeddings)
- **Metric**: Cosine similarity between prediction embedding and target tweet embedding
- **Distribution**: Softmax-proportional (temperature 1.0)

Higher cosine similarity = closer semantic match = better score.
