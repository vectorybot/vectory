# Vectory Thesis

## Purpose of this file

This file is the first-read handoff for any AI agent or engineer working on **Vectory**.

It captures the current product framing, architectural decisions, reasoning, constraints, and next implementation steps so a new agent can pick up where prior discussions left off without re-deriving core decisions.

Do not treat this as generic background context. Treat it as **active product and architecture guidance**.

---

# 1. Project identity

## What Vectory is

**Vectory** is the renamed evolution of **Cliptions**.

It is a **Twitter-native semantic prediction game** in which players try to predict future public content and compete for crypto rewards based on semantic similarity scoring.

The project originally used CLIP and future video frames as the target. The current direction is moving toward a more Twitter-native design where users predict future tweets or similar public text outcomes, scored with a text embedding model such as **bge-m3** rather than CLIP.

At a high level:

1. The system announces a round on Twitter/X.
2. Players publicly reply with a **commitment hash** before the deadline.
3. Later, they publicly reveal the plaintext prediction and salt.
4. The revealed prediction is scored against the target content using embeddings.
5. Winners are ranked and paid from the prize pool.
6. Results are publicly visible, creating spectacle and credibility.

## What Vectory is not

Vectory is **not** just a backend scoring service.
Vectory is **not** just an internal agent system.
Vectory is **not** just a semantic search demo.
Vectory is **not** a hidden workflow that happens off-platform.

The public nature of the game is part of the product itself.

---

# 2. Product goal and hypothesis

## Immediate goal

The immediate goal is to test whether people will actually play this game in public.

This means the system should optimize for:

- fast iteration
- tight feedback loops
- measurable user engagement
- public participation
- visible outcomes
- replayable and understandable rounds
- minimal infrastructure and operational risk where possible

## Core product hypotheses

Vectory is testing several hypotheses at once:

1. **People will publicly play a semantic prediction game on Twitter/X.**
2. **Public commitment/reveal mechanics can create trust and suspense.**
3. **Embedding-based scoring can create a legible, objective-seeming ranking system.**
4. **Crypto rewards can increase participation and retention.**
5. **Twitter-native public play creates built-in virality and social proof.**
6. **Agents can become visibly better at predicting future content over time, and that itself is compelling.**

## Secondary strategic angle

Over time, Vectory may become a public proving ground where humans and agents demonstrate predictive skill in a very visible way.

That is part of the long-term appeal:
- prediction ability becomes legible
- public performance becomes a reputation signal
- agents can improve and show that improvement in public

---

# 3. Non-negotiable product principle

> **Vectory is Twitter-native by design.**
> Twitter is where the game happens publicly.
> Internal durable state exists to verify, orchestrate, and settle the game - not to replace the public ritual.

This is the most important architectural principle from prior discussions.

Any implementation that undermines this should be viewed with suspicion.

---

# 4. Architecture decision from prior discussion

We considered whether Vectory should adopt a **Durable Sessions / durable streams** architecture similar to ElectricSQL's collaborative AI session model.

## Conclusion

Do **not** replace Twitter/X with an internal durable-session-first interaction model.

Instead:

- **Twitter/X should remain the public record, public interface, and public ritual**
- an internal event log may still exist
- but the internal event log should **support Twitter-native gameplay**, not replace it

## The chosen split

### Twitter/X is for:
- public round announcements
- public commitment posts
- public reveals
- public winner announcements
- public leaderboard/payout visibility
- marketing
- virality
- social proof
- public evidence that the round happened

### Internal durable state / event log is for:
- orchestration
- timing control
- retries
- worker coordination
- ingestion and normalization of tweet-derived events
- reveal verification
- scoring
- payout workflow
- audit reconstruction
- operational sanity

This is the central architecture decision currently in force.

---

# 5. Reasoning behind the architecture decision

## Why Twitter must remain central

Twitter is not just an output channel. It is part of the game design.

It provides:
- visibility
- credibility
- built-in distribution
- player-vs-player public participation
- public proof of participation
- spectacle
- a natural place for agents and humans to interact in front of others

If commitments and reveals happen publicly, the game becomes a social object rather than a hidden backend process.

That is valuable.

## Why Twitter alone is not enough

Twitter is not a reliable transaction engine or workflow engine.

Problems with using Twitter alone as the only system of record:

- API failures
- rate limits
- delayed ingestion
- brittle parsing of replies/threads
- unclear ordering semantics
- deletions/edits/edge cases
- poor support for retries and recovery
- difficult internal coordination between workers/agents
- awkward reconstruction of state after crashes

Therefore:
- Twitter is excellent as **public truth**
- Twitter is weak as the only **operational truth**

## Therefore the correct split is:

- **Twitter = public state expression**
- **Internal append-only event log = operational state execution**

That preserves virality while avoiding brittle backend logic.

---

# 6. Source-of-truth policy

This distinction is important.

## Public truth

Twitter/X is the canonical source of truth for what happened **publicly**.

Examples:
- whether a commitment tweet exists
- whether a reveal tweet exists
- what the public saw
- what was announced and when, according to the platform

## Operational truth

The internal event log is the canonical source of truth for what the system believes happened operationally and what step comes next.

Examples:
- commitment observed
- commitment validated
- reveal accepted/rejected
- score computed
- payout requested
- payout sent
- round closed

## Projection truth

Database projections or cached views are not the source of truth.
They are rebuildable from internal events.

## Policy implications

- workers should be **idempotent**
- projections should be **rebuildable**
- payouts must only happen from **verified internal state**
- do not execute critical state transitions from raw tweet parsing alone
- always normalize public events into internal machine-readable events

---

# 7. Recommended system shape

The current recommended architecture is:

## Layer 1: Twitter/X public interface
This is where the game is visible.

Responsibilities:
- posting round announcements
- receiving commitments via replies
- receiving reveals via replies
- posting winner announcements
- posting leaderboard/payout summaries

## Layer 2: Ingestion and normalization
A service observes relevant tweets/replies and converts them into machine-readable internal events.

Responsibilities:
- monitor round tweet threads
- fetch replies and metadata
- parse commitments/reveals
- identify relevant player messages
- normalize tweet activity into domain events

## Layer 3: Internal append-only event log
This stores normalized domain events and drives the state machine.

Responsibilities:
- canonical operational history
- retries
- audit trail
- deterministic replay
- worker subscriptions

## Layer 4: Projection database
This stores current derived state for queries and UI.

Examples:
- round status
- commitments
- reveals
- verified reveals
- scores
- payouts
- leaderboards

## Layer 5: Workers / agents
Independent workers consume events and perform actions.

Examples:
- round manager
- commitment validator
- reveal verifier
- scorer
- payout executor
- social announcer
- dispute/audit worker

---

# 8. Event model

The exact names may change, but the system should conceptually support events like the following.

## Round lifecycle events
- `round_created`
- `round_announced`
- `round_opened`
- `target_locked`
- `round_closed`

## Commitment events
- `commitment_observed`
- `commitment_parsed`
- `commitment_validated`
- `commitment_rejected`
- `commitment_accepted`

## Reveal events
- `reveal_observed`
- `reveal_parsed`
- `reveal_verified`
- `reveal_rejected`

## Scoring events
- `score_requested`
- `score_computed`
- `winner_declared`
- `leaderboard_updated`

## Payment events
- `payout_requested`
- `payout_prepared`
- `payout_sent`
- `payout_failed`

## Audit / moderation events
- `round_disputed`
- `round_reconstructed`
- `manual_override_applied`

## Social events
- `announcement_posted`
- `winner_posted`
- `leaderboard_posted`

---

# 9. Example round lifecycle

A typical round should conceptually look like this:

1. round is created internally
2. public round announcement is posted on Twitter
3. users reply with commitment hashes
4. ingestion worker observes commitment tweets
5. commitments are parsed and validated internally
6. target event arrives or target content is locked
7. reveal window opens
8. users reply with plaintext reveal + salt
9. ingestion worker observes reveal tweets
10. reveals are verified against commitments
11. scoring worker computes embedding similarity
12. ranking is finalized
13. payout worker executes payments
14. social agent posts winners and payouts
15. round is closed
16. audit trail remains reconstructable

---

# 10. Implementation boundary: Twitter vs internal system

## Put these on Twitter
- round announcement
- public rules summary
- commitment reply flow
- reveal reply flow
- winner announcement
- top players and payouts
- visible proof that the round occurred

## Put these internally
- machine-readable parsing
- strict validation results
- replayable event history
- state transitions
- score computation details
- wallet payout execution
- worker retries
- audit reconstruction
- status projections

## Important note

Twitter is the **venue**.
The internal system is the **referee and settlement engine**.

Do not confuse the two.

---

# 11. Constraints and anti-goals

The coding agent must preserve the following constraints.

## Constraint: preserve Twitter-native gameplay
Do not move the core loop off Twitter unless explicitly directed.

## Constraint: preserve public commitment/reveal dynamics
Public participation is part of the product value.

## Constraint: keep implementation aligned with fast experimentation
The product is still in a hypothesis-testing phase.

## Constraint: support auditability
Critical steps should be reconstructable after the fact.

## Constraint: support deterministic settlement
Scoring and payouts must happen from verified internal state.

## Anti-goal: do not overengineer into a generic multi-agent platform
Vectory is a specific product, not a general-purpose research framework.

## Anti-goal: do not hide the game behind a private backend-only UX
That would destroy much of the marketing and social value.

## Anti-goal: do not rely on Twitter alone for correctness
Twitter should not be the sole operational engine.

## Anti-goal: do not "improve away" the public ritual
The ritual is part of the product.

---

# 12. Model and scoring direction

## Historical design
The project originally used **CLIP** scoring against future video frames.

## Current direction
The project is moving toward text-native prediction targets and away from CLIP for the core Vectory loop.

The leading direction discussed is:
- use a text embedding model such as **bge-m3**
- compare revealed prediction text to target text semantically
- preserve commitment/reveal integrity
- avoid unnecessary prompt wrappers or prefixes in the actual scored text unless justified by testing

## Current practical stance
The coding agent should assume:
- CLIP-based code may exist historically
- Vectory's newer direction is text-native embedding scoring
- existing CLIP-specific assumptions may need to be removed or abstracted behind interfaces

---

# 13. Existing codebase assumptions

The project already has or had some relevant components in Rust, including versions of:
- commitment hash generation
- some Twitter API access
- embedding/scoring logic
- payout-related logic

However:
- some of that code is tied to CLIP-era assumptions
- the embedding model and scoring pipeline are expected to change
- the architecture should be updated to reflect the Twitter-native Vectory direction

The coding agent should prefer **clean interfaces** over hardwiring old assumptions.

---

# 14. Public UX and fairness assumptions

The following fairness assumptions are important to preserve:

- submissions after the deadline are disqualified
- commitment tweets must be public and attributable
- reveal must match the prior commitment
- edited commitment tweets are disqualified
- paid entry or wallet-linked mechanics may exist and must be handled clearly
- payout decisions should be explainable and auditable
- rules should be simple enough for public users to understand

---

# 15. Why an internal event log still matters

Even though Twitter is public state, an internal event log is still recommended because it gives:

- simpler retries
- cleaner state machine transitions
- easier replay after failure
- safer payout handling
- better debugging
- normalized data for analytics
- easier dispute handling
- support for multiple internal workers/agents

This is not a contradiction with the Twitter-native design.
It is the internal machinery that makes the Twitter-native design operationally viable.

---

# 16. Open questions

These items are still open and should not be silently assumed.

## Product / mechanics
- exact target format for each round
- exact reveal window mechanics
- exact payout structure and treasury flow
- whether some milestones should always be mirrored publicly or only selected ones

## Technical
- exact event log technology
- exact projection database design
- exact ingestion mechanism for Twitter/X
- exact retry semantics
- exact wallet/payment rail
- exact scoring model and infrastructure path
- exact dispute-resolution workflow

## Compliance / framing
- how the system should be described publicly to avoid unnecessary regulatory confusion
- what language should be used around contest, market, prediction, prize pool, and settlement

---

# 17. Preferred implementation style

The coding agent should bias toward:

- explicit state machines
- clean event types
- deterministic transitions
- idempotent workers
- modular boundaries
- interfaces around external dependencies
- rebuildable projections
- auditability over magical hidden logic
- simple and testable data flows

The system should feel more like:
- an event-sourced round engine with social IO

and less like:
- an improvised collection of scripts scraping Twitter and mutating random DB rows

---

# 18. Immediate implementation guidance

If starting fresh or resuming active implementation, the next useful steps are:

## Step 1: define the round state machine
Document the legal states and transitions for a round.

Example states:
- draft
- announced
- commitments_open
- target_locked
- reveals_open
- scoring
- payouts_pending
- closed
- disputed

## Step 2: define normalized domain events
Create a typed schema for the events listed earlier.

## Step 3: define Twitter ingestion boundaries
Specify:
- how round threads are identified
- how replies are fetched
- how relevant tweets are parsed
- how tweet metadata maps to domain entities

## Step 4: define projection models
At minimum:
- rounds
- commitments
- reveals
- scores
- payouts
- social posts

## Step 5: define worker interfaces
At minimum:
- `RoundManager`
- `CommitmentValidator`
- `RevealVerifier`
- `ScoringEngine`
- `PayoutExecutor`
- `SocialAnnouncer`

## Step 6: abstract scoring model
Do not bind the whole system directly to CLIP-era assumptions.

## Step 7: keep Twitter-native gameplay intact
When in doubt, preserve the public game loop.

---

# 19. Guidance for future agents

Before changing the architecture, ask:

1. Does this preserve Twitter as the public venue of the game?
2. Does this preserve public commitment and reveal as part of the ritual?
3. Does this make settlement safer and more deterministic?
4. Does this improve iteration speed rather than slow it down?
5. Does this accidentally turn Vectory into a private backend product instead of a public social contest?

If the answer to #1 or #2 is "no", the change is probably directionally wrong unless explicitly approved.

---

# 20. One-sentence summary

**Vectory is a Twitter-native semantic prediction contest where public social play happens on Twitter, while an internal event-driven system verifies, scores, and settles the game reliably behind the scenes.**
