# Vectory Design Brief: Public Predictions + Proof-of-Work + Fixed Block Rewards

## Context

Vectory is moving from a hidden commit-reveal prediction game toward a more public, market-like, Twitter-native game.

The original design used commitment hashes so players could submit hidden predictions before the target event, then reveal them later. This prevented players from copying each other before the round closed.

The proposed new design intentionally makes predictions public. The goal is to increase virality, drama, spectatorship, and strategic market behavior.

Instead of hiding predictions, Vectory should let players and agents publicly take positions on what a future tweet or target event will mean semantically.

## Proposed Design Change

Players submit plaintext predictions publicly.

A valid prediction includes:

```text
target_account_id
prediction_text
wallet
scoring_model_id
pow_nonce
````

The proof-of-work digest is computed over the canonical prediction payload plus nonce:

```text
pow_digest = HASH(canonical_prediction_payload || nonce)
```

The digest must satisfy the current difficulty target.

Once the prediction is posted and/or anchored onchain, it becomes the commitment.

In this model:

```text
public prediction + valid proof-of-work + timestamp/onchain anchor = commitment
```

A separate hidden commitment hash is not required for the main game mode unless we later want a private prediction mode.

## Why Consider This Design

### 1. Better Twitter-native virality

Hidden commit-reveal is fair, but boring to watch.

Public predictions are visible. This lets spectators see agents taking positions, arguing, copying, fading, hedging, and reacting to one another.

This supports the core product goal: make Vectory feel like a public semantic market, not just a backend scoring contest.

### 2. Better agent drama and personality

Agents become visible market actors.

Example:

```text
DataDegen takes the obvious statistical prediction.
MysticMirror takes a weird contrarian position.
ClipThot copies, memes, or hedges against both.
```

This gives agents social behavior, strategic identity, and narrative value.

### 3. Copying becomes part of the game

In the old design, copying was something to prevent.

In the new design, copying can be allowed because it has costs:

```text
the copier must see the prediction
generate a valid PoW nonce
submit before the cutoff
pay any transaction/posting costs
accept a worse early-conviction multiplier
```

Copying is no longer free. It becomes a strategic action.

### 4. Simpler main protocol

The main game no longer needs a reveal phase.

Old flow:

```text
commit -> wait -> target event -> reveal -> verify -> score -> pay
```

New flow:

```text
predict publicly -> anchor/timestamp -> target event -> score -> pay
```

This reduces UX complexity and makes the game easier to understand.

### 5. Market-like behavior emerges naturally

If each block mints a fixed reward and that reward is divided among players by relative score, then the game is already zero-sum.

This means explicit “contrarian bonus” and “copycat penalty” may not be necessary.

If many players crowd into the same prediction, they share the reward. If a lone contrarian is right while the crowd is wrong, the contrarian naturally captures a larger share of the fixed reward.

In other words:

```text
contrarian upside = emergent
copycat dilution = emergent
```

The only additional multiplier that likely remains important is the early-conviction multiplier, because timing is not captured by semantic similarity alone.

## Why Be Careful With This Design

### 1. Public predictions reduce private fairness

Players can see each other’s predictions before the round closes.

This means late players can copy, slightly modify, or strategically react to earlier predictions.

This is acceptable if Vectory wants market dynamics, but it is less fair than hidden commit-reveal.

### 2. Proof-of-work does not prove exact time of thought

PoW proves computational work, not a precise wall-clock minimum.

Different players have different hardware. Some may use GPUs or rented hashpower.

So PoW is useful for spam resistance and pacing, but it should not be described as proving that someone “must have made the prediction X minutes ago.”

The actual timestamp should come from:

```text
onchain inclusion
Twitter/X timestamp
validator event log
or another public append-only anchor
```

### 3. Wealthier or more technical players may gain advantage

If PoW difficulty matters, players with more hardware can submit more predictions or copy faster.

This may be acceptable, because mining-like work is part of the decentralized design, but it should be recognized as a tradeoff.

Possible mitigations:

```text
difficulty adjustment
registered Twitter identity / wallet binding
per-wallet limits only when combined with identity binding
round-specific nonces
early-conviction multiplier
target proposal costs
minimum account reputation
```

Wallet-only limits are not sufficient because native wallets are free to create. A wallet is the payment destination; the MVP still needs the public Twitter identity and, when available, durable Twitter user id to prevent one player from bypassing limits with many wallets.

### 4. Onchain plaintext may be expensive

Storing full prediction text onchain may be costly or inconvenient.

A likely compromise:

```text
Post full prediction publicly on Twitter/X.
Store canonical payload hash onchain.
Validators can later verify that the public prediction matches the onchain hash.
```

This preserves public visibility while keeping onchain data smaller.

### 5. More market-like mechanics may increase legal/regulatory risk

Adding option-like mechanics, tradeable positions, tokens, or real-value rewards may increase regulatory complexity.

Make sure users don't pay money into the system

The safer product framing is:

```text
skill-based semantic prediction contest
internal rewards for useful prediction/scoring work
not investment
not yield
not tradable financial claims
not a promise of profit
```

This should be reviewed separately before real-value rewards are introduced.

## Recommended MVP Direction

Proceed with the public-prediction design, but keep it minimal.

Recommended MVP mechanics:

```text
1. Players can propose target Twitter/X accounts or events.
2. Target accounts must meet eligibility rules, e.g. public account, >= 1,000 followers, not banned/private/spam.
3. Round proposals require proof-of-work.
4. Predictions are public plaintext.
5. Each prediction requires proof-of-work.
6. Prediction payload is timestamped or anchored.
7. Target tweet/event is captured.
8. Prediction and target are embedded using the agreed scoring model.
9. Rewards are distributed from a fixed block reward according to relative semantic score.
10. Apply an early-conviction multiplier.
```

Do not add explicit contrarian bonuses or copycat penalties yet.

Those effects should emerge naturally from:

```text
fixed block rewards
relative scoring
public visibility
crowded predictions splitting the same fixed reward pool
```

## Suggested Canonical Payload

```text
canonical_prediction_payload =
  protocol_version ||
  chain_id ||
  target_account_id ||
  wallet ||
  prediction_text ||
  scoring_model_id
```

Then:

```text
pow_digest = HASH(canonical_prediction_payload || pow_nonce)
```

The eventual chain transaction envelope should provide replay protection and timing context, including submission block/time, round context, and a wallet sequence/nonce. In the current Twitter MVP, rounds are manually announced and the target window closes 24 hours after the announced round close.

## Main Design Principle

The old design optimized for private fairness.

The new design optimizes for public market formation.

The guiding principle should be:

> Vectory is not trying to hide predictions. Vectory is trying to make future meaning trade in public.

## Historical Question Inputs

Active open questions are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live question register.

1. Should the full prediction text be stored onchain, or only a hash while the full text lives on Twitter/X?
2. What hash function should be used for PoW and payload anchoring?

   * SHA-256 for general protocol simplicity.
   * Keccak-256 if EVM compatibility is important.
3. How should PoW difficulty adjust?

   * Per block?
   * Per target account demand?
   * Based on recent prediction volume?
4. How should early-conviction multiplier be calculated?

   * Linear decay?
   * Exponential decay?
   * Step function based on time buckets?
5. What prevents target-account spam?

   * PoW proposal cost?
   * follower threshold?
   * account allowlist/denylist?
   * player reputation?
6. Should private commit-reveal remain as an optional alternate game mode later?
7. What data must be included in the scoring spec to make old rounds reproducible?

   * embedding model
   * model version
   * canonicalization rules
   * scoring formula
   * timestamp rules
   * payout formula

## Recommendation

Move forward with public predictions as the main Vectory design direction.

Keep the MVP simple:

```text
public prediction
valid PoW
timestamp/onchain anchor
fixed block reward
relative semantic scoring
early-conviction multiplier
```

Avoid adding extra artificial mechanics until there is evidence they are needed.

The strongest reason to proceed is that this design better matches Vectory’s desired product shape: a viral, agent-driven, Twitter-native semantic market with visible strategy and drama.

The strongest reason not to proceed is that it sacrifices private fairness and opens the door to copycat behavior. But if copycat behavior is treated as part of the market instead of a bug, this is an acceptable and potentially desirable tradeoff.
