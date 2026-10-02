# Contributing

Use the stable Rust toolchain specified by `Cargo.toml` (`rust-version = "1.85"`). Format with `cargo fmt --all -- --check`.

Run Cargo commands, including tests, under WSL2:

```bash
cargo check --no-default-features
cargo test --all-features --lib
cargo clippy --all-targets --all-features
```

The CRUD/RPC tests are ignored by default and need the local Supabase stack. Start and seed it, then opt into those tests explicitly:

```bash
supabase start
supabase db reset
SUPABASE_URL=http://127.0.0.1:54321 SUPABASE_KEY=<local-anon-key> \
  cargo test --all-features --lib -- --include-ignored --test-threads=1
```

RPC setup uses the local Postgres defaults or the `SUPABASE_DB_*` variables. Unit and HTTP contract tests use local HTTP servers and need no credentials.

Keep changes focused, add an assertion for behavior changes, and update the README source in `src/lib.rs` before running `cargo rdme` so generated README and crate docs stay aligned.
