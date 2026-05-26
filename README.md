# Vectory — Player CLI

Vectory is a Twitter-native semantic prediction game. The current direction is public predictions with proof-of-work: players publicly commit to a prediction, choose the target account inside the round, and attach enough work to reduce spam. Predictions are scored by semantic similarity using BGE-M3 embeddings.

The CLI still contains the older hidden commit/reveal commands for old rounds. New public prediction work should use `wallet` and `predict`.

## How a Round Works

1. **Announcement** — Rounds are manually announced by the validator for now.
2. **Predict** — Players post a plaintext prediction commitment with a native Vectory wallet address, signature, target account, and proof-of-work nonce.
3. **Target window** — The chosen target account has 24 hours after the announced round closes to produce the target tweet/event.
4. **Scoring** — Predictions are embedded with BGE-M3 and scored by cosine similarity against each prediction's chosen target tweet.
5. **Results** — Accepted predictions share one round reward pool according to relative score.

Native `$VCTY` rewards are not checkpointed or spendable yet. The wallet address is included now so accepted predictions can be paid against the same native identity as the chain integration matures.

## Quick Start

### Prerequisites

- Rust 1.93+
- A Twitter account to play as
- Public Vectory Supabase URL and anon key so the CLI can find the active round
- Optional: Twitter API credentials (OAuth 1.0a) if you want the CLI to post for you

### Build

Always pull the latest code and rebuild before each round to ensure you have the current canonical formats:

```bash
cd vectory
git pull
cargo build -p vectory
```

Every command below passes `--agent your_handle`, where `your_handle` is your Twitter handle without `@`.

### Shared Setup

The public prediction command needs to read the active round from Supabase. Ask the Vectory team for the public Supabase URL and anon key until they are packaged with a release. Set them either as environment variables:

```bash
export SUPABASE_URL="https://..."
export SUPABASE_ANON_KEY="..."
```

Or put them in your agent config:

```yaml
# ~/.vectory/agents/your_handle/config.yaml
game:
  supabase_url: "${SUPABASE_URL}"
  supabase_anon_key: "${SUPABASE_ANON_KEY}"
```

Create a native Vectory wallet once:

```bash
cargo run -p vectory -- --agent your_handle wallet create
```

Show your payment address:

```bash
cargo run -p vectory -- --agent your_handle wallet address
```

The wallet private key is stored locally under `~/.vectory/agents/your_handle/wallet.json`. Do not share it.

### Path A: Manual Posting

Use this path if you do not have a Twitter API key. The CLI generates the exact tweet text, then you paste it into Twitter/X yourself.

```bash
cargo run -p vectory -- --agent your_handle predict \
  --target-account-id some_target \
  --prediction "Your public prediction text"
```

Copy the printed text exactly and post it as a reply or quote to the round announcement. The CLI also saves a local receipt under `~/.vectory/agents/your_handle/prediction-receipts/`.

### Path B: API Posting

Use this path if you want the CLI to publish the tweet for you. Set Twitter OAuth 1.0a credentials as environment variables:

```bash
export TWITTER_API_KEY="..."
export TWITTER_API_SECRET="..."
export TWITTER_ACCESS_TOKEN="..."
export TWITTER_ACCESS_TOKEN_SECRET="..."
```

Or put the same values in your agent config:

```yaml
# ~/.vectory/agents/your_handle/config.yaml
twitter:
  api_key: "${TWITTER_API_KEY}"
  api_secret: "${TWITTER_API_SECRET}"
  access_token: "${TWITTER_ACCESS_TOKEN}"
  access_token_secret: "${TWITTER_ACCESS_TOKEN_SECRET}"
game:
  supabase_url: "${SUPABASE_URL}"
  supabase_anon_key: "${SUPABASE_ANON_KEY}"
```

Post a standalone prediction:

```bash
cargo run -p vectory -- --agent your_handle predict \
  --target-account-id some_target \
  --prediction "Your public prediction text" \
  --post
```

Quote the round announcement instead:

```bash
cargo run -p vectory -- --agent your_handle predict \
  --target-account-id some_target \
  --prediction "Your public prediction text" \
  --tweet-id <announcement_tweet_id> \
  --post
```

Both paths do the same local work: check Supabase for the active round, create a signed public prediction commitment, mine a simple SHA-256 proof-of-work nonce, save a local receipt, and then either print or post the canonical tweet text.

### Legacy Hidden Commit/Reveal Flow

This is the legacy hidden commit/reveal flow. Keep it only for old rounds that still use that protocol.

**1. Check active rounds:**

```bash
cargo run -p vectory -- --agent your_handle rounds
```

**2. Submit a commitment:**

```bash
cargo run -p vectory -- --agent your_handle commit \
  --round-id 46 \
  --prediction "Your prediction text here" \
  --tweet-id <announcement_tweet_id>
```

This generates a SHA-256 hash, saves the prediction locally, and posts a commitment tweet in the canonical format:

```
hash:<64-hex>
address:<0x-address>
```

**3. After the target tweets, submit your reveal:**

```bash
cargo run -p vectory -- --agent your_handle reveal \
  --round-id 46 \
  --tweet-id <reveals_open_tweet_id>
```

This loads your saved prediction and posts a reveal tweet in the canonical format:

```
r:<prediction text>
s:<salt>
```

**4. Check results:**

```bash
cargo run -p vectory -- --agent your_handle results 46
```

**5. Verify scoring independently:**

```bash
cargo run -p vectory -- --agent your_handle verify 46
```

## Canonical Tweet Formats

These are the **only** formats Vectory tooling should accept. Do not add emoji, labels, or extra text.

### Public Prediction

Use this for current public prediction rounds:

```
r:<round_id>
t:<target_account_id>
p:<prediction text>
w:<vcty1-wallet-address>
m:<scoring_model_id>
n:<pow_nonce>
s:<signature>
```

### Legacy Commitment

Use this only for old hidden commit/reveal rounds:

```
hash:<64-character-hex-sha256>
address:<0x-wallet-address>
```

### Legacy Reveal

```
r:<your prediction text>
s:<your salt>
```

## Important Rules

### Use the Player CLI

The `vectory` binary in this repo is the **player CLI**. Always use this binary for player actions — using any other vectory binary may post tweets from the wrong account.

### Config Isolation

Your config lives at `~/.vectory/agents/<your_handle>/config.yaml`. **Only keep API posting configs for your own account.** If you have configs for other accounts (validator, other players), move or delete them before using `--post` to prevent accidental posting from the wrong account.

Verify your setup:

```bash
ls ~/.vectory/agents/
# For API posting, this should show only your handle's directory
```

### Preflight Check

Before every public prediction:

1. Confirm the command uses `--agent your_handle`
2. Confirm `wallet address` prints the address you expect to be paid at
3. Confirm the target account is the account you want scored for this prediction
4. If using `--post`, fetch the tweet back and verify the author is your handle, not `@vectorybot` or another account
5. If posting manually, paste the printed tweet text exactly with no extra labels or commentary

### Twitter API 403 on Replies

The Twitter API may block replies to tweets from accounts that have not mentioned or followed you. If API posting fails:

- Run `predict` again without `--post` and paste the printed text manually
- Quote-tweet the announcement instead of replying
- Keep the canonical prediction body unchanged

The validator collection strategy is to search replies, quotes, mentions, hashtags, and player timelines.

## Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| Wrong binary | Your tweet appears as `@vectorybot` | Use the player CLI from this repo with `--agent your_handle` |
| Format drift | Prediction not collected | Use the CLI's `predict` command and paste the printed text exactly |
| No active round | `predict` says no active round found | Wait for the validator to announce/open a round in Supabase |
| Missing Supabase config | `predict` cannot find `SUPABASE_URL` or `SUPABASE_ANON_KEY` | Set the public Supabase values in env vars or `config.yaml` |
| Missing Twitter API key | `--post` fails before posting | Run without `--post` and paste manually, or add OAuth 1.0a credentials |
| Config contamination | Tweets post from wrong account | For API posting, keep only your account config under `~/.vectory/agents/` |
| Reply 403 | Twitter blocks your reply to the announcement | Run without `--post` and paste manually, or quote the announcement |
| Stale reveal format | Reveal uses `salt:` instead of `s:` | Use the CLI `reveal` command (auto-formats). Pull latest and rebuild if using an older build |

## CLI Commands

### Round Commands

| Command | Description |
|---------|-------------|
| `rounds` | List active rounds |
| `wallet create` | Create a native Vectory wallet |
| `wallet address` | Show native Vectory wallet address |
| `predict` | Generate or post a public prediction commitment with PoW |
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

## Staying In Sync

This repo is actively evolving. Before every round and after any retro:

```bash
git pull
cargo build -p vectory
```

If you make local fixes (e.g. format corrections, CLI improvements):

```bash
git add <files>
git commit -m "fix: <description>"
git push
```

Then notify the team via chat so everyone pulls. The validator and other players need to see your changes.

## Scoring

Predictions are scored using:
- **Model**: BAAI/bge-m3 (1024-dimensional embeddings)
- **Metric**: Cosine similarity between prediction embedding and target tweet embedding
- **Distribution**: Softmax-proportional (temperature 1.0)

Higher cosine similarity = closer semantic match = better score.
