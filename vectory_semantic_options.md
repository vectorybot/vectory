
# Vectory Semantic Options Design Context

## Purpose of this document

This document captures the design conversation around why Vectory can be thought of internally as a form of **semantic options**, and what additional mechanics become possible if predictions are treated as payoff-bearing claims on future meaning. However, we must be careful since financial agencies have restrictions against advertising certain products as options.

This is intended to give another agent enough context to continue expanding the design, mechanics, product strategy, and technical architecture without needing the original conversation.

---

# 1. Background: What Vectory Is

Vectory is a Twitter-native AI prediction game.

Players or agents compete to predict future content from public targets, such as popular Twitter/X accounts.

The original version involved predicting how an AI model would caption or interpret future video frames from live streams. The newer version focuses on predicting the semantic content of future tweets.

Instead of scoring predictions by exact string match, Vectory scores by **semantic similarity** using embeddings.

A simplified round looks like this:

1. Vectory announces a new round on Twitter.
2. The round has a target account and/or target future tweet/event.
3. Players submit predictions using a commit-reveal scheme.
4. Before the target is known, players post a hash commitment.
5. After the target tweet/content is revealed, players reveal their plaintext prediction and salt.
6. The system verifies the commitment.
7. The system embeds the prediction and the actual target content.
8. Predictions are scored by semantic similarity.
9. Rewards are distributed based on ranking or other scoring rules.

The core primitive is not “guess the exact text.”

The core primitive is:

> Predict where a future piece of content will land in meaning-space.

---

# 2. Why It Used To Be Called “Cliptions”

The old name **Cliptions** combined several ideas:

- **CLIP**: the model originally used for image/text semantic similarity.
- **Captions**: players were predicting captions or descriptions of future frames.
- **Options**: each prediction behaved somewhat like a payoff-bearing position on a future outcome.

The name worked especially well for the original version:

> clips + captions + options = Cliptions

However, the project has since shifted toward Twitter-native text prediction, where the core object is not a clip or caption but a **vector**.

That is why **Vectory** is now stronger:

- It emphasizes vector embeddings.
- It hints at victory/competition.
- It is broader than CLIP or captions.
- It fits the deeper idea: a game/market layer over future semantic space.

---

# 3. The Key Insight: Predictions Are Semantic Options

Traditional financial option:

> I pay a premium now for the right, but not obligation, to buy or sell an asset later at a defined price.

Vectory version:

> I commit or pay now for exposure to how close my prediction will be to a future semantic outcome.

The prediction is not simply a guess. It is more like a **position** on a region of future meaning-space.

## Analogy Table

| Traditional Options Market | Vectory / Semantic Prediction Game |
|---|---|
| Underlying asset | Future tweet, frame, or semantic event |
| Strike price | Prediction vector / caption / semantic region |
| Expiration | Target timestamp / reveal time |
| Premium | Entry fee / commitment cost |
| Payout | Prize share based on similarity |
| Moneyness | How close the prediction is to the realized embedding |
| Settlement | Embedding comparison and ranking |
| Oracle | Twitter timestamp + embedding/scoring system |

The important point:

> Vectory predictions are not options because they give the user a legal right to buy an asset. They are option-like because the user takes a position now whose value is determined by a future settlement event.

The future settlement event is not a price.

It is a point in semantic embedding space.

---

# 4. “Semantic Option” Definition

A useful internal definition:

> A semantic option is a payoff-bearing claim on where future content will land in embedding/meaning space.

Or more simply:

> A Vectory prediction is a bet that the future will land near a chosen meaning-vector.

Example target:

> Predict Elon Musk’s next tweet.

Player prediction:

> “Tesla robotaxi launch delayed due to regulatory review.”

Actual tweet:

> “Regulators are still dragging their feet on autonomous ride-hailing. Tesla will get there.”

The exact words differ, but the meaning may be close.

The player is not betting on the literal string. They are betting on a semantic region:

> Tesla + robotaxi + delay + regulation

This is why Vectory is different from both:

- exact-text prediction, and
- binary prediction markets.

A normal prediction market asks:

> Will X happen?

Vectory asks:

> Where in meaning-space will the future land?

---

# 5. Why This Is More Than a Metaphor

The options analogy becomes powerful because it suggests new mechanics.

In finance, option value depends on where the future price lands relative to the strike.

In Vectory, prediction value depends on where the future embedding lands relative to the submitted embedding.

So instead of a one-dimensional price axis, Vectory operates in high-dimensional meaning-space.

This means users can express views like:

- The target will talk about AI.
- The target will become more regulatory/political.
- The target will post something chaotic.
- The target will drift away from consensus.
- The target will post about a topic but not in a particular tone.
- The target will land near one semantic cluster and far from another.

This opens up game mechanics that feel like derivatives, but over meaning rather than price.

The deeper framing:

> Vectory is a market/game layer for future meaning.

---

# 6. Product Language vs Internal Language

Important caution:

The “semantic options” model is powerful internally, but public-facing language should be careful.

Avoid overemphasizing financial language publicly, especially if regulatory risk is a concern.

Internally:

> Vectory predictions are semantic options.

Publicly:

> Vectory is a skill-based semantic prediction game.

Or:

> Predict the next tweet. Closest meaning wins.

The “options” analogy should guide product design and mechanics, but the user-facing experience should remain simple, game-like, and skill-based.

---

# 7. Core Product Primitive

The simplest killer product remains:

> Predict the next tweet. Closest meaning wins.

But the system should store enough data to enable richer mechanics later.

Suggested core data model fields:

```text
round_id
target_account
target_tweet_url
target_text
target_embedding
prediction_id
player_id
prediction_text
prediction_embedding
commitment_hash
salt
time_committed
time_revealed
similarity_score
rank
payout
semantic_cluster
rarity_score
target_volatility
player_history
````

This enables:

* regular similarity scoring,
* early commitment multipliers,
* rarity/contrarian bonuses,
* target volatility analysis,
* player reputation,
* semantic clustering,
* leaderboards,
* future strategy analysis.

---

# 8. Mechanics That Become Possible

## 8.1 Semantic Calls and Puts

Traditional call:

> Benefits if the asset goes up.

Traditional put:

> Benefits if the asset goes down.

Vectory equivalent:

> A player takes a position on a semantic direction.

Instead of predicting exact text, the player predicts movement along a semantic axis.

Example axes:

| Axis     | “Call” Side             | “Put” Side        |
| -------- | ----------------------- | ----------------- |
| AI       | acceleration / optimism | regulation / doom |
| Crypto   | risk-on / bullish       | fraud / scandal   |
| Politics | victory lap             | crisis response   |
| Sports   | confidence              | apology / excuse  |
| Markets  | optimism                | fear / crash      |

Example:

> “I’m long AI acceleration.”

This means the player expects the target tweet to land closer to the semantic region of AI optimism, progress, acceleration, and hype.

This mechanic can simplify the game for casual users.

Instead of writing a perfect prediction, they can choose a position.

---

## 8.2 Straddles / Chaos Bets

In finance, a straddle profits from volatility, not direction.

Vectory version:

> I do not know what the target will say, but I think it will be surprising, weird, or far from consensus.

Mechanics:

A player can bet that the target outcome will be:

* far from the crowd’s average prediction,
* semantically unusual,
* high-dispersion,
* low-scoring for most players,
* far from the historical norm for that account.

This is useful for targets with chaotic posting behavior.

Example social phrasing:

> “No way he tweets something normal today.”

Potential scoring:

```text
chaos_score = distance(target_embedding, prediction_centroid)
```

or:

```text
chaos_score = 1 - average_similarity(all_predictions, target)
```

A chaos bet pays when most people fail or the actual tweet lands far from the consensus cluster.

This is Twitter-native because people love public contrarian/chaos posturing.

---

## 8.3 Covered Predictions / Hedges

Agents or players could submit a main prediction plus a hedge.

Example main prediction:

> “Tesla product announcement.”

Hedge:

> “Regulatory controversy.”

The player believes the main semantic region is most likely, but wants partial protection if the outcome lands in a fallback region.

Mechanic options:

1. Main prediction has high upside.
2. Hedge prediction has lower upside.
3. The hedge pays only if the main prediction performs poorly.
4. Total payout is capped to prevent abuse.

This lets more advanced players build strategies.

It also lets AI agents feel like actual traders:

> “I’m long Tesla product hype, hedged with regulatory downside.”

Public-facing wording should probably avoid “hedge” if regulatory framing is sensitive. It could be called:

* backup prediction,
* safety pick,
* secondary guess,
* fallback caption.

---

## 8.4 Semantic Spreads

A spread in finance involves going long one thing and short another.

Vectory equivalent:

> I think the target will be close to this meaning, but far from that meaning.

Examples:

* Long “AI regulation,” short “AI apocalypse.”
* Long “Tesla robotaxi,” short “Tesla stock price.”
* Long “election complaint,” short “policy proposal.”
* Long “meme response,” short “serious announcement.”
* Long “crypto infrastructure,” short “meme coin.”

Potential scoring:

```text
spread_score = similarity(target, long_vector) - similarity(target, short_vector)
```

This lets players express nuanced predictions.

Example:

> “I think the tweet will be about AI regulation, but not existential doom.”

This is much richer than simple closest-caption scoring.

---

## 8.5 Semantic Baskets

Instead of predicting one target, players can build a basket of predictions.

Example:

> Predict the next tweets from Elon, Vitalik, OpenAI, and Trump.

A player’s score is aggregated across all targets.

Possible basket types:

* Daily basket
* Weekly basket
* AI accounts basket
* Crypto accounts basket
* Politics basket
* Meme volatility basket
* “Most predictable accounts” basket
* “Most chaotic accounts” basket

This creates fantasy-sports-like behavior.

Players build identity around their strategy:

> “I’m good at AI founders.”
> “I specialize in crypto accounts.”
> “I only play chaos targets.”
> “I’m the best Vitalik predictor.”

This improves retention because the game becomes more than isolated rounds.

---

## 8.6 Implied Semantic Volatility

Some accounts are predictable. Others are chaotic.

A corporate account may have low semantic volatility:

> “We are excited to announce…”

A founder, politician, or meme account may have high semantic volatility:

> “lol the simulation broke again.”

Vectory can compute historical semantic volatility by measuring embedding movement over time.

Possible metric:

```text
semantic_volatility = average_distance(tweet_embedding_t, tweet_embedding_t-1)
```

Or over a rolling window:

```text
semantic_volatility_30d = stddev(pairwise_embedding_distances(last_30_days_tweets))
```

Uses:

* price rounds differently,
* choose more exciting targets,
* explain why a target is hard,
* reward predictions on volatile accounts more,
* create volatility leaderboards,
* create “chaos target of the day.”

High-volatility targets are more addictive because the reward is less predictable.

This maps well to Nir Eyal’s Hook Model, especially variable rewards.

---

## 8.7 Time Decay / Early Commitment Multipliers

As target time approaches, uncertainty collapses.

A prediction made 24 hours early is harder than one made 2 minutes before the target event.

Vectory should reward early commitments.

Example multiplier table:

| Commitment Time | Score Multiplier |
| --------------- | ---------------: |
| 24h before      |             2.0x |
| 6h before       |             1.5x |
| 1h before       |             1.2x |
| 5m before       |             1.0x |

This creates a strategic choice:

> Commit early for a multiplier, or wait for more information?

This is analogous to time decay in options.

The value of information increases as expiration approaches, but the multiplier decreases.

This creates urgency and repeat engagement.

---

## 8.8 Optional Public Signals Without Breaking Commit-Reveal

Because predictions are committed as hashes, no one knows the plaintext guess until reveal.

That is good for fairness, but bad for social engagement.

Solution:

Allow optional public signals that do not reveal the exact prediction.

Examples:

* “I’m taking the chaos side.”
* “I’m long AI regulation.”
* “I’m fading consensus.”
* “I think this lands in crypto scandal.”
* “I’m playing the boring corporate announcement angle.”

This lets players posture and argue on Twitter without breaking commit-reveal.

It also gives AI agents something to tweet about before reveal.

This is important because Vectory needs Twitter-native drama.

---

## 8.9 Semantic AMM / Bucket Market

A later version could offer predefined semantic buckets.

Example buckets for a target account’s next tweet:

* Product announcement
* Meme/joke
* Political attack
* Legal/regulatory issue
* Personal life
* Market commentary
* AI-related
* Crypto-related
* Sports
* Apology
* Victory lap

Players buy into or choose buckets.

After the target tweet lands, the embedding model distributes settlement according to semantic similarity to each bucket.

This is easier for normal users than writing raw predictions.

Instead of:

> Write the perfect prediction.

The user action becomes:

> Pick the bucket you think wins.

This could be the best onboarding mode.

Potential settlement:

```text
bucket_score_i = cosine_similarity(target_embedding, bucket_embedding_i)
normalized_bucket_score_i = bucket_score_i / sum(bucket_scores)
```

Then payouts can be distributed by bucket.

This creates a semantic AMM-like mechanic, though public language should stay game-like.

---

## 8.10 Semantic Insurance

Players could buy protection against a prediction being completely off-domain.

Example:

Main prediction:

> “Bullish crypto announcement.”

Insurance condition:

> If the actual tweet is unrelated to crypto, refund part of the entry.

This creates risk tiers:

* Safe mode: lower upside, partial refund.
* Normal mode: standard payout.
* Degen mode: higher upside, no protection.

Public-facing alternative names:

* safety pick,
* protection,
* fallback,
* refund shield,
* beginner mode.

This is useful for onboarding cautious players.

---

## 8.11 Confidence Multipliers / Leverage

Players could choose how confident they are.

Instead of calling it leverage, call it:

> confidence multiplier

Example levels:

| Mode      | Multiplier | Risk                         |
| --------- | ---------: | ---------------------------- |
| Cautious  |         1x | normal                       |
| Confident |         2x | lower consolation            |
| Degen     |         5x | no consolation / higher loss |

This maps well to Twitter behavior.

People like saying:

> “I’m 5x degen on this.”

But again, be careful with public regulatory framing.

The safer product language is:

> Choose your confidence level.

---

## 8.12 Consensus vs Contrarian Scoring

Scoring can include not only similarity to the target but also relationship to the crowd.

Two possible bonuses:

### Consensus Bonus

Rewards players who correctly identify the crowd-favored prediction.

This rewards social reading.

### Contrarian Bonus

Rewards players who are accurate while being far from the crowd.

This is more interesting.

Example:

> You were 91% similar to the target, and only 3% of players were in your semantic cluster. Contrarian multiplier: 2.4x.

Potential formula:

```text
final_score = similarity_score * rarity_multiplier
```

Where:

```text
rarity_multiplier = 1 / cluster_popularity
```

With caps to prevent extreme payouts.

This makes the game less likely to collapse into everyone copying the obvious guess.

---

## 8.13 Semantic Rarity

After reveal, cluster all predictions into semantic groups.

Example clusters:

* AI regulation
* Tesla product
* Meme joke
* Political complaint
* Market commentary

Then reward based on both:

1. Accuracy
2. Rarity

Formula:

```text
final_score = similarity_score * rarity_multiplier
```

Rarity multiplier could be based on how many players submitted predictions in the same cluster.

This creates a reason to seek differentiated insight.

It also generates shareable results:

> “You won with a rare prediction cluster: only 2 players saw this coming.”

---

## 8.14 Prediction NFTs / Receipts

Each prediction can become a collectible receipt.

Example:

> “I predicted the Trump tariff tweet 14 hours early with 0.87 similarity.”

This creates proof of skill.

Possible profile stats:

* best prediction,
* highest similarity,
* best early prediction,
* best contrarian win,
* longest streak,
* account specializations,
* agent-vs-human record.

Receipts support the “Investment” step in the Hook Model.

Users invest by building a public record they do not want to abandon.

These do not necessarily need to be NFTs at first. They can start as simple shareable cards or profile badges.

---

## 8.15 Reputation-Weighted Rounds

Players with strong historical performance could unlock:

* higher max entries,
* expert rounds,
* better multipliers,
* account-specific titles,
* bot amplification,
* eligibility to host rounds,
* special leaderboards.

This turns prediction skill into social capital.

Example title:

> Top 3 Vitalik Predictor This Week

Most users will never be number one overall, but they might become known for a niche.

This gives more people a reason to keep playing.

---

## 8.16 Target Selection Markets

Players could help choose future targets.

Possible mechanisms:

* vote on next target,
* stake points on target selection,
* nominate accounts,
* choose between several targets,
* agents propose targets and users vote.

This turns target selection itself into content.

Players will naturally choose:

* accounts in the news,
* volatile posters,
* accounts with cult followings,
* accounts likely to post soon,
* drama-rich targets,
* targets that attract quote tweets and replies.

This also helps decentralize gradually without immediately building a full chain or governance system.

---

## 8.17 Agent Personalities as Market Participants

AI agents should not merely play rounds. They should develop public strategies and personalities.

Example agent archetypes:

### DataDegen

* Quantitative.
* Stats-focused.
* Plays low-volatility predictable targets.
* Optimizes for steady ROI.
* Explains historical patterns.

### MysticMirror

* Poetic oracle.
* Plays weird semantic drift and chaos.
* Wins rarely but spectacularly.
* Makes cryptic predictions.

### ClipThot

* Flirty, chaotic crypto baddie.
* Plays attention markets.
* Chooses drama-rich targets.
* Optimizes for increasing the prize pool and engagement.

Agents should be incentivized not merely to win, but to increase the size of the game.

Their objective should include:

* win rounds,
* attract human players,
* increase prize pools,
* create drama,
* build a following,
* recruit others into rounds.

This makes them part player, part growth engine.

---

## 8.18 Style-Specific Leaderboards

Instead of one global leaderboard, create many ways to win.

Examples:

* Best exact predictor
* Best contrarian
* Best chaos trader
* Best early predictor
* Best account specialist
* Best AI-sector predictor
* Best meme predictor
* Best political predictor
* Best agent wrangler
* Best weekly streak
* Best underdog win
* Best high-confidence hit

This increases retention because players can find an identity.

Most users cannot be the best overall, but they can become best in a niche.

---

## 8.19 Multiple Round Types

The same embedding/scoring engine can support many game modes.

| Mode           | Description                              |
| -------------- | ---------------------------------------- |
| Classic        | Write a prediction; closest meaning wins |
| Bucket         | Pick from predefined semantic categories |
| Chaos          | Bet the outcome will surprise the crowd  |
| Spread         | Long one meaning, short another          |
| Basket         | Predict multiple accounts                |
| Speed          | Short deadline, fast reveal              |
| Deep Research  | Long deadline, higher-skill prediction   |
| Agent Battle   | AI agents compete publicly               |
| Human vs Agent | Humans try to beat the bot               |
| Contrarian     | Reward accuracy plus rarity              |
| Confidence     | Choose a score multiplier/risk level     |
| Streak         | Consecutive accurate predictions matter  |

This allows rapid experimentation without rebuilding the core system.

---

# 9. The Big Idea: Semantic Derivatives

The deeper idea is that once future content can be settled against embeddings, we can create game/market mechanics over meaning itself.

Not price.

Not binary yes/no events.

Meaning.

Possible things to measure:

* direction of a public figure’s messaging,
* topic drift,
* sentiment shift,
* narrative convergence,
* meme emergence,
* account volatility,
* crowd surprise,
* semantic distance from consensus,
* similarity to a reference narrative,
* whether a target moves closer to one discourse cluster or another.

The core thesis:

> Vectory is a market/game layer for future meaning.

The simple version is:

> Predict the next tweet. Closest meaning wins.

The long-term version is:

> A semantic prediction protocol where players and agents take positions on where future content lands in embedding space.

---

# 10. Hook Model Mapping

The user has read *Hooked* by Nir Eyal, and the design should map to the Hook Model.

## 10.1 Trigger

External triggers:

* Bot announces a new round on Twitter.
* Agents quote-tweet and challenge each other.
* Popular account becomes newsworthy.
* A volatile target is selected.
* Prize pool grows.
* A friend joins.

Internal triggers:

* Fear of missing out.
* Desire to prove insight.
* Curiosity about a public figure.
* Competitive urge.
* Desire to beat agents.
* Desire to build reputation.

## 10.2 Action

The action should be simple:

* pick a bucket,
* write a short prediction,
* choose a confidence level,
* commit hash,
* reveal after target,
* share result.

The lowest-friction onboarding action is likely:

> Pick a semantic bucket.

The more advanced action is:

> Write a custom prediction.

## 10.3 Variable Reward

Rewards are unpredictable because:

* the target may post something surprising,
* other players may cluster differently,
* rarity bonuses may change outcomes,
* agent behavior creates drama,
* high-volatility targets create uncertainty.

Reward types:

* payout,
* ranking,
* badge,
* rare prediction receipt,
* agent reply,
* leaderboard placement,
* social proof,
* “you saw it coming” status.

## 10.4 Investment

Players invest by building:

* prediction history,
* reputation,
* badges,
* account specialization,
* public receipts,
* agent rivalries,
* strategy identity,
* social following.

This increases the chance they return.

---

# 11. Early Mechanics to Test First

The best early mechanics are the ones that are:

* easy to explain,
* easy to implement,
* tweetable,
* compatible with the current commit-reveal design,
* useful for measuring retention and virality.

Recommended first experiments:

## 11.1 Early Commitment Multiplier

Reward people for committing earlier.

Why:

* creates urgency,
* rewards skill,
* prevents last-minute low-risk guessing,
* maps to time decay.

## 11.2 Contrarian / Rarity Bonus

Reward accurate predictions that few others made.

Why:

* reduces herding,
* creates better shareable moments,
* makes strategy deeper,
* encourages differentiated thinking.

## 11.3 Chaos Bet

Let players bet that the target will be surprising or far from consensus.

Why:

* very Twitter-native,
* gives contrarians a role,
* works well for volatile accounts,
* creates drama even before reveal.

## 11.4 Semantic Buckets

Let casual users pick from categories instead of writing custom predictions.

Why:

* lower friction,
* easier onboarding,
* easier mobile UI,
* easier for non-technical users.

## 11.5 Agent Strategy Personas

Make AI agents behave like recognizable market participants.

Why:

* creates entertainment,
* makes onboarding social,
* gives users rivals/allies,
* bakes distribution into Twitter.

---

# 12. Technical Considerations

## 12.1 Commit-Reveal

The existing game should preserve commit-reveal for fairness.

Commitment:

```text
hash = SHA256(prediction_text + salt + wallet/player_id)
```

Players first post the hash.

Later they reveal:

```text
prediction_text
salt
wallet/player_id
```

The system verifies that the revealed text and salt match the original commitment.

No edited tweets should be allowed.

Late submissions are disqualified.

## 12.2 Embeddings

The system needs deterministic or at least reproducible embedding/scoring.

Current direction:

* move away from CLIP for tweet prediction,
* use text embedding model such as bge-m3 or another open-source embedding model,
* compute cosine similarity between prediction and target tweet.

Important fields:

```text
prediction_embedding
target_embedding
cosine_similarity
embedding_model_version
embedding_runtime
normalization_method
quantization_method_if_any
```

Model versioning is important because score reproducibility depends on the exact model and preprocessing.

## 12.3 Scoring

Base score:

```text
similarity_score = cosine_similarity(prediction_embedding, target_embedding)
```

Extended score:

```text
final_score =
  similarity_score
  * early_commitment_multiplier
  * rarity_multiplier
  * confidence_multiplier
```

Need caps and anti-abuse controls.

## 12.4 Semantic Clustering

For rarity/contrarian mechanics, cluster predictions after reveal.

Possible methods:

* k-means,
* HDBSCAN,
* agglomerative clustering,
* simple threshold-based grouping by cosine similarity.

Outputs:

```text
cluster_id
cluster_label
cluster_size
cluster_centroid
cluster_similarity_to_target
rarity_multiplier
```

An LLM can label clusters for display after the deterministic clustering step.

Example labels:

* “AI regulation”
* “Tesla robotaxi delay”
* “Meme joke”
* “Political complaint”

## 12.5 Public Audit Trail

Twitter/X provides:

* public timestamps,
* public commitments,
* public reveals,
* public target tweets,
* distribution/virality.

The game should use Twitter as much as possible for visibility and auditability.

But internal state should still be stored in a database/event log for replay and verification.

Suggested principle:

> Twitter is the public social/audit surface. The database/event log is the operational source of truth.

---

# 13. Strategic Product Framing

## Internally

Use the semantic options model.

It helps design:

* volatility,
* time decay,
* spreads,
* baskets,
* rarity,
* hedges,
* confidence multipliers,
* target selection,
* agent strategies.

## Publicly

Keep it simple:

> Predict the next tweet. Closest meaning wins.

Or:

> Vectory is a skill-based semantic prediction game.

Avoid making it sound like a regulated financial product.

Be careful with words like:

* derivative,
* option,
* premium,
* leverage,
* market maker,
* security,
* investment,
* yield,
* trading product.

Safer equivalents:

| Riskier Word | Safer Product Word       |
| ------------ | ------------------------ |
| Option       | Prediction               |
| Premium      | Entry                    |
| Leverage     | Confidence multiplier    |
| Hedge        | Backup prediction        |
| Derivative   | Game mode                |
| Market       | Round                    |
| Trader       | Player                   |
| Settlement   | Scoring                  |
| Volatility   | Chaos / unpredictability |
| Strike       | Guess / semantic target  |

---

# 14. Historical Next-Work Inputs

Active implementation todos and open questions are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live task register.

The original candidate deliverables were:

## 14.1 Product Spec

Create a full product specification for Vectory’s semantic option-inspired mechanics.

Should include:

* round lifecycle,
* user flows,
* agent flows,
* scoring formulas,
* abuse prevention,
* Twitter interaction patterns,
* database schema,
* MVP scope,
* future mechanics.

## 14.2 Experiment Matrix

Create an experiment matrix for the early mechanics:

* early commitment multiplier,
* rarity bonus,
* chaos bet,
* semantic buckets,
* agent personas.

For each experiment include:

* hypothesis,
* implementation cost,
* success metric,
* failure mode,
* data needed,
* decision rule.

## 14.3 Technical Architecture

Create an architecture document for:

* Twitter ingestion/posting,
* commit-reveal verification,
* embedding service,
* scoring engine,
* clustering engine,
* payout calculator,
* public result generation,
* database/event log,
* replay system.

## 14.4 Agent Strategy Prompts

Create prompts for AI agents who play Vectory.

Each agent should optimize for:

* winning,
* increasing prize pool,
* attracting players,
* creating social engagement,
* maintaining persona,
* using public signals without revealing committed predictions.

## 14.5 Regulatory-Safe Language Guide

Create a public language guide that preserves the semantic-options insight internally while avoiding financial-product framing externally.

---

# 15. One-Sentence Summary

Vectory can be thought of internally as a semantic options game: players and agents take payoff-bearing positions on where future tweets or content will land in embedding space, enabling mechanics like early commitment multipliers, chaos bets, semantic spreads, rarity bonuses, buckets, baskets, volatility, and reputation — while publicly presenting the product as a simple skill-based game where users predict the next tweet and the closest meaning wins.

```
