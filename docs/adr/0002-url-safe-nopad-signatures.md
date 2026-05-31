# 0002. Player commit signatures use URL_SAFE_NO_PAD base64

- Date: 2026-05-31
- Status: Accepted
- Related: [[0001-bech32-addresses]]

## Context

Vectory commit tweets carry both a wallet address and an Ed25519 signature over the canonical prediction payload. [ADR 0001](0001-bech32-addresses.md) switched addresses to bech32 for UX reasons (lowercase, no hyphens, built-in checksum, matches the Cosmos convention telegraphed by the `vcty1` prefix). The natural follow-up question: should signatures use the same encoding for consistency?

Three viable encodings for a 64-byte Ed25519 signature:

| Encoding | Chars | Charset | Notes |
|---|---|---|---|
| URL_SAFE_NO_PAD base64 | 86 | `A-Z a-z 0-9 - _` | most compact |
| Bech32 | ~110 | lowercase + 6-char checksum | matches address encoding |
| Hex | 128 | `0-9 a-f` | matches Bitcoin/Ethereum signature display style |

The full canonical commit body must fit in **one tweet (≤280 chars)** and carry all of `r:<round_id>`, `t:<target_account_id>`, `p:<prediction text>`, `w:<vcty1...>`, `m:<scoring_model_id>`, `n:<pow_nonce>`, `s:<signature>`. Every fixed field consumes characters that would otherwise be available for `p:<prediction text>` — the only field whose length the player controls and whose length matters for scoring quality (longer, more specific predictions can score better cosine similarity).

Round 74 (`p:Claude Opus 4.8 agentic coding sandbox security Datasette llm Anthropic Python SQLite`) used ~85 chars of `p:` and the canonical body fit comfortably. But longer predictions, or future protocol additions (e.g. mining nonces, chain IDs), would tighten the budget. Char count matters.

Unlike addresses, signatures are **machine-only**. Users never read, copy, paste, or verify a signature by eye. The validator parses them, decodes them, and `VerifyingKey::verify`s. Nobody asks "why is there a hyphen in my signature" because nobody ever looks at a signature.

## Decision

Encode player commit signatures as **URL_SAFE_NO_PAD base64**.

The signature appears in `s:<signature>` in the canonical commit body. The same encoding is used on both the sign side (`Wallet::sign` in the player CLI) and the verify side (the future validator parser, see [PROJECT_STATUS.org](../../PROJECT_STATUS.org) "Add validator collection for public prediction commitments").

## Alternatives considered

- **Bech32 (match addresses).** Aesthetically symmetric with bech32 addresses, all-lowercase, gets a checksum. Costs ~24 extra chars per commit. Rejected — the checksum is wasted (anyone tampering with a signature gets a signature-verification failure anyway, which is a stronger guarantee than a 6-bit checksum), and the consistency win is invisible since users don't read signatures.
- **Hex (match Bitcoin/Ethereum convention).** Familiar from other chains, no hyphens. Costs ~42 extra chars per commit. Rejected for the same reason — invisible benefit, real tweet-budget cost.
- **Keep URL_SAFE_NO_PAD.** Most compact; preserves char budget for `p:<prediction text>`. Hyphens and underscores in the signature visually clash with the hyphen-free bech32 address, but only a parser ever sees both side by side. Accepted.

## Consequences

- Signatures contain `-` and `_` characters, visually inconsistent with addresses. This is **intentional** — the consistency cost is invisible (signatures are machine-only) while the char-budget benefit is concrete (~42 chars vs hex, ~24 chars vs bech32, all of which become available to `p:<prediction text>`).
- The Ed25519 signature is 64 raw bytes → 86 base64 chars. The full canonical body for a Round-74-style commit uses ~270 chars, leaving ~10 chars of headroom. Tight, but survivable.
- The validator-side parser must decode signatures via `URL_SAFE_NO_PAD` base64 specifically, not standard base64 (which uses `+` and `/` and would fail on tweet-typed `-` and `_`).
- If the tweet transport is ever dropped (e.g. predictions move off Twitter entirely, no 280-char limit), this decision should be revisited — the char-budget rationale evaporates and consistency-with-addresses becomes the stronger argument.
- Ledger transaction signatures (`services/ledger/src/main.rs`) currently use `STANDARD` base64, not `URL_SAFE_NO_PAD`. That's a separate decision governing a separate signing context (signed JSON over HTTP, not tweet bodies) and is **not** covered by this ADR. If unification across both surfaces becomes desirable, write a new ADR.

## Related

- [[0001-bech32-addresses]] — sibling decision for the address field. Different encoding because addresses are user-facing and signatures are not.
