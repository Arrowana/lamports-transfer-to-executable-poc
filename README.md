# LiteSVM executable-account CPI repro

Build the on-chain program, then run the LiteSVM harness:

```bash
cargo build-sbf
cargo test
```

The on-chain program CPIs into SPL Token `CloseAccount`, using a second loaded
executable program account as the recipient. The CPI refreshes the closed token
account's local view but leaves the executable recipient's local view stale.

The instruction accepts one byte. Flag `0` leaves the stale view untouched and
reproduces `ExternalAccountLamportSpend`. Flag `1` manually adds the closed
account's lamports to the executable account's local view, allowing the caller
to return successfully.
