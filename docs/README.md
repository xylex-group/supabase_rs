# Additional documentation

The crate overview and current feature flags are maintained in the [README](../README.md).

- [RPC usage and API](RPC.md) (enable the `rpc` feature)
- [Migration to 0.8.0](../MIGRATION.md)

Storage downloads are available with the `storage` feature. Construct a reference with `SupabaseStorage::new(&client, bucket, path)`; private objects use the configured API key, and `.public()` selects the public object endpoint.

GraphQL is available with the `graphql` feature and remains experimental.