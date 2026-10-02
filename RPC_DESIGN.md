# RPC design

RPC is implemented in `src/rpc.rs` behind the `rpc` Cargo feature. It uses the shared `SupabaseClient`, request headers, endpoint builder, and response parser.

See [RPC documentation](docs/RPC.md) for the current API and examples. The previous design drafts have been removed because they described planned code as current behavior.