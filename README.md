<!-- cargo-rdme start -->

Async Rust client for Supabase PostgREST.

CRUD operations and fluent PostgREST queries, including embedded joins, are available by default.
RPC, Storage, GraphQL, and type generation are opt-in Cargo features. GraphQL remains experimental.

```toml
[dependencies]
supabase_rs = "0.8"
```

```rust
use supabase_rs::SupabaseClient;

let client = SupabaseClient::new(
    std::env::var("SUPABASE_URL")?,
    std::env::var("SUPABASE_KEY")?,
)?;
let rows = client.select("users").eq("active", "true").execute().await?;
```

Select counts are response metadata, available through `execute_with_count`; they are not rows.
Errors use [`Error`](https://docs.rs/supabase_rs/latest/supabase_rs/errors/enum.Error.html) and preserve HTTP status and structured provider details where available.

Feature flags: `rpc`, `storage`, `graphql` (experimental), `typegen`, `rustls`, and `native_tls`.
The `nightly` flag remains as an alias for `graphql`.

<!-- cargo-rdme end -->
