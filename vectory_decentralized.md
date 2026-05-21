# Vectory Decentralized — Design Notes

---

## 2026-04-13 — Canonical Vector & Consensus

### The Canonical Vector

The canonical [prediction + transaction] vector is the one whose:

1. **Vector summation tip** is closest to the source summation vector's tip (the *ground truth sum vector*)

2. **Transactions** follow basic accounting rules — no double spend, etc.

This is valid because vectors are commutative.

3. **Time dimension** — each prediction vector includes a time dimension. Its magnitude is proportional to how early the prediction was made, but its *direction* is a wild card: it always points in the direction of the source vector's tip.

This incentivizes canonical vectors to be distributed to the group faster, to help consensus.

---

### How Far Back

The predicting agent can specify the time their prediction will come through — but *not* how early it will be included in the stream by whoever creates the canonical sum vector.

They are rewarded by how early they get it into the canonical vector.

So to incentivize the creator, they can contract to split whatever their earnings are for that prediction — when it's revealed — with the creator.

So the creator is incentivized to put it in as early as possible.

**Each block in the chain must include:**
- The source vector
- The revealed predictions vector
- Committed predictions that are still hashed
- Transactions

---

## 2026-04-13 (cont.) — The Double Spend Problem

We still have another problem we have to solve — it's basically the same double spending problem that's in Bitcoin.

Assuming that vectors are commutative and that the reveal sum vector tip closest to the source sum vector — hereafter known as the **ground truth sum vector** — is the canonical one.

**The question is:** is it possible for two different equally valid chains to exist at the same time?

So we have to determine what else makes a valid chain. A valid chain must contain:

1. The transactions
2. Predictions that are still in their commitment form
3. The revealed predictions
4. The ground truth vectors

---

## Background — Vectory (Twitter-Native Game, earlier design)

> *From prior sessions — included for context.*

Vectory is a Twitter-native prediction game. Players submit text-based predictions as cryptographic commitments (so no one can see them until after the target account tweets), then reveal their plaintext predictions after the event. Predictions are scored using **BGE-M3** embeddings.

### Reward Formula

$$R_i^{\text{net}} = \underbrace{\frac{e^{\tau \cdot \text{sim}(\bar{e}_i,\ \mathbf{t})}}{\sum_j e^{\tau \cdot \text{sim}(\bar{e}_j,\ \mathbf{t})}}}_{\text{softmax gross reward}} \cdot\ \text{spec}(\bar{e}_i)\ \cdot\ \phi(\Delta t_i)\ -\ \lambda(m_i - 1)^\alpha$$

Where:
- $\tau$ — softmax temperature (sharpness of competition)
- $\text{spec}(\bar{e}_i) = \text{sim}(e, \mu_{\text{corpus}})^{-\beta}$ — specificity multiplier (penalizes vague embeddings)
- $\phi(\Delta t_i) = \left(\frac{\Delta t_i}{\Delta t_{\max}}\right)^\gamma$ — temporal reward (earlier = more)
- $\lambda(m_i - 1)^\alpha$ — superlinear payload cost (penalizes hedging)

### System Parameters

| Parameter | Effect | Starting value |
|-----------|--------|----------------|
| τ | Softmax temperature | 5–10 |
| λ | Payload cost scale | Tune vs gross reward |
| α | Cost superlinearity exponent | 2 |
| β | Specificity penalty exponent | Tune vs corpus |
| γ | Time reward shape | 1 (linear) |

### Three Forces on Agents

| Force | Mechanism | Pressure |
|-------|-----------|----------|
| Precision | Softmax + specificity | Be close to target with a pointed embedding |
| Conciseness | Superlinear cost | Minimize embeddings sent |
| Timing | φ(Δt) | Submit as early as you can justify |

**Dominant rational strategy:** resolve uncertainty internally, commit to one precise embedding, submit early.


## 2026-04-30 — Problems with the Above Summation Vector
### It's not adequate to solve the double spending problem 
The problem with the above is that making the canonical vector the summation that whose tip is closest to the source isn't defined enough to solve the double spending problem. It doesn't even have anything to do with it.


You need to make it so that with each new block, it becomes computationally infeasible for an equally valid chain, being just as long with equally valid vectors, to be generated. So that agents can agree

### First Why go through all this trouble to make it decentralized: NO RESTRICTIONS ON THE TYPES OF OPTIONS

Because centralization will really limit what we can do cause we'll be subject to U.S. gambling laws. Decentralization means we have the freedom to create different types of options 

### Solution

#### Changes to Commits
1. The only vector representation that get put into the block are hashes of the vectors, i.e. the commits
2. The vectors themselves are stored off-chain in IPFS. Not floating point so they remain the same across platforms

3. To be precise, what gets put into the block is not the commit but a proof of work of the commit. Such that we make each agent making a prediction, to first generate the commitment and then create a proof of work of that commitment, e.g. they need to hash it again with a nonce that makes the output hash have some amount of leading zeros. And what they submit is the commit with the nonce. Such that when you hash that commit with the nonce, you get a hash with N leading zeros. That's what gets submitted.

4. This is even in the case of predictions that are submitted through twitter. The only thing different is that Twitter users provide the reveal, they'll still provide the plain text, but th
e hash 


#### Market Decides Twitter Accounts to Predict
This proof of work allows each indvidual agent to decide which account they want to target. Since they're paying a cost. Which also prevents them from flooding the network with arbitrary requests. It forces them to be strategic cause it costs them. 

We can have some rule like the account can't have fewer than 10,000 followers or something. Those who construct the block can decide to include them or not.

#### IPFS
What gets stored in IPFS
1. Revealed prediction vector and the target it points to. It's address will be the commit+nonce 
2. Target: Each Ground Truth data point (Twitter URLS and Text) + their vectors. Addressed by its target


#### Each Block Will Be Composed of 
1. Commits+Nonces
2. Transactions

#### Ordering
There's only one way to order the commits+nonces. Take the hexidecimal encoding and sort them alphanumerically. Using a Merkle Tree (What bitcoin uses), A Merkle Patricia Trie (what Ethereum uses) or whatever the best one is

#### Payment

It'd be too much to pay everyone who makes predictions for a particular account since we're letting anyone make predictions for any account. So for any particular twitter account, it'll only be the winner who gets paid for that account. This reduction in inclusivity is offset by allowing anyone to decide what accounts they want to make predictions for.

Or is it too much, maybe we can pay everyone. They'll all be included in the Merkle Tree no, and the main goal is getting as many tweets as possible.




