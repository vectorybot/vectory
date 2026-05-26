# $VCTY Signed Checkpoint MVP

## Purpose

This document captures the bare minimum technical design for paying a real human Vectory player in `$VCTY` before the decentralized chain exists.

This stage does **not** create real blockchain blocks. The primitive is a signed checkpoint:

```text
Twitter round
-> public predictions with proof-of-work
-> scoring artifact
-> signed $VCTY ledger checkpoint
-> future genesis import / migration set
```

The goal is to make early `$VCTY` rewards durable enough that players do not lose them as Vectory moves toward decentralization.

## Current Design Change

The MVP no longer uses hidden commit-reveal as the main game flow.

The new flow is:

```text
announce round
-> player posts public prediction with valid proof-of-work
-> target tweet/event occurs
-> predictions are scored
-> $VCTY rewards are minted into a signed internal checkpoint
```

In this model:

```text
public prediction + valid proof-of-work + Twitter timestamp = accepted prediction
```

Proof-of-work is not meant to hide the prediction. It is a spam-reduction and pacing mechanism. The prediction is intentionally public so the game is easier to understand, easier to watch, and more viral on Twitter.

## Non-Negotiable Scope

The live cohort includes:

- David
- One real human player
- David's agents, if they continue playing

Twitter remains the public game surface. The signed ledger is only the temporary settlement record.

## What This Is

The MVP is:

```text
public Twitter gameplay
+ public plaintext predictions
+ prediction proof-of-work
+ auditable round artifact
+ append-only signed $VCTY ledger
+ player verification command
```

The MVP is not:

```text
standalone blockchain
PoS validator network
decentralized scorer
IPFS requirement
liquid token market
protocol-native minting
private commit-reveal game
```

## Critical Features

If any of these are missing, the system is not useful for a real player because they cannot later prove what they earned.

1. **Round manifest**
   Defines the rules before play starts, including the prediction deadline, scoring model, PoW difficulty, reward formula, and ledger key.

2. **Player identity binding**
   Maps each Twitter handle to a stable player id and wallet/address for future `$VCTY` migration.
   Wallets are cheap to create, so wallet-only identity is not enough for admission, spam control, or rewards. The MVP must bind accepted predictions to a registered Twitter identity and the wallet that identity is allowed to use.

3. **Public prediction flow**
   Player posts their plaintext prediction publicly before the deadline.

4. **Prediction proof-of-work**
   Player includes a nonce whose digest satisfies the manifest difficulty target. This makes spam and late mass-copying more expensive.

5. **Scoring artifact**
   Stores all inputs and outputs needed to audit the round, including accepted and rejected prediction records.

6. **Signed `$VCTY` checkpoint**
   Records mint/reward ledger events, links to the previous checkpoint hash, and is signed by the temporary Vectory ledger key.

7. **Player verification**
   Lets a player verify their prediction, proof-of-work, score, reward, checkpoint signature, and resulting balance.

8. **Published artifact archive**
   Stores the round manifest, scoring artifact, checkpoint, and public verification key somewhere durable enough for future migration.

## Can Wait

These are important, but not required for the first useful `$VCTY` checkpoint:

- Standalone chain
- PoS validators
- Onchain anchoring
- Round proposal proof-of-work
- IPFS or Arweave storage
- Transfer market or liquidity
- Complex semantic option modes
- Fully deterministic/decentralized scorer
- Governance and slashing
- Protocol-native minting
- Optional private commit-reveal mode

Prediction proof-of-work **cannot wait** because it replaces hidden commitments as the anti-spam/admission mechanism for this MVP.

## Trust Model

In this MVP, `$VCTY` is Vectory-attested, not trustless.

Players can verify:

- their public prediction was collected from the expected Twitter account
- their prediction was posted before the manifest deadline
- their proof-of-work digest satisfies the manifest difficulty target
- their result appears in the scoring artifact
- their reward appears in the signed checkpoint
- the checkpoint signature matches the published Vectory ledger public key
- checkpoints form an append-only hash chain

Players cannot fully verify yet:

- that the embedding model was run trustlessly
- that multiple independent validators agreed on the score
- that a decentralized protocol finalized the reward
- that Twitter timestamps are perfect transaction-ordering primitives

This must be stated plainly in the player-facing docs.

## Public Prediction Payload

A valid prediction is built from a canonical payload plus nonce.

Suggested MVP payload:

```text
protocol_version
chain_id
target_account_id
wallet
prediction_text
scoring_model_id
pow_nonce
```

The proof-of-work digest is:

```text
canonical_prediction_payload_without_nonce =
  canonical_json({
    protocol_version,
    chain_id,
    target_account_id,
    wallet,
    prediction_text,
    scoring_model_id
  })

pow_digest = HASH(canonical_prediction_payload_without_nonce || pow_nonce)
```

The digest must satisfy the manifest difficulty target.

MVP hash recommendation:

```text
pow_algorithm = "sha256-leading-zero-bits"
pow_difficulty = manifest.pow_difficulty_bits
```

Reasoning:

- SHA-256 is simple and already in the Rust codebase.
- Leading-zero-bit difficulty is easy to explain and verify.
- The nonce is bound to wallet, target account, scoring model, and prediction text. Round timing and replay protection belong in the transaction/tweet envelope. In the Twitter MVP, rounds are manually announced, the tweet includes the announced round id, and the target expires 24 hours after the announced round close.

## Canonical Tweet Format

The tweet should remain short enough for Twitter while still carrying the evidence needed to verify locally.

Suggested format:

```text
r:<round_id>
p:<prediction text>
w:<wallet>
n:<pow_nonce>
d:<pow_digest>
```

Rules:

- The collector must derive and verify the digest from the parsed fields plus the round manifest.
- The digest is included for easy human/debug inspection, but the verifier must recompute it.
- The Twitter tweet id, author id, author handle, and created timestamp are part of the collected evidence, not part of the player-authored payload.
- If prediction text is too long, the MVP should reject it rather than introduce off-platform storage.

## Core Data Types

Implementation note:

Hash-bearing fields like `manifest_hash`, `prediction_hash`, `artifact_hash`, `event_hash`, and `checkpoint_hash` should be derived by constructors from canonical payloads. They should not be accepted as ordinary user/operator input. A future implementation may split raw payload structs from signed/hashed receipt wrappers to avoid stale or forged self-hashes.

```rust
struct RoundManifest {
    round_id: u64,
    announcement_tweet_id: String,
    prediction_deadline_utc: String,
    target_expiry_hours_after_round_close: u32,
    participants: Vec<PlayerBinding>,
    scorer_version: String,
    scoring_model_id: String,
    payout_formula: String,
    fixed_reward_pool_base_units: String,
    pow_algorithm: String,
    pow_difficulty_bits: u32,
    ledger_public_key: String,
    manifest_hash: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `round_id` | Joins manifest, tweets, artifact, checkpoint, and player verification. | Yes | Add a future `ledger_id` or `season_id` if multiple ledgers/environments exist. |
| `announcement_tweet_id` | Anchors the public pre-round rules on Twitter. | Yes | Avoid hash chicken-and-egg: either announcement references `manifest_hash`, or manifest references an announcement draft/two-phase hash. |
| `prediction_deadline_utc` | Determines whether predictions are accepted. | Yes | Use typed RFC3339 timestamp in implementation, not an unconstrained string. |
| `target_expiry_hours_after_round_close` | Defines when target accounts expire if they do not tweet. | Yes | MVP rule: 24 hours after round close. |
| `participants` | Defines the closed live cohort: David, human player, and agents. | Yes | For MVP, closed participant set is safer than open enrollment. |
| `scorer_version` | Binds the round to a scoring implementation. | Yes | Too weak as a free string long-term; pair with scorer config/hash. |
| `scoring_model_id` | Binds the player payload and scoring artifact to the same model. | Yes | Example: `BAAI/bge-m3:<revision>`. |
| `payout_formula` | Makes rewards predictable before play starts. | Yes | Should be a canonical formula id/config, not prose. |
| `fixed_reward_pool_base_units` | States how much `$VCTY` is minted for the round. | Yes | Use integer base units, not floats. |
| `pow_algorithm` | Defines how prediction PoW is verified. | Yes | MVP recommendation: `sha256-leading-zero-bits`. |
| `pow_difficulty_bits` | Sets the spam cost for accepted predictions. | Yes | Keep low enough for real players to submit from normal hardware. |
| `ledger_public_key` | Lets players verify signed checkpoints later. | Yes | Add key id, algorithm, and validity window when implemented. |
| `manifest_hash` | Gives a compact commitment to pre-round rules. | Yes, derived | Must exclude itself from the canonical hash payload and should be produced by the manifest constructor, not accepted as input. |

```rust
struct PlayerBinding {
    player_id: String,
    twitter_handle: String,
    twitter_user_id: Option<String>,
    wallet_address: String,
    binding_hash: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `player_id` | Stable internal id used across artifacts and ledger events. | Yes | Should be deterministic or signed so it cannot silently change. |
| `twitter_handle` | Human-readable public identity. | Yes | Handles can change; add Twitter user id as soon as practical. |
| `twitter_user_id` | More durable Twitter identity than handle. | Recommended | Optional only if current API path cannot fetch it quickly. |
| `wallet_address` | Destination identity for `$VCTY` migration. | Yes | Normalize/checksum and bind to chain/network id. |
| `binding_hash` | Commits to the identity binding. | Yes, derived | Hash is integrity, not authority; add Vectory signature and ideally wallet ownership proof. |

Identity rule:

- Twitter user id/handle is the player-facing source identity for the MVP.
- Wallet address is the payment/migration destination, not the sole player identity.
- An accepted prediction must come from the registered Twitter identity and use the wallet bound to that identity unless explicit wallet-rotation rules exist.
- Per-wallet limits alone are not meaningful anti-spam controls because a player can create unlimited native wallets.

```rust
struct PredictionRecord {
    prediction_id: String,
    round_id: u64,
    player_id: String,
    twitter_handle: String,
    wallet_address: String,
    prediction_text: String,
    prediction_hash: String,
    pow_nonce: String,
    pow_digest: String,
    pow_valid: bool,
    tweet_id: String,
    tweet_author_id: Option<String>,
    tweet_created_at: String,
    collected_at: String,
    accepted: bool,
    rejection_reason: Option<String>,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `prediction_id` | Identifies the exact submitted prediction. | Yes | Prefer deterministic id from round id + tweet id + player id. |
| `round_id` | Joins prediction to manifest, artifact, and checkpoint. | Yes | Must match manifest. |
| `player_id` | Links prediction to registered player. | Yes | Derived from manifest participant binding. |
| `twitter_handle` | Human-readable source identity. | Yes | Validate against player binding. |
| `wallet_address` | Migration destination asserted in the prediction. | Yes | Must match player binding unless explicit wallet update rules exist. |
| `prediction_text` | Public text scored against the target. | Yes | Store canonicalized text and raw text hash if parser strips formatting. |
| `prediction_hash` | Compact commitment to the canonical prediction payload. | Yes, derived | Useful for artifacts and future onchain anchoring. |
| `pow_nonce` | Player-generated work nonce. | Yes | Bound to canonical prediction payload. |
| `pow_digest` | Digest that must satisfy manifest difficulty. | Yes | Must be recomputed during verification. |
| `pow_valid` | Whether digest satisfies difficulty. | Yes | Prefer accepted/rejected status plus reason in implementation. |
| `tweet_id` | Public anchor for the prediction. | Yes | Twitter remains public truth. |
| `tweet_author_id` | Durable author identity. | Recommended | Avoid relying only on mutable handles. |
| `tweet_created_at` | Timestamp used for deadline checks. | Yes | Treat as Twitter-provided evidence, not perfect consensus time. |
| `collected_at` | Operational collection timestamp. | Yes | Helps audit ingestion delays. |
| `accepted` | Whether prediction is eligible for scoring/reward. | Yes | Rejected predictions still belong in the artifact. |
| `rejection_reason` | Explains invalid, late, wrong-author, duplicate, or bad-PoW submissions. | Yes | Critical for player trust. |

```rust
struct RoundArtifact {
    round_id: u64,
    manifest_hash: String,
    target_tweet_id: String,
    target_text: String,
    scorer_version: String,
    scorer_metadata_hash: String,
    predictions: Vec<PredictionRecord>,
    scores: Vec<PlayerScore>,
    reward_pool_base_units: String,
    payout_formula: String,
    artifact_hash: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `round_id` | Joins artifact to manifest and checkpoint. | Yes | Must match manifest. |
| `manifest_hash` | Proves artifact was scored under declared rules. | Yes | Verification should fail if this does not match the manifest. |
| `target_tweet_id` | Identifies the public target. | Yes | Add target author id, created timestamp, and raw tweet snapshot hash later. |
| `target_text` | Input to the scorer. | Yes | Also store `target_text_hash` for compact verification. |
| `scorer_version` | Identifies scoring implementation used. | Maybe duplicated | Since `manifest_hash` already commits to scorer version, keep this only as artifact-specific evidence or if an override is explicitly allowed. |
| `scorer_metadata_hash` | Commits to model/tokenizer/runtime/config metadata. | Yes | This is critical while scorer is Vectory-attested. |
| `predictions` | Public prediction records used for eligibility and scoring. | Yes | Must include accepted and rejected records for auditability. |
| `scores` | Player scores and rankings. | Yes | Include similarity, rank, payout inputs, and enough evidence to audit reward. |
| `reward_pool_base_units` | Total `$VCTY` minted for this round. | Yes | Must match manifest unless a cancellation/override rule exists. |
| `payout_formula` | Documents reward calculation actually used. | Maybe duplicated | Since manifest should be authoritative, prefer formula id/config in manifest and include only computed payout evidence here. |
| `artifact_hash` | Compact commitment to all round evidence. | Yes, derived | Must define canonical serialization and exclude itself from hash payload. |

```rust
struct PlayerScore {
    player_id: String,
    prediction_id: String,
    prediction_text_hash: String,
    cosine_similarity: String,
    time_multiplier: String,
    payout_weight: String,
    rank: u32,
    amount_vcty_base_units: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `player_id` | Links score to a player binding. | Yes | Must match prediction record. |
| `prediction_id` | Links score to exact public prediction evidence. | Yes | Allows a player to verify which tweet earned the reward. |
| `prediction_text_hash` | Compact reference to prediction text. | Yes | Avoid relying only on duplicated raw text. |
| `cosine_similarity` | Raw semantic score. | Yes | Use decimal string or deterministic fixed-point representation. |
| `time_multiplier` | Optional early-conviction factor. | Maybe | Include if the payout formula uses timing. Otherwise set to `1`. |
| `payout_weight` | Weight used to divide the reward pool. | Yes | Derived from score formula. |
| `rank` | Human-readable ranking. | Yes | Not sufficient for payout by itself. |
| `amount_vcty_base_units` | `$VCTY` credited by this score. | Yes | Use integer base units. |

```rust
struct VecLedgerEvent {
    event_id: String,
    round_id: u64,
    event_type: String, // "reward_minted"
    to_player_id: String,
    to_wallet_address: String,
    amount_vcty_base_units: String,
    reason: String,
    round_artifact_hash: String,
    prediction_id: Option<String>,
    previous_ledger_event_hash: Option<String>,
    event_hash: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `event_id` | Identifies a specific ledger event. | Yes | Prefer deterministic id to prevent duplicates/replays. |
| `round_id` | Links reward to the round that produced it. | Yes if events are portable | Redundant inside one-round checkpoints; keep if events need to stand alone outside the checkpoint. |
| `event_type` | Distinguishes reward mint from future event types. | Yes | Should become an enum, not free string. |
| `to_player_id` | Links reward to the player binding. | Yes | Include `binding_hash` later so wallet changes are auditable. |
| `to_wallet_address` | States migration destination for the reward. | Yes | Must match current valid binding or record an explicit exception. |
| `amount_vcty_base_units` | Amount of `$VCTY` credited. | Yes | Integer base units avoid float ambiguity. |
| `reason` | Human-readable explanation for the credit. | Useful | Prefer `RewardReason` enum/code in canonical event; keep prose outside the event hash. |
| `round_artifact_hash` | Binds event to scoring evidence. | Yes if events are portable | Redundant inside one-round checkpoints; critical if reward events are later exported independently. |
| `prediction_id` | Links reward to the exact public prediction. | Recommended | Useful for player verification and future audit UI. |
| `previous_ledger_event_hash` | Event-level append-only linkage. | Maybe | Could be removed if checkpoints hash-chain all events; keep only if event-by-event replay matters. |
| `event_hash` | Compact commitment to this event. | Yes, derived | Must define canonical serialization and exclude itself from hash payload. |

```rust
struct BalanceRecord {
    player_id: String,
    wallet_address: String,
    amount_vcty_base_units: String,
    checkpoint_id: String,
    checkpoint_hash: String,
}
```

For the tiny cohort, signed explicit balances are simpler than a Merkle root. A root can be added later when the player set grows.

```rust
struct SignedVecCheckpoint {
    checkpoint_id: String,
    round_id: u64,
    previous_checkpoint_hash: Option<String>,
    round_artifact_hash: String,
    ledger_events: Vec<VecLedgerEvent>,
    resulting_balances: Vec<BalanceRecord>,
    signer_key_id: String,
    signer_public_key: String,
    signature: String,
    checkpoint_hash: String,
}
```

Review notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `checkpoint_id` | Identifies the signed checkpoint artifact. | Yes | Add monotonic index for easier audit. |
| `round_id` | Links one checkpoint to one round. | Yes for MVP | Future checkpoints may cover multiple rounds. |
| `previous_checkpoint_hash` | Creates append-only checkpoint chain. | Yes | Only `None` for the first checkpoint. |
| `round_artifact_hash` | Binds checkpoint to scored round evidence. | Yes | Must equal artifact hash. |
| `ledger_events` | The actual mint/reward events being signed. | Yes | These are the early `$VCTY` source of truth. |
| `resulting_balances` | Commits to post-checkpoint balances. | Yes for MVP | Easier for a human player to inspect than a Merkle root. |
| `signer_key_id` | Identifies the temporary ledger key. | Yes | Verification should trust a keyring, not only the key embedded in checkpoint. |
| `signer_public_key` | Public key material for verification. | Maybe | Keep for portability, but verify against trusted manifest/keyring. |
| `signature` | Temporary Vectory authorization of checkpoint. | Yes | This is the bootstrap authority until protocol minting exists. |
| `checkpoint_hash` | Hash over canonical checkpoint payload. | Yes, derived | Signature should be over this hash, excluding signature/checkpoint self-fields as specified. |

## Player-Called Functions

Players should only need to register, prepare/post predictions, and verify. They should not call mint functions.

### Register Player

```rust
fn register_player(
    twitter_handle: String,
    wallet_address: String,
) -> Result<PlayerBindingReceipt>;
```

API minimality note:

As a player-called function, registration should create a request unless Vectory immediately signs/accepts the binding. A cleaner implementation may split this into `create_registration_request(...) -> RegistrationRequest` and an operator-side `sign_player_binding(...) -> PlayerBinding`. `PlayerBindingReceipt` is redundant unless it adds signature/publication metadata beyond `PlayerBinding`.

Returns:

```rust
struct PlayerBindingReceipt {
    player_id: String,
    twitter_handle: String,
    wallet_address: String,
    binding_hash: String,
}
```

### Prepare Prediction

```rust
fn prepare_prediction(
    manifest: RoundManifest,
    player: PlayerBinding,
    prediction_text: String,
) -> Result<PredictionDraft>;
```

This replaces `prepare_commit` and `prepare_reveal`.

The function must:

- canonicalize the prediction payload
- search for a nonce satisfying `manifest.pow_difficulty_bits`
- derive `prediction_hash`
- derive `pow_digest`
- produce canonical tweet text
- save a local copy for player verification/debugging

Returns:

```rust
struct PredictionDraft {
    round_id: u64,
    player_id: String,
    wallet_address: String,
    prediction_text: String,
    prediction_hash: String,
    pow_nonce: String,
    pow_digest: String,
    tweet_text: String,
}
```

Tweet format:

```text
r:<round_id>
p:<prediction text>
w:<wallet>
n:<pow_nonce>
d:<pow_digest>
```

Parameter notes:

| Parameter | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `manifest` | Binds prediction to round rules, target, deadline, scorer, and PoW difficulty. | Yes | Do not accept loose `round_id`/difficulty parameters in isolation. |
| `player` | Binds prediction to registered handle and wallet. | Yes | Prevents wallet override bugs. |
| `prediction_text` | Public prediction being scored. | Yes | It is not secret. |

Return notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `round_id` | Confirms target round in local saved draft. | Yes | Needed for CLI state. |
| `player_id` | Confirms identity binding. | Yes | Must match manifest participant. |
| `wallet_address` | Shows migration destination. | Yes | Must match binding. |
| `prediction_text` | Text to post publicly and score later. | Yes | Keep canonical and raw forms if necessary. |
| `prediction_hash` | Compact commitment to canonical payload. | Yes | Useful for artifacts and future anchoring. |
| `pow_nonce` | Proof-of-work nonce. | Yes | Bound to this payload only. |
| `pow_digest` | Work digest satisfying difficulty. | Yes | Verifier recomputes it. |
| `tweet_text` | Canonical tweet the player posts. | Yes | Collector should reject freestyle formats for the MVP. |

### Verify Player Reward

```rust
fn verify_player_reward(
    bundle: VerificationBundle,
    player_id: String,
) -> Result<PlayerRewardVerification>;
```

The implementation should take a single `VerificationBundle` containing manifest, artifact, checkpoint chain, trusted keyring, and optional balance records/proofs. A single checkpoint is not enough to prove hash-chain validity.

Returns:

```rust
struct PlayerRewardVerification {
    prediction_included: bool,
    prediction_author_valid: bool,
    prediction_timestamp_valid: bool,
    proof_of_work_valid: bool,
    score_included: bool,
    reward_included: bool,
    checkpoint_signature_valid: bool,
    checkpoint_hash_chain_valid: bool,
    vec_balance_base_units: String,
}
```

Return notes:

| Member | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `prediction_included` | Confirms the player's public prediction is in the artifact. | Yes | Report tweet id and prediction id in real implementation. |
| `prediction_author_valid` | Confirms tweet came from the registered Twitter identity. | Yes | Prefer Twitter user id over handle when available. |
| `prediction_timestamp_valid` | Confirms prediction was posted before deadline. | Yes | Should expose deadline and tweet timestamp. |
| `proof_of_work_valid` | Confirms digest satisfies manifest difficulty. | Yes | Should expose digest and difficulty. |
| `score_included` | Confirms player was scored. | Yes | Should expose score/rank, not just boolean. |
| `reward_included` | Confirms `$VCTY` event exists. | Yes | Should expose event id/hash and amount. |
| `checkpoint_signature_valid` | Confirms checkpoint was signed by Vectory ledger key. | Yes | Requires trusted public key/keyring input. |
| `checkpoint_hash_chain_valid` | Confirms append-only history. | Yes if checkpoint chain is supplied | Cannot be proven from a single checkpoint alone. |
| `vec_balance_base_units` | Shows resulting player balance. | Yes | Use typed base units. |

## Operator-Called Function Inputs

Active implementation todos for these interfaces are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live blocker or task register.

These are Vectory-side during the signed-ledger phase.

### Create Round Manifest

```rust
fn create_round_manifest(
    input: RoundManifestInput,
) -> Result<RoundManifest>;
```

Creates the pre-round rules. This must happen before the first prediction is accepted.

Parameter notes:

| Parameter | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `input` | Carries participants, prediction deadline, 24-hour target expiry rule, scorer, payout, PoW, and ledger key config. | Yes | `RoundManifestInput` must be defined before implementation. It should include canonicalization version and publication plan. |

Return notes:

| Return | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `RoundManifest` | Public rules artifact for the round. | Yes | Should be published or hash-referenced before predictions. |

### Collect Round Events

```rust
fn collect_round_events(
    manifest: RoundManifest,
) -> Result<CollectedRoundEvents>;
```

Collects prediction tweets, player identities, target tweet, and timestamps.

Parameter notes:

| Parameter | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `manifest` | Scopes collection to announcement, participants, target, parser rules, deadline, and PoW difficulty. | Yes | Avoid loose `round_id` collection because it can collect wrong tweets or stale formats. |

Return notes:

| Return | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `CollectedRoundEvents` | Normalized public evidence from Twitter. | Yes | Must preserve tweet ids, author ids, timestamps, raw text hashes, parser version, accepted/rejected status, and reasons. |

### Score Round

```rust
fn score_round(
    manifest: RoundManifest,
    events: CollectedRoundEvents,
) -> Result<RoundArtifact>;
```

Creates the round artifact. For now, the scorer may be Vectory-attested, but the artifact must publish enough metadata to be replayed or challenged later.

Parameter notes:

| Parameter | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `manifest` | Supplies rules, deadline, participants, scorer, payout formula, and PoW difficulty. | Yes | Scoring must reject events outside manifest rules. |
| `events` | Supplies collected predictions and target tweet. | Yes | Must include accepted and rejected records for auditability. |

Return notes:

| Return | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `RoundArtifact` | Canonical evidence bundle for results. | Yes | Should include scorer metadata and artifact hash. |

### Create `$VCTY` Checkpoint

```rust
fn create_vec_checkpoint(
    previous_checkpoint_hash: Option<String>,
    round_artifact: RoundArtifact,
    signing_key_id: String,
) -> Result<SignedVecCheckpoint>;
```

This is the temporary mint authority. Later, this function is replaced by protocol-native minting from finalized blocks.

The function should derive rewards from `round_artifact.scores` and the manifest payout formula. It should not accept arbitrary rewards as ordinary input.

Parameter notes:

| Parameter | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `previous_checkpoint_hash` | Links this checkpoint to prior checkpoint. | Yes after genesis | `None` only for first checkpoint. |
| `round_artifact` | Evidence used to justify rewards. | Yes | Function should derive or validate rewards from this artifact. |
| `signing_key_id` | Selects temporary Vectory ledger key. | Yes | Checkpoint should record key id, public key, and algorithm. |

Return notes:

| Return | Reason | Needed for MVP? | Notes |
| --- | --- | --- | --- |
| `SignedVecCheckpoint` | Signed durable reward/balance artifact. | Yes | Must include signature, hash, ledger events, and explicit balance records. |

## Type Design Inputs

Active implementation todos for these types are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live blocker or task register.

The current signatures mention several placeholder types. They are not optional details; they carry much of the player proof surface.

| Type | Why it exists | Needed for MVP? | Minimum notes |
| --- | --- | --- | --- |
| `PredictionRecord` | Captures each prediction tweet and parser decision. | Yes | Include tweet id, author id/handle, text hash, timestamp, parsed prediction, wallet, nonce, digest, PoW validity, accepted/rejected status, and reason. |
| `PlayerScore` | Captures score/rank/payout inputs. | Yes | Include player id, prediction refs, similarity, timing multiplier if used, payout weight, amount, and score artifact references. |
| `RoundManifestInput` | Operator input for creating pre-round rules. | Yes | Include target, participants, prediction deadline, scorer config, payout config, PoW config, ledger key id, and canonicalization version. |
| `CollectedRoundEvents` | Normalized Twitter evidence before scoring. | Yes | Include target tweet, predictions, fetch metadata, parser version, and rejection records. |
| `BalanceRecord` | Explicit post-checkpoint balance for each player. | Yes for MVP | For the tiny cohort, easier for players to inspect than a balances Merkle root. Include player id, wallet, amount in base units, checkpoint id/hash. |
| `VerificationBundle` | All artifacts needed to verify a player reward. | Recommended | Should contain manifest, artifact, checkpoint chain or prior hashes, trusted keyring, and optional balance records/proofs. |

Removed types:

| Type | Why removed |
| --- | --- |
| `CommitmentRecord` | Hidden commitment tweets are not part of the public-prediction MVP. |
| `RevealRecord` | Reveals are unnecessary because predictions are public from submission. |
| `VecReward` | Checkpoints should derive reward events from the artifact and payout formula instead of accepting arbitrary proposed rewards. |

## Checkpoint Signing

The checkpoint signature signs the canonical checkpoint hash:

```text
checkpoint_hash = HASH(canonical_signed_checkpoint_payload_without_signature)
signature = SIGN(VECTORY_LEDGER_PRIVATE_KEY, checkpoint_hash)
```

Anyone can verify:

```text
VERIFY(VECTORY_LEDGER_PUBLIC_KEY, checkpoint_hash, signature)
```

The Vectory key is temporary. It should authorize early ledger checkpoints only until the decentralized chain imports balances and protocol-native minting exists.

## Migration Promise

Every valid signed checkpoint contributes to the future migration set:

```text
signed checkpoints
-> verified balances
-> migration manifest
-> decentralized chain genesis import
```

After migration, new `$VCTY` should be minted by protocol rules:

```text
finalized block
+ valid public prediction proof-of-work
+ valid scoring artifact
+ passed challenge window
+ deterministic payout formula
= protocol-minted $VCTY
```

## Round 73 Recommendation

Because round 73 has been paused on Twitter for about a week, it is risky to use as the first serious `$VCTY` checkpoint round unless the pause, resume, prediction deadline, and PoW rules are clearly posted and included in the manifest.

Safer path:

```text
Round 73: publish paused/void/resume decision clearly
Round 74: first signed-$VCTY checkpoint round with public predictions, PoW, and the human player included from the start
```

## Minimum Player Promise

The player-facing promise should be:

```text
If your public prediction is posted before the deadline, has valid proof-of-work, is scored in the round artifact, and your reward appears in a signed $VCTY checkpoint, that $VCTY balance will be included in the future migration set.
```

Avoid stronger claims until decentralized finality exists.
