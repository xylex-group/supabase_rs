# RPC Implementation Design Document

## 1. Architectural Analysis

The current `supabase_rs` architecture follows a clear "Fluent Builder" pattern for database operations.
*   **Client**: `SupabaseClient` acts as the entry point, holding connection state (`reqwest::Client`), configuration (`url`, `key`), and schema information.
*   **Builders**: Operations like `select` return a `QueryBuilder`. This builder accumulates state (filters, sorts, pagination) in a `Query` struct without executing immediately.
*   **Execution**: The `.execute()` method triggers the HTTP request. For `select`, this is a `GET` request to `/rest/v1/{table}`.
*   **Query Construction**: The `Query` struct handles the logic of converting filters (e.g., `eq`, `gt`) into PostgREST-compatible URL query parameters.

**Gap Analysis for RPC**:
*   **Method**: RPC calls require `POST` requests, unlike the `GET` used for `select`.
*   **Parameters**: RPC arguments are sent in the JSON request body, not the URL.
*   **Filtering**: PostgREST allows filtering the *return value* of an RPC function (if it returns a table) using the same query parameters as `select`.
*   **Endpoint**: The endpoint format differs: `/rest/v1/rpc/{function_name}`.

## 2. RPC Design

To maintain consistency with the existing architecture, we will implement a `RpcBuilder` pattern.

*   **RpcBuilder**: Similar to `QueryBuilder`, but specialized for RPCs.
*   **State Separation**: It will manage two distinct sets of data:
    1.  **Function Arguments**: Passed in the body (JSON).
    2.  **Result Filters**: Passed in the URL query string (reusing the existing `Query` logic).
*   **Chaining**: The builder will support chaining filter methods (`eq`, `order`, etc.) which will apply to the *result* of the function call.

## 3. Module Structure

New files and modifications:

```text
src/
├── rpc.rs              # NEW: Implementation of RpcBuilder and execution logic
├── lib.rs              # MOD: Export rpc module and add rpc() method to SupabaseClient
└── tests/
    └── methods/
        └── rpc.rs      # NEW: Integration tests for RPC calls
```

### Integration
*   `src/lib.rs`: Add `pub mod rpc;` and `SupabaseClient::rpc(...)`.
*   `src/rpc.rs`: Will use `crate::query::Query` to reuse filter logic.

## 4. API Design

### `SupabaseClient` Extension

```rust
impl SupabaseClient {
    /// Calls a Postgres RPC function.
    ///
    /// # Arguments
    /// * `function_name` - The name of the RPC function to call.
    /// * `params` - The arguments to pass to the function (serializable to JSON).
    pub fn rpc<T>(&self, function_name: &str, params: T) -> RpcBuilder
    where
        T: serde::Serialize;
}
```

### `RpcBuilder` Interface

```rust
pub struct RpcBuilder {
    client: SupabaseClient,
    function_name: String,
    params: serde_json::Value,
    query: Query, // Reusing existing Query struct for filters
}

impl RpcBuilder {
    // Filter methods (delegating to self.query)
    pub fn eq(mut self, column: &str, value: &str) -> Self;
    pub fn gt(mut self, column: &str, value: &str) -> Self;
    // ... other standard filters ...
    
    // Modifiers
    pub fn order(mut self, column: &str, ascending: bool) -> Self;
    pub fn limit(mut self, n: usize) -> Self;

    // Execution
    pub async fn execute(self) -> Result<Vec<Value>, String>;
    pub async fn execute_void(self) -> Result<(), String>; // For functions returning void
    pub async fn execute_single(self) -> Result<Value, String>; // For scalar returns
}
```

## 5. Implementation Details

### `src/rpc.rs`

```rust
use crate::{SupabaseClient, query::{Query, QueryBuilder}};
use serde::Serialize;
use serde_json::{Value, json};

pub struct RpcBuilder {
    client: SupabaseClient,
    function_name: String,
    params: Value,
    query: Query,
}

impl RpcBuilder {
    pub fn new<T: Serialize>(client: SupabaseClient, function_name: &str, params: T) -> Self {
        Self {
            client,
            function_name: function_name.to_string(),
            params: serde_json::to_value(params).unwrap_or(json!({})),
            query: Query::new(),
        }
    }

    // Example filter delegation
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("eq.{}", value));
        self
    }

    pub async fn execute(self) -> Result<Vec<Value>, String> {
        let endpoint = format!("{}/rest/v1/rpc/{}", self.client.url, self.function_name);
        let query_string = self.query.build();
        let url = if query_string.is_empty() {
            endpoint
        } else {
            format!("{}?{}", endpoint, query_string)
        };

        // Reuse headers logic from SupabaseClient (likely need to expose or duplicate)
        // ... header setup ...

        let response = self.client.client
            .post(&url)
            .headers(headers)
            .json(&self.params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // Reuse response handling
        crate::success::handle_response(response).await
    }
}
```

## 6. Testing Strategy

### Unit Tests
*   **Builder Logic**: Verify that calling `.rpc(...).eq(...)` correctly populates both `params` (body) and `query` (URL params).
*   **Serialization**: Ensure parameters are correctly serialized to JSON.

### Integration Tests
*   **Setup**: Requires a Supabase instance with specific RPC functions defined (e.g., `echo_text`, `get_filtered_users`).
*   **Scenarios**:
    1.  **Void Return**: Call a function that performs an action but returns nothing.
    2.  **Scalar Return**: Call a function returning an integer or string.
    3.  **Table Return**: Call a function returning `SETOF users`.
    4.  **Filtering**: Call a table-returning function and apply `.eq()` and `.limit()`.
    5.  **Error**: Call a non-existent function or pass invalid parameters.

## 7. Type Generation Extension

The `src/type_gen.rs` module should be updated to support generating types for RPC functions.

*   **Discovery**: Query `information_schema.routines` where `routine_type = 'FUNCTION'` and `routine_schema = 'public'`.
*   **Parameters**: Join with `information_schema.parameters` to generate `Args` structs for each function.
*   **Output**:
    *   Generate a `Rpc` module or `rpc` submodule in `supabase_types.rs`.
    *   For each function, generate a helper struct:
        ```rust
        pub struct MyFunctionArgs {
            pub param1: String,
            pub param2: i32,
        }
        ```

## 8. Error Handling

*   **Network Errors**: handled by `reqwest` conversion.
*   **Postgres Errors**: PostgREST returns specific error JSON bodies (e.g., `PGRST200`). These should be parsed and returned as structured errors if possible, or clear strings.
*   **Argument Errors**: If serialization of `params` fails, return an immediate error before the request.

## 9. Performance Considerations

*   **Connection Pooling**: `RpcBuilder` reuses the `SupabaseClient`'s reqwest client, ensuring connection reuse.
*   **Allocation**: `Query` uses `Vec<(String, String)>`. For high-performance scenarios, we might consider using `Cow<str>` to reduce string cloning during builder chaining.
*   **Params**: `serde_json::Value` involves some overhead. The `rpc` method takes `T: Serialize`, which is efficient. The storage in `RpcBuilder` as `Value` allows flexibility but causes one serialization step. This is acceptable for typical usage.

## 10. Documentation Requirements

*   **Method Docs**: `SupabaseClient::rpc` must have examples showing both simple calls and filtered calls.
*   **Module Docs**: `src/rpc.rs` should explain that filters apply to the *result* of the function.
*   **README**: Add a section "Calling Stored Procedures".

### Example Documentation
```rust
/// Calls a Postgres RPC function.
///
/// # Examples
///
/// ```rust,no_run
/// // Call a function 'echo' with arguments
/// let result = client.rpc("echo", json!({"message": "hello"})).execute().await?;
///
/// // Call a function returning a table and filter the results
/// let users = client.rpc("get_active_users", json!({}))
///     .eq("role", "admin")
///     .limit(10)
///     .execute()
///     .await?;
/// ```
```
