# LiteSVM program-to-executable lamport movement

Build the on-chain program, then run the LiteSVM harness:

```bash
cargo build-sbf
cargo run
```

The on-chain program directly moves lamports from a program-owned account to
LiteSVM's executable SPL Token Program account. No System Program CPI is used,
because the System Program cannot debit an account owned by this program.
