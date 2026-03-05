# RPC Implementation Design V2

## 1. Architectural Integration

The RPC implementation will integrate into the existing `supabase_rs` architecture by adding a new `RpcBuilder` that parallels the existing `QueryBuilder`. This ensures a consistent developer experience while handling the unique requirements of PostgREST RPC calls (POST method, body parameters, and URL filtering).

### Integration Points
1.  **`SupabaseClient`**: Extended with a `rpc()` method serving as the entry point.
2.  **`RpcBuilder`**: A new builder struct in `src/rpc.rs` that manages the request lifecycle.
3.  **`Query`**: Reused from `src/query.rs` to handle response filtering (filtering the return set of a function).
4.  **`Headers`**: Enhanced to support schema switching via `Content-Profile` and `Accept-Profile`.

## 2. API Specification

### `SupabaseClient` Extension
We will add the `rpc` method to `SupabaseClient` in `src/lib.rs`.

```rust
// src/lib.rs

impl SupabaseClient {
    /// Calls a Postgres RPC function.
    ///
    /// # Arguments
    /// * `function_name` - The name of the RPC function to call.
    /// * `params` - The arguments to pass to the function. Can be a struct, map, or `json!({})`.
    ///
    /// # Returns
    /// Returns a `RpcBuilder` for further chaining (filtering) or execution.
    pub fn rpc<T>(&self, function_name: &str, params: T) -> crate::rpc::RpcBuilder
    where
        T: serde::Serialize,
    {
        crate::rpc::RpcBuilder::new(self.clone(), function_name, params)
    }
}
```

### `RpcBuilder` Structure
Located in `src/rpc.rs`, this builder manages the state.

```rust
// src/rpc.rs

pub struct RpcBuilder {
    client: SupabaseClient,
    function_name: String,
    params: serde_json::Value,
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

    // ... filters ...
    // ... execution methods ...
}
```

## 3. Header Management

RPC calls require specific headers to handle schemas correctly. PostgREST uses:
*   `Content-Profile`: To specify the schema for the input arguments (request body).
*   `Accept-Profile`: To specify the schema for the return value (response body).

We must update `src/request/headers.rs` or handle this in `RpcBuilder::execute`. Since `SupabaseClient` holds the `schema` (defaulting to "public"), we use this.

```rust
// Inside RpcBuilder::execute()

let mut headers = self.client.headers.clone(); // Assuming headers are accessible or re-created

// 1. Content-Type is always application/json
headers.insert("Content-Type", "application/json");

// 2. Handle Schema
if self.client.schema != "public" {
    // Tell PostgREST to search for the function/types in this schema
    headers.insert("Content-Profile", &self.client.schema);
    // Tell PostgREST to return results from this schema
    headers.insert("Accept-Profile", &self.client.schema);
}

// 3. RPC Parameter Format
// Default PostgREST behavior expects a JSON object for parameters.
// If we ever need single unnamed parameter support, we'd use:
// headers.insert("Prefer", "params=single-object"); 
```

## 4. Endpoint Construction

We need a dedicated method to construct the RPC endpoint, handling the `rest/v1` prefix and the `rpc/` segment.

```rust
// src/lib.rs

impl SupabaseClient {
    pub(crate) fn rpc_endpoint(&self, function_name: &str) -> String {
        let dont_use_rest_v1: bool = std::env::var("SUPABASE_RS_DONT_REST_V1_URL")
            .map(|val| val.to_lowercase() == "true")
            .unwrap_or(false);

        let base = if dont_use_rest_v1 {
            format!("{}/rpc", self.url)
        } else {
            format!("{}/rest/v1/rpc", self.url)
        };

        format!("{}/{}", base, function_name)
    }
}
```

## 5. Parameter Handling System

To minimize serialization overhead while keeping the API flexible:
1.  **Input**: The `rpc` method takes `T: Serialize`.
2.  **Storage**: We convert to `serde_json::Value` immediately in `new()`. This allows `RpcBuilder` to be non-generic, simplifying the type signature for chaining.
    *   *Optimization Note*: While this causes one serialization, it avoids complex generic bounds on the Builder which would complicate `Filter` trait implementations or delegation.
3.  **Sending**: The `reqwest` client takes `&Value` directly.

## 6. Response Type System

RPC functions can return:
1.  `void`: No content.
2.  `scalar`: A single value (int, string, bool).
3.  `row`: A single object.
4.  `setof`: An array of objects.

We provide typed execution methods:

```rust
impl RpcBuilder {
    /// Default execution expecting a list of results (SETOF records)
    pub async fn execute(self) -> Result<Vec<Value>, ErrorTypes> {
        // ... implementation ...
    }

    /// For functions returning a single scalar or object
    pub async fn execute_single(self) -> Result<Value, ErrorTypes> {
         // ... sets Accept: application/vnd.pgrst.object+json if needed, or parses result ...
    }

    /// For functions returning void
    pub async fn execute_void(self) -> Result<(), ErrorTypes> {
         // ... implementation checks for 204 No Content ...
    }
}
```

## 7. Error Handling Integration

We will use the existing `crate::errors::ErrorTypes` and `Result`.

*   **PostgREST Errors**: Map HTTP 404 (Function not found) and 500 (SQL Error) to `ErrorTypes`.
*   **Parsing**: PostgREST returns JSON error bodies (e.g., `{"code": "...", "message": "..."}`). We should attempt to parse this into a structured error message within `ReqwestError` or a new `PostgresError` variant if we decide to expand `ErrorTypes`. For now, we'll stick to stringifying the error body into the `Result`.

```rust
// In execute()
let status = response.status();
if !status.is_success() {
    let error_body = response.text().await.unwrap_or_default();
    return Err(ErrorTypes::ReqwestError(
        // Construct a meaningful error
    ));
}
```

## 8. Type Generation Extension

We will extend `src/type_gen.rs` to generate types for RPC function arguments.

**SQL Query:**
```sql
SELECT
    r.routine_name,
    p.parameter_name,
    p.data_type,
    p.ordinal_position,
    p.parameter_mode
FROM information_schema.routines r
JOIN information_schema.parameters p 
    ON r.specific_name = p.specific_name
WHERE r.routine_type = 'FUNCTION' 
  AND r.routine_schema = 'public' -- or configurable
ORDER BY r.routine_name, p.ordinal_position;
```

**Rust Output Strategy:**
Generate a `pub mod rpc` inside `supabase_types.rs`.
For a function `create_user(name text, age int)`, generate:

```rust
pub mod rpc {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct CreateUserArgs {
        pub name: String,
        pub age: i32,
    }
}
```

## 9. Testing Infrastructure

### Setup Script (`src/tests/setup_rpc.sql`)
```sql
-- Void return
CREATE OR REPLACE FUNCTION test_void_func() RETURNS void AS $$
BEGIN
    -- do nothing
END;
$$ LANGUAGE plpgsql;

-- Scalar return
CREATE OR REPLACE FUNCTION test_echo(val text) RETURNS text AS $$
BEGIN
    RETURN val;
END;
$$ LANGUAGE plpgsql;

-- Set return
CREATE OR REPLACE FUNCTION test_get_users() RETURNS SETOF users AS $$
BEGIN
    RETURN QUERY SELECT * FROM users;
END;
$$ LANGUAGE plpgsql;
```

### Integration Test (`src/tests/methods/rpc.rs`)
```rust
#[tokio::test]
async fn test_rpc_echo() {
    let client = create_client();
    let res = client.rpc("test_echo", json!({"val": "hello"}))
        .execute_single()
        .await
        .unwrap();
    assert_eq!(res.as_str().unwrap(), "hello");
}
```

## 10. Schema Support

The implementation must strictly respect `SupabaseClient.schema`.
1.  **Header Injection**: As detailed in section 3, inject `Content-Profile` and `Accept-Profile`.
2.  **Type Gen**: The generator must accept a schema parameter to inspect `information_schema.routines` for that specific schema.

## 11. Performance Optimizations

1.  **Connection Pooling**: Inherited from `SupabaseClient` (reqwest).
2.  **String Cloning**: `RpcBuilder` stores `String` for function name. This is negligible.
3.  **Serialization**: We avoid double serialization in the request body by taking `&Value` in the final `reqwest` call.

## 12. Feature Flag Strategy

Add a `rpc` feature to `Cargo.toml`.

```toml
[features]
default = []
rpc = []
```

Conditionally compile the `rpc` module in `lib.rs`:

```rust
#[cfg(feature = "rpc")]
pub mod rpc;

impl SupabaseClient {
    #[cfg(feature = "rpc")]
    pub fn rpc(...) { ... }
}
```

## 13. Migration Considerations

*   **Non-Breaking**: Adding `rpc` is additive.
*   **Deprecation**: None required.
*   **Compatibility**: Works with standard PostgREST setup. Requires no changes to existing `select`, `insert`, etc.
