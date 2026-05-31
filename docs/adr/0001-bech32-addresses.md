# 0001. Wallet and validator addresses use bech32 with HRP `vcty`

- Date: 2026-05-31
- Status: Accepted

## Context

A Vectory wallet address identifies a player on Twitter, in the canonical commit body, and (eventually) in the native ledger. Players see and copy these addresses; they appear in tweets, prediction receipts, and any future payout flow. The encoding directly shapes user perception of the protocol.

The original encoding was `vcty1` + URL-safe base64 of the 32-byte Ed25519 public key, e.g. `vcty11ctx-62u4Cj0w6Ez1RloBFVAEsPl8uZxNDrWQq-q6bw`. This produced addresses with `-` and `_` characters, which looked foreign to anyone used to mainstream chain addresses:

| Chain | Encoding | Charset | Hyphens? |
|---|---|---|---|
| Bitcoin (legacy), Solana | Base58 | excludes `0`, `O`, `I`, `l` deliberately | no |
| Bitcoin SegWit, Cosmos, modern chains | Bech32 | lowercase letters + digits, `1` separator | no |
| Ethereum, hex chains | Hex | `0-9 a-f` | no |
| Vectory (original) | URL-safe base64 | `A-Z a-z 0-9 - _` | **yes** |

The `vcty1` prefix telegraphed Bech32 (the `1` is bech32's HRP/data separator), so onlookers expected bech32 behavior — no hyphens, all lowercase, built-in checksum — and got URL-safe base64 instead. The mismatch was a UX cost paid every time someone read or copied an address.

## Decision

Encode wallet and validator addresses as **bech32 with HRP `vcty`**. The pubkey-bearing payload after `vcty1` is the 32-byte Ed25519 public key in bech32's base32 alphabet, plus a 6-character BCH checksum.

`vcty` (not `vc`) is the chosen HRP. `vc` would save two characters on every address but loses immediate brand recognition with the `$VCTY` token symbol, which matters more than the marginal density savings on a 63-character string.

Implementation sites:
- `vectory/bin/vectory/src/wallet.rs::encode_address` (player wallets)
- `vectory-internal/services/ledger/src/main.rs::encode_address` (validator address and sender-pubkey derivation check)

Both use the `bech32` crate (0.11+) with the same HRP constant.

## Alternatives considered

- **Keep URL-safe base64.** Cheapest path, but leaves the cosmetic UX cost in place forever and loses the checksum benefit. Rejected on UX grounds.
- **Hex.** Familiar from Ethereum, no hyphens, but no checksum and longer than bech32 for the same data. Rejected.
- **Base58 (Bitcoin/Solana style).** No hyphens, but no built-in checksum (Bitcoin layers Base58Check on top) and not aligned with the `vcty1` prefix convention. Rejected.
- **HRP = `vc` instead of `vcty`.** Saves 2 chars per address. Rejected — token symbol is `$VCTY` and brand recognition wins over marginal density.

## Consequences

- Addresses are lowercase `a-z 0-9` only, no hyphens or underscores.
- Length grows from 48 chars (URL-safe base64) to 63 chars (bech32), an acceptable cost for the BCH checksum that catches single-character typos.
- Wallets stored on disk (`~/.vectory/agents/<handle>/wallet.json` and `~/.vectory/agents/validator/ledger/key.json`) re-derive the address from the stored private key on load and auto-rewrite if the on-disk address no longer matches the canonical encoding. This silently migrates any pre-bech32 wallet on the next load.
- Player CLIs that cached a validator pubkey under the old encoding (via `vectory validator-info`) must re-run that command after upgrade — the cache becomes stale.
- The HRP constant is duplicated across two crates that can't share types (the player CLI and the ledger service). Both must change together if the HRP is ever revisited.
- Signatures intentionally stay in a different encoding (see [0002](0002-url-safe-nopad-signatures.md)) because their constraints differ.

## Related

- [[0002-url-safe-nopad-signatures]] — sibling decision for the other crypto-bearing field in commit tweets.
