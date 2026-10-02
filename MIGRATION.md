# Migrating to 0.8.0

Version 0.8.0 is a quality-hardening release. CRUD methods, fluent PostgREST queries, embedded joins, and the local Supabase integration suite remain available.

## Typed errors

Operations return `supabase_rs::Result<T>` (or `supabase_rs::Error` directly). Replace string matching on errors with variant and status inspection:

```rust
use supabase_rs::Error;

match client.select("users").execute().await {
    Ok(rows) => println!("{} rows", rows.len()),
    Err(Error::Api(api)) => eprintln!("HTTP {}: {}", api.status, api.message),
    Err(error) => eprintln!("{error}"),
}
```

`Error::Api` carries provider status, code, message, details, hint, and bounded raw response context where available. Transport, serialization, local I/O, invalid input, and configuration errors have their own variants.

## Count responses

Counts are no longer represented as synthetic rows. Use `execute_with_count()` and read `ResponseData::count`:

```rust
let response = client.select("users").count().execute_with_count().await?;
let rows = response.data;
let total = response.count;
```

## Client configuration

`SupabaseClient::new(url, key)` remains supported. Use the builder for explicit schema, REST prefix, or a reusable `reqwest::Client`:

```rust
let client = SupabaseClient::builder(url, key)
    .schema("public")
    .rest_prefix("rest/v1")
    .build()?;
```

Malformed project URLs and empty keys now fail during construction.

## Feature flags

- `rpc`: optional PostgREST RPC support.
- `storage`: optional Storage downloads; create references with `SupabaseStorage::new(&client, bucket, path)`. Private access is the default; call `.public()` for public objects.
- `graphql`: experimental GraphQL support. `nightly` remains as a compatibility alias.
- `typegen`: optional database type generation.
- `native_tls` is the default TLS backend; `rustls` selects rustls.

Remove references to the old Realtime module and the `SUPABASE_RS_DONT_REST_V1_URL` and nightly warning environment variables; they are not part of the 0.8 API.