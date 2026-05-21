````
# Vectory Semantic Digest Spec — Draft v0.1

## Context

Vectory is a Twitter-native semantic prediction game.

Players try to predict the content/meaning of a future tweet from a target account. They submit a commitment before the target tweet is known, then later reveal their prediction text and salt. The system scores revealed predictions by comparing their semantic similarity to the actual target tweet.

The current core scoring idea is:

prediction text -> embedding vector  
target tweet text -> embedding vector  
cosine similarity(prediction, target) -> ranking/payout

The determinism problem is that neural-network embeddings are not naturally crypto-native. Floating-point outputs can vary slightly across runtimes, hardware, tokenizer versions, model versions, and inference libraries. For a game with rewards, the scoring process needs to be auditable, reproducible, and eventually validator-friendly.

This spec introduces the concept of a **semantic digest**: a fixed, versioned, quantized representation of text meaning.

---

## Core Concept: Semantic Digest

A semantic digest is a compact, deterministic-ish fingerprint of text meaning.

Cryptographic hash digest:

```text
text -> SHA-256 -> fixed byte digest
````

Semantic digest:

```text
text -> canonical embedding model -> float vector -> quantized integer vector -> digest hash
```

Example:

```text
"AI model announcement"
-> [0.013928174, -0.20441893, 0.88201912, ...]
-> [139, -2044, 8820, ...]
-> SHA-256(serialized_quantized_vector)
```

The quantized vector is the semantic digest. The digest hash is a compact commitment to that vector.

Unlike SHA-256, a semantic digest is not meant to prove exact textual equality. It is meant to locate a piece of text in meaning-space so that semantically similar texts produce nearby vectors.

---

## Goals

1. Make Vectory scoring reproducible enough for public verification.
2. Treat embeddings as first-class game objects, not hidden internal artifacts.
3. Reduce floating-point fragility by quantizing vectors.
4. Version the entire scoring pipeline.
5. Support future validator/challenger verification.
6. Keep the immediate implementation simple enough to ship.

---

## Non-Goals

1. Do not solve perfect cross-platform neural-network determinism in v0.1.
2. Do not build a fully decentralized verifier yet.
3. Do not require zero-knowledge proofs yet.
4. Do not put full vectors on Twitter if they are too large.
5. Do not redesign the whole game loop.

---

## Current Game Flow

1. Vectory announces a round on Twitter.
2. Players reply with a commitment hash and wallet address before the deadline.
3. After the target tweet/event is known, players reveal:

   * prediction text
   * salt
   * wallet address, if needed
4. System verifies:

```text
SHA-256(prediction_text || salt || wallet_address || round_id) == submitted_commitment
```

5. System computes semantic digest for:

   * actual target tweet
   * each revealed prediction
6. System computes cosine similarity between each prediction digest and target digest.
7. Players are ranked.
8. Prize/payout is distributed based on rank.

---

## Proposed Semantic Digest Pipeline

```text
input_text
-> canonical text normalization
-> canonical tokenizer
-> fixed embedding model
-> float embedding vector
-> vector normalization
-> quantization
-> canonical serialization
-> digest hash
```

### 1. Text Normalization

Normalize input text before embedding.

Suggested v0.1:

```text
- Unicode normalize to NFC
- Trim leading/trailing whitespace
- Collapse repeated internal whitespace to single spaces
- Preserve case for now
- Preserve punctuation for now
- Preserve emoji for now
```

Reasoning:

Text normalization avoids accidental differences from whitespace, Unicode forms, and copy/paste artifacts. Do not over-normalize yet because case, punctuation, and emoji can carry semantic meaning on Twitter.

---

### 2. Canonical Model

Use one fixed open-source embedding model.

Example candidates:

```text
- bge-m3
- bge-small-en-v1.5
- e5-large-v2
- another MTEB-ranked text embedding model
```

Each scorer version must specify:

```json
{
  "model_name": "...",
  "model_source": "...",
  "model_hash": "...",
  "tokenizer_hash": "...",
  "runtime": "...",
  "scorer_version": "vectory-scorer-v0.1"
}
```

Reasoning:

The model is part of the game rules. Changing the model changes the game. Every scoring result must be tied to a specific model and tokenizer version.

---

### 3. Canonical Runtime

For v0.1, use a locked Docker image as the canonical scorer.

Example:

```text
vectory-scorer:v0.1
- OS image pinned
- Python/Rust dependencies pinned
- embedding model pinned
- tokenizer pinned
- CPU inference preferred for reference scoring
- GPU inference allowed only for non-canonical previews
```

Reasoning:

The Docker image becomes the reference implementation. Validators and challengers can rerun the same image and compare results. CPU inference is slower but usually easier to make reproducible than GPU inference.

---

### 4. Vector Normalization

Normalize the float embedding vector before quantization.

Typical embedding models often already output normalized vectors, but the scorer should make this explicit.

```text
v_normalized = v / ||v||
```

Reasoning:

Cosine similarity depends on angle, not magnitude. Normalizing before quantization makes scoring more stable and easier to reason about.

---

### 5. Quantization

Convert float vector values into integers.

Suggested v0.1 rule:

```text
scale = 10_000
q_i = round(v_i * scale)
```

Example:

```text
0.013928174 * 10_000 = 139.28174 -> 139
-0.20441893 * 10_000 = -2044.1893 -> -2044
```

Output:

```text
float vector: [0.013928174, -0.20441893, 0.88201912, ...]
quantized vector: [139, -2044, 8820, ...]
```

Reasoning:

Floating-point outputs can vary slightly across machines. Quantization collapses tiny differences into identical or near-identical integer vectors. This does not solve large runtime/model differences, but it reduces noise and makes the digest easier to serialize, hash, compare, and verify.

Open design question:

```text
Should scale be 1_000, 10_000, 100_000, int8, int16, or another scheme?
```

Tradeoff:

```text
lower precision = more stable, less semantically precise
higher precision = more precise, less tolerant of runtime noise
```

---

### 6. Canonical Serialization

Serialize the quantized vector in exactly one way before hashing.

Suggested v0.1:

```json
{
  "scorer_version": "vectory-scorer-v0.1",
  "model_hash": "...",
  "normalization": "nfc_trim_collapse_whitespace",
  "quantization": {
    "method": "round_nearest",
    "scale": 10000
  },
  "dims": 1024,
  "vector": [139, -2044, 8820, "..."]
}
```

Then:

```text
digest_hash = SHA-256(canonical_json_bytes)
```

Reasoning:

Serialization bugs are a common source of nondeterminism. Use canonical JSON or a binary format with strict ordering and no ambiguity.

Possible better long-term options:

```text
- canonical JSON
- CBOR canonical encoding
- MessagePack with strict ordering
- fixed-width little-endian binary encoding
```

---

## Scoring

Use quantized semantic digests for scoring.

Suggested v0.1:

```text
score = cosine_similarity(q_prediction, q_target)
```

Implementation can compute cosine over integer vectors using deterministic arithmetic.

Possible formula:

```text
dot(qp, qt) / (sqrt(dot(qp, qp)) * sqrt(dot(qt, qt)))
```

For ranking, exact floating result may not be necessary. The system can rank by a fixed-point approximation.

Reasoning:

The score should be derived from the published or reproducible digest, not from hidden float vectors. This makes the scoring artifact inspectable and auditable.

---

## Published Round Artifact

For every completed round, publish or archive a machine-readable scoring artifact.

Example:

```json
{
  "round_id": "vectory-2026-05-04-001",
  "target_account": "@example",
  "target_tweet_url": "https://x.com/example/status/...",
  "target_text": "...",
  "scorer_version": "vectory-scorer-v0.1",
  "model_hash": "...",
  "tokenizer_hash": "...",
  "text_normalization": "nfc_trim_collapse_whitespace",
  "quantization": {
    "scale": 10000,
    "rounding": "round_nearest"
  },
  "target_digest_hash": "...",
  "target_digest": [139, -2044, 8820],
  "reveals": [
    {
      "player": "@player1",
      "wallet": "0x...",
      "prediction_text": "...",
      "salt": "...",
      "commitment_hash": "...",
      "commitment_tweet_url": "...",
      "reveal_tweet_url": "...",
      "prediction_digest_hash": "...",
      "prediction_digest": [141, -2039, 8799],
      "score": 0.8731,
      "rank": 1,
      "payout": "..."
    }
  ]
}
```

Reasoning:

This artifact gives future agents, validators, and players enough information to reproduce and audit the round.

---

## Twitter Constraints

Full semantic digest vectors may be too large to post directly on Twitter.

Therefore, Twitter should carry compact references:

```text
Round complete.
Target digest hash: 0xabc...
Scorer: vectory-scorer-v0.1
Results: <URL/IPFS/Arweave/GitHub artifact>
```

Reasoning:

Twitter is best used for public ordering, virality, commitments, reveals, and result announcements. The full scoring artifact can live elsewhere.

---

## Verification Model

### v0.1: Centralized but Auditable

Vectory runs the canonical scorer and publishes the artifact.

Anyone can rerun:

```text
docker run vectory-scorer:v0.1 score round_artifact.json
```

Expected output:

```text
- same normalized input text
- same or near-same quantized digests
- same scores
- same rankings
- same payouts
```

### v0.2: Challenger Verification

A challenger can dispute a result by submitting:

```text
round_id
input text
expected digest
recomputed digest
difference proof/log
```

Rules define a tolerance threshold.

Example:

```text
A digest is valid if at least 99.9% of dimensions match exactly
or if cosine(q_claimed, q_recomputed) >= 0.99999
```

### v1.0: Validator Network

Multiple validators run the canonical scorer and sign scoring artifacts.

Future options:

```text
- validator quorum
- bonded validators
- slashing for bad scoring
- random audit sampling
- on-chain digest hashes
- IPFS/Arweave artifact storage
```

---

## Design Decisions and Reasons

### Decision: Use semantic digest as a first-class object

Reason:

This makes Vectory feel more crypto-native. The thing being predicted is not merely “a tweet”; it is a future location in semantic space.

---

### Decision: Quantize vectors

Reason:

Raw floats are fragile. Quantized integer vectors are easier to compare, hash, store, serialize, and verify.

---

### Decision: Keep full neural inference off-chain for now

Reason:

Embedding inference is too expensive and awkward to verify directly on-chain. The immediate goal is public reproducibility, not full trustlessness.

---

### Decision: Use Dockerized canonical scorer

Reason:

This is the fastest practical path to reproducible scoring. It avoids getting stuck on the harder problem of perfect cross-platform neural-network determinism.

---

### Decision: Publish scorer version and model hashes

Reason:

Changing the model/tokenizer/scoring logic changes the game. Every round must be tied to a clear scoring version.

---

### Decision: Use Twitter for commitments/reveals, not full scoring data

Reason:

Twitter gives distribution, social proof, timestamps, and viral loops. It is not a good place to store large vectors or complete scoring artifacts.

---

### Decision: Start with tolerance-based verification

Reason:

Perfect determinism may not be possible immediately. Tolerance-based verification allows practical progress while still detecting meaningful manipulation.

---

## Historical Question Inputs

Active open questions are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live question register.

1. Which embedding model should be canonical for v0.1?
2. Should the scorer use bge-m3 or a smaller/faster model first?
3. What quantization scale gives the best stability/accuracy tradeoff?
4. Should vectors be int8, int16, int32, or scaled integers?
5. Should scoring use quantized vectors only, or floats for canonical score plus quantized vectors for audit?
6. What exact text normalization rules should apply to tweets?
7. How should URLs, mentions, hashtags, emojis, and quote tweets be handled?
8. Where should full artifacts live: GitHub, S3, IPFS, Arweave, or database?
9. What is the dispute/challenge process?
10. How much nondeterminism is acceptable before a score is invalid?
11. Should the target tweet text include only visible text, or also media captions/alt text/context?
12. Should deleted or edited tweets invalidate a round?
13. Should the semantic digest be public before reveal, after reveal, or only in final artifact?
14. Should players ever submit predicted semantic digests directly instead of prediction text?

---

## Historical v0.1 Implementation Task Inputs

Active implementation todos are tracked in `PROJECT_STATUS.org`. This section is retained as design-source context only; do not update it as a live task register.

1. Create `vectory-scorer` package.
2. Add canonical text normalization function.
3. Add embedding model wrapper.
4. Add vector normalization.
5. Add quantization function.
6. Add canonical serialization.
7. Add digest hash function.
8. Add cosine scoring over quantized vectors.
9. Add round artifact schema.
10. Add CLI command:

```bash
vectory-scorer score-round round.json > scored_round.json
```

11. Add tests for:

* text normalization
* quantization
* serialization stability
* digest hash stability
* cosine scoring
* full round scoring replay

12. Add golden test fixtures:

```text
input text -> normalized text -> quantized digest -> digest hash
```

---

## Best One-Sentence Summary

A semantic digest is a versioned, quantized embedding vector that turns a piece of text into a reproducible fingerprint of meaning, allowing Vectory to score predictions in a way that is more transparent, auditable, and eventually validator-friendly.

```
```
