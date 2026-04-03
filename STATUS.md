# Vectory Status

## Current State
- Player CLI (`bin/vectory/`) operational — can commit, reveal, verify, and post tweets
- Shared crates (`types`, `twitter-api`) in place
- Camoufox-cli installed for browser-based posting
- Round 39 completed end-to-end using this repo for player actions

## What Works
- `vectory rounds` — check active rounds from validator
- `vectory commit` — generate hash, save prediction locally, post commitment tweet
- `vectory reveal` — load saved prediction, post reveal tweet
- `vectory verify` — verify round scoring (commitment hashes, cosine similarity, softmax)
- `vectory results` — fetch round results from Supabase
- `vectory show` — display saved prediction for a round
- `vectory hash` — compute commitment hash offline
- `vectory tweet/quote/reply` — direct Twitter posting
- Quote-to-reply fallback when API quote tweets are blocked (announcement must @mention players)

## What Doesn't Work
- `vectory verify` only checks scoring math from stored embeddings — it does not independently recompute embeddings from text. A player must trust that the stored embeddings are correct.
- No automated round detection — player must manually provide tweet IDs
- No tests

## Next Steps
- Add deep verification mode to `vectory verify`: player provides their own HuggingFace API key, embeds the prediction and target text independently, and compares against stored embeddings
- Add README with full onboarding instructions
- Add automated round detection (parse validator tweets for round IDs and tweet IDs)
- Add tests for commitment hashing, scoring math, and config loading
