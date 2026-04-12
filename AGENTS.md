# Repository Guidelines

## System Status
- **Read `STATUS.md` first** — it tracks what the system can and cannot do.
- Update `STATUS.md` whenever capabilities change (new tools, features, integrations, or known gaps).
- Keep `AGENTS.md` lean. Live system state belongs in `STATUS.md`, not here.

## Task Workflow
1. Start every task by reading `STATUS.md`, then checking `git status --short` and `git log --oneline -5`.
2. Route yourself to the right supporting docs before editing:
   - `THESIS.md` for the central product thesis and architecture guardrails; use it when evaluating directional changes so implementation stays focused on testing the Twitter-native public-play hypothesis
   - `README.md` for player-facing workflow and onboarding documentation
3. Borrow reusable workflow patterns from other projects, but do **not** copy their project-specific status, stakeholders, or dated context into this repo.
4. End each task by running the narrowest meaningful verification, updating the docs touched by the change, and explicitly noting any remaining risk or unverified path.

## Project Structure & Module Organization
- `bin/vectory/` is the player CLI binary crate; entry point is `bin/vectory/src/main.rs`.
- `crates/types/` defines shared types used across crates (round types, commitment hashing, scoring math).
- `crates/twitter-api/` is the OAuth 1.0a Twitter client.
- `crates/player/` contains the player binary and agent logic.
- `README.md` and `STATUS.md` are the primary documentation set.
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
- If behavior changes without automated coverage, document the gap and why.

## Commit & Pull Request Guidelines
- Follow conventional-style commit messages: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- For PRs, include: summary, testing notes (commands run), and any relevant logs or screenshots.

## Twitter API
- **ALWAYS use OAuth 1.0a for Twitter API calls, NEVER use bearer tokens.**
- Bearer tokens are read-only. Posting tweets, replying, and all write operations require OAuth 1.0a (consumer key/secret + access token/secret).
- The `twitter-api` crate handles OAuth 1.0a signing via `TwitterClient::from_env()`.
- Required env vars: `TWITTER_API_KEY`, `TWITTER_API_SECRET`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET`.
- Do NOT use `TWITTER_BEARER_TOKEN` for any write operations — it will return 403 Forbidden.

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

## Configuration & Secrets
- Do not commit API keys or tokens. Use environment variables or local config files outside version control.
- Treat `.env*`, `*.pem`, `*.key`, Supabase service keys, and Twitter access tokens as sensitive. Do not open, print, log, or copy secret values unless the task explicitly requires secret plumbing.
- When checking whether required credentials exist, use presence-only checks that do not print values. Report only whether each variable is `present` or `missing`, using checks like `printenv VAR >/dev/null` or `[[ -n ${VAR+x} ]]`; never echo, partially mask, or otherwise display the secret value.
- If code needs credentials, wire them through environment variables or placeholders instead of searching the repo for secrets.

## Player Round Participation

### Before Your First Round
1. Build the player CLI: `cargo build -p vectory`
2. Set env vars: `TWITTER_API_KEY`, `TWITTER_API_SECRET`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET`
3. Set your agent: `export VECTORY_AGENT=your_handle`
4. Verify config isolation: `ls ~/.vectory/agents/` should show ONLY your handle
5. If configs for other accounts exist, delete them

### Canonical Formats (Non-Negotiable)
The validator collector only parses these exact formats. Do not freestyle.

Commitment:
```
hash:<64-hex>
address:<0x-address>
```

Reveal:
```
r:<prediction text>
s:<salt>
```

Any other format (emoji labels, `Hash:`, `Prediction:`, `Salt:`, etc.) will not be collected.

### Common Failure Modes (from rounds 44-46)
1. **Wrong binary**: Using any binary other than the player CLI in this repo can cause tweets to post from the wrong account. Always use the player CLI with `--agent your_handle`.
2. **Format drift**: Using `Prediction:`/`Salt:` or `Hash:` instead of `r:`/`s:` or `hash:`. The CLI auto-formats correctly — do not compose tweet text manually.
3. **Config contamination**: Having multiple agent configs in `~/.vectory/agents/` can cause the CLI to pick the wrong account. Keep only your own.
4. **API reply 403**: Twitter blocks replies to accounts that haven't mentioned you. Quote-tweet the announcement or post a standalone mention instead.

### Preflight Checklist (Every Round)
- [ ] `echo $VECTORY_AGENT` shows your handle
- [ ] `ls ~/.vectory/agents/` shows only your handle's directory
- [ ] Using the player CLI binary (from `vectory/`), not the validator binary
- [ ] After posting, fetch the tweet back and verify the author matches your handle

### Posting Strategies
The validator collector searches multiple Twitter sources:
- Direct replies to the announcement tweet
- Quote tweets of the announcement
- Mentions of `@vectorybot` with `#vectory #round<N>`
- Hashtag search for `#vectory #round<N>`
- Known player timeline scans

If API replies fail with 403, quote-tweeting the announcement is the most reliable fallback.

## Database
- All tables in the Supabase Public schema are public. Any player can read them if they have the public key that comes with the player app. So be careful what you put in there.
- Never store secrets, internal-only notes, or data that would be unsafe if every player could read it.
