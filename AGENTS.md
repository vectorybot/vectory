# Repository Guidelines

## System Status
- **Read `PROJECT_STATUS.org` first** — it tracks what the system can and cannot do.
- Update `PROJECT_STATUS.org` whenever capabilities change (new tools, features, integrations, or known gaps).
- Keep `AGENTS.md` lean. Live system state belongs in `PROJECT_STATUS.org`, not here.

## Task Workflow
1. Start every task by reading `PROJECT_STATUS.org`, then checking `git status --short` and `git log --oneline -5`.
2. Route yourself to the right supporting docs before editing:
   - `THESIS.md` for the central product thesis and architecture guardrails; use it when evaluating directional changes so implementation stays focused on testing the Twitter-native public-play hypothesis
   - `README.md` for player-facing workflow and onboarding documentation
3. Borrow reusable workflow patterns from other projects, but do **not** copy their project-specific status, stakeholders, or dated context into this repo.
4. End each task by running the narrowest meaningful verification, updating the docs touched by the change, and explicitly noting any remaining risk or unverified path. If that risk or gap remains active, add or update a `PROJECT_STATUS.org` item.
5. **Whenever the player-facing CLI surface changes — a command added, removed, renamed, hidden, or its flags/behavior changed — update `README.md` in the same change.** The `## CLI Commands` tables and the Quick Start examples must always match the actual `vectory --help` output. Run `cargo run -p vectory -- --agent x --help` and reconcile before considering the task done.

## Project Structure & Module Organization
- `bin/vectory/` is the player CLI binary crate; entry point is `bin/vectory/src/main.rs`.
- `crates/types/` defines shared types used across crates (round types, commitment hashing, scoring math).
- `crates/twitter-api/` is the OAuth 1.0a Twitter client.
- `crates/player/` contains the player binary and agent logic.
- `README.md` and `PROJECT_STATUS.org` are the primary documentation set.
- `target/` is Cargo build output; do not edit or commit.

## Build, Test, and Development Commands
- `cargo build` — build the workspace in debug mode.
- `cargo run -p vectory -- <args>` — run the player CLI binary from `bin/vectory`.
- `cargo test` — run all tests.
- `cargo fmt` — format Rust code with rustfmt.
- `cargo clippy --workspace --all-targets` — lint all crates and targets.

## Coding Style & Naming Conventions
- Rust 2024 edition with `rust-version = "1.93"` set at the workspace root.
- Standard Rust formatting (4-space indentation via rustfmt).
- Naming: `UpperCamelCase` for types, `snake_case` for functions/modules, `SCREAMING_SNAKE_CASE` for constants.
- Prefer `Result<T, E>` with `?` for error propagation; use `thiserror`/`eyre` consistently.
- For important design decisions, add concise code comments that explain **why** the approach was chosen (not just what the code does).

## Testing Guidelines
- `cargo test` should stay green before pushing changes.
- Prefer the smallest check that proves the change: `cargo test`, `cargo check -p <crate>`, or `cargo run --example <name>`.
- Keep examples runnable with `cargo run --example <name>`.
- If behavior changes without automated coverage, document the gap and why. If the gap remains active, track it in `PROJECT_STATUS.org`.

## Commit & Pull Request Guidelines
- Follow conventional-style commit messages: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- For PRs, include: summary, testing notes (commands run), and any relevant logs or screenshots.

## Releases
- The release workflow fires on `v*.*.*` tag pushes.
- **Always bump `workspace.package.version` in `Cargo.toml` to match the tag in the same commit, before tagging.** v0.1.0 and v0.1.1 were both tagged without bumping it, so those binaries reported `0.1.0` in Cargo metadata regardless of their tag. Don't repeat that.
- Order: edit `Cargo.toml` → `cargo build` (confirms `Cargo.lock` updates cleanly) → commit `chore(release): bump workspace version to vX.Y.Z` → push master → `git tag vX.Y.Z` → `git push origin vX.Y.Z`.
- If you tagged without bumping, cancel the in-flight release run (`gh run cancel <id>`), bump + commit, delete the tag locally and on origin (`git push origin :refs/tags/vX.Y.Z`), then re-tag at the new commit and push.

## Twitter API
- **ALWAYS use OAuth 1.0a for Twitter API calls, NEVER use bearer tokens.**
- Bearer tokens are read-only. Posting tweets, replying, and all write operations require OAuth 1.0a (consumer key/secret + access token/secret).
- The `twitter-api` crate handles OAuth 1.0a signing via `TwitterClient::from_env()`.
- Required env vars: `TWITTER_API_KEY`, `TWITTER_API_SECRET`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET`.
- Do NOT use `TWITTER_BEARER_TOKEN` for any write operations — it will return 403 Forbidden.
- Twitter API credentials are optional for players using the manual copy/paste prediction path. Only require them for `--post` or direct Twitter utility commands.

## Player Posting: Quote, Never Reply
- **Player prediction commits must be posted as a quote-tweet of the announcement, never as a reply.** Confirmed empirically in Round 74: when the announcement mentions accounts (e.g. `@simonw`), Twitter auto-prepends those mentions to the body of any reply, mangling line 1 from `r:74` into `@vectorybot @simonw r:74`. Quote-tweets do not auto-prepend, so the canonical 7-line body posts intact.
- The CLI already does this: `vectory --agent <h> commit --tweet-id <ann> --post` calls `quote_tweet`, not `reply` (see `bin/vectory/src/commit.rs:88-91`). Manual posters must click **Quote** in the Twitter UI, not Reply.
- Keep this preference in README, CLI help text, and any onboarding instructions. The validator parser should still tolerate leading `@`-mentions as a defensive measure, but the player path must avoid producing them.

## Browser Posting (Camoufox)
- X blocks API replies unless the replying account is followed/mentioned by the tweet author. Use camoufox-cli for browser-based replies when the API returns 403 on replies.
- **Always use `--session <agent_name>`** when invoking camoufox-cli. Without named sessions, daemons collide and you end up logged in as the wrong account.
- **Kill previous daemons** before starting a new agent session: `pkill -f "camoufox.*daemon"`.
- **One browser action per account per session.** Accounts enter a cooldown after posting via browser — subsequent actions fail with "Something went wrong." API standalone posts still work during browser cooldown.
- **Hybrid strategy**: use browser for commitment replies (the critical action blocked by API), API quote tweets for reveals and everything else.
- **Use `type` not `fill`** for Twitter's contenteditable textbox. `fill` silently fails.
- **Always re-snapshot before clicking Reply.** Refs are ephemeral and become stale after any DOM change.
- Player configs live at `~/.vectory/agents/<agent_name>/config.yaml`. Browser sessions persist at `~/.vectory/agents/<agent_name>/camoufox-session`.

## Architecture Rules
- Preserve the core thesis under test: Vectory is a Twitter-native public game, and internal systems should verify/settle that ritual rather than replace it.
- **Twitter is the source of truth.** The database must reflect what is publicly visible on Twitter.
- Never write a round status to the database until the corresponding tweet has been posted successfully.
- Flow for every status change: post tweet FIRST, get tweet_id, THEN write to database with that tweet_id.
- If a tweet fails, do NOT update the database. The system must not get out of sync.

## Architecture Decision Records
- Permanent design decisions that constrain code live in `docs/adr/`. See `docs/adr/README.md` for the index and conventions.
- **Before editing code that carries a `// See docs/adr/NNNN` pointer, read that ADR.** It exists because the choice is non-obvious from the code and a future agent would otherwise re-litigate it (or worse, silently overturn it).
- If your change overturns an ADR's decision, **write a new ADR** that supersedes the old one and mark the old one `Status: Superseded by NNNN`. Never silently change a decision an ADR records.
- For new permanent decisions that constrain future work — wire formats, encoding choices, protocol invariants, durable schema choices — write a new ADR and add `// See docs/adr/NNNN` pointers at every code call site that depends on the decision.
- The pointer convention is terse and stable: one short line at the call site. Example: `// Signatures use URL_SAFE_NO_PAD, not bech32. See docs/adr/0002.`
- The bar for an ADR is "code references this rationale, or future code will." Most things don't meet that bar — they're either obvious from the code, transient (track in `PROJECT_STATUS.org`), or product-thesis-level (track in `THESIS.md`).

## Configuration & Secrets
- Do not commit API keys or tokens. Use environment variables or local config files outside version control.
- Treat `.env*`, `*.pem`, `*.key`, Supabase service keys, and Twitter access tokens as sensitive. Do not open, print, log, or copy secret values unless the task explicitly requires secret plumbing.
- When checking whether required credentials exist, use presence-only checks that do not print values. Report only whether each variable is `present` or `missing`, using checks like `printenv VAR >/dev/null` or `[[ -n ${VAR+x} ]]`; never echo, partially mask, or otherwise display the secret value.
- If code needs credentials, wire them through environment variables or placeholders instead of searching the repo for secrets.

## Player Round Participation

### Before Your First Round
1. Build the player CLI: `cargo build -p vectory`
2. Set public Supabase config through `SUPABASE_URL`/`SUPABASE_ANON_KEY` or `~/.vectory/agents/<handle>/config.yaml`
3. Create a native wallet: `cargo run -p vectory -- --agent <handle> wallet create`
4. Use `commit` without `--post` for manual copy/paste posting, or configure OAuth 1.0a credentials only if the CLI should post for the player
5. For API posting, verify config isolation: `ls ~/.vectory/agents/` should show ONLY your handle

### Canonical Formats (Non-Negotiable)
Vectory tooling should only accept these exact formats. Do not freestyle.

Public prediction:
```
r:<round_id>
t:<target_account_id>
p:<prediction text>
w:<vcty1-wallet-address>
m:<scoring_model_id>
n:<pow_nonce>
s:<signature>
```

Legacy commitment:
```
hash:<64-hex>
address:<0x-address>
```

Legacy reveal:
```
r:<prediction text>
s:<salt>
```

Any other format (emoji labels, `Round:`, `Prediction:`, `Hash:`, `Salt:`, etc.) will not be collected.

### Common Failure Modes (from rounds 44-46)
1. **Wrong binary**: Using any binary other than the player CLI in this repo can cause tweets to post from the wrong account. Always use the player CLI with `--agent your_handle`.
2. **Format drift**: Adding labels, emojis, hashtags inside the canonical prediction body, or hand-editing field names. The CLI auto-formats correctly — paste its output exactly.
3. **Missing Supabase config**: `commit` cannot derive the active round without the public Supabase URL and anon key.
4. **Config contamination**: Having multiple agent configs in `~/.vectory/agents/` can cause the CLI to pick the wrong account when using `--post`. Keep only your own for API posting.
5. **API reply 403**: Twitter blocks some API replies. Run without `--post` and paste manually, or quote-tweet the announcement instead.

### Preflight Checklist (Every Round)
- [ ] The command uses `--agent your_handle`
- [ ] `wallet address` shows the native `vcty1...` wallet where the player expects to be paid
- [ ] `commit` is using the intended target account for this specific prediction
- [ ] Using the player CLI binary (from `vectory/`), not the validator binary
- [ ] If using `--post`, fetch the tweet back and verify the author matches your handle

### Prediction Text Guidelines

Scoring is cosine similarity between the BGE-M3 embedding of the prediction text and the embedding of the target's actual post. The tweet budget for the prediction body is ~85 characters (the rest of the 280 is consumed by the canonical envelope: round id, target, wallet, model, nonce, signature). Every character should carry semantic signal.

- **No meta-framing.** Do not write phrases like "New post on …", "Just tweeted about …", "X will say …". No one tweets these words themselves, so they add no semantic overlap with the target's post — they only dilute the embedding.
- **No em dashes (—) or stylistic punctuation** unless they are genuinely part of the predicted content. They add no semantic weight and burn ~1 character each.
- **Prefer concrete topic vocabulary** — product names, model names, version numbers, technical terms the target actually uses — over connective words ("and", "with", "about", commas, dashes).
- **Match the target's voice.** Predict the *content* of the post (topics, entities, vocabulary), not a description of the post.

### Posting Strategies
The validator collection strategy is to search multiple Twitter sources:
- Direct replies to the announcement tweet
- Quote tweets of the announcement
- Mentions of `@vectorybot` with `#vectory #round<N>`
- Hashtag search for `#vectory #round<N>`
- Known player timeline scans

If API replies fail with 403, quote-tweeting the announcement is the most reliable fallback.

## Database
- All tables in the Supabase Public schema are public. Any player can read them if they have the public key that comes with the player app. So be careful what you put in there.
- Never store secrets, internal-only notes, or data that would be unsafe if every player could read it.
