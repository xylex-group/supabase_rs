# Architecture

`SupabaseClient` owns the validated project URL, API key, schema, REST prefix, and reusable `reqwest::Client`. CRUD, query-builder, and RPC requests construct endpoints and headers from that shared client.

Successful PostgREST row responses are decoded by the shared response module. Counts are parsed from `Content-Range` into `ResponseData::count`; provider failures become `Error::Api` with status and structured response fields.

Optional modules are feature-gated: `rpc`, `storage`, `graphql`, and `typegen`. GraphQL remains experimental. See [README](README.md) for usage and [RPC docs](docs/RPC.md) for RPC behavior.