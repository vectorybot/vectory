# Architecture Decision Records

Permanent design decisions that constrain Vectory's code. Each ADR captures the context, the chosen direction, and the consequences so future contributors don't re-litigate settled questions.

## Conventions

- One decision per file, named `NNNN-short-slug.md` with a zero-padded sequential number.
- Status is one of `Proposed`, `Accepted`, `Superseded by NNNN`, or `Deprecated`.
- Once accepted, an ADR is immutable except for status changes. To overturn a decision, write a new ADR that supersedes the old one.
- Code that depends on an ADR carries a `// See docs/adr/NNNN` pointer at the call site, so anyone modifying the code finds the rationale immediately.

## Index

| # | Title | Status |
|---|---|---|
| [0001](0001-bech32-addresses.md) | Wallet and validator addresses use bech32 with HRP `vcty` | Accepted |
| [0002](0002-url-safe-nopad-signatures.md) | Player commit signatures use URL_SAFE_NO_PAD base64 | Accepted |

When adding a new ADR, append a row here.
