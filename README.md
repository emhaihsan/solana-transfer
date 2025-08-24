# Solana CPI Token Transfer

A Solana program that performs an SPL token transfer via Cross-Program Invocation (CPI) using a Program Derived Address (PDA) as the authority.

- Entrypoint: [src/lib.rs](https://github.com/emhaihsan/solana-transfer/blob/main/src/lib.rs:0:0-0:0) ([process_instruction](https://github.com/emhaihsan/solana-transfer/blob/main/src/lib.rs:19:0-107:1))
- Transfer uses `spl_token::instruction::transfer_checked` via `invoke_signed`
- PDA seeds: `b"authority"`
- Current transfer amount: `10_000` (hard-coded)

## Test

- Run: `cargo test`
- Integration test: [tests/test.rs](https://github.com/emhaihsan/solana-transfer/blob/main/tests/test.rs) (creates mint/accounts, mints tokens, invokes program, asserts transfer)

## Repo

- https://github.com/emhaihsan/solana-transfer

---

This is my submission #2 for the Rise In Solana course.
