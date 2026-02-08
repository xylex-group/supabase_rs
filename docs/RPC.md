# Remote Procedure Call (RPC) Support

## Overview

The RPC (Remote Procedure Call) module provides comprehensive support for calling PostgreSQL functions via PostgREST's RPC endpoints. This enables executing stored procedures, functions, and custom SQL operations with full parameter support and result filtering capabilities.

## 🎯 Implementation Summary

### Architecture
The RPC implementation integrates seamlessly into the existing `supabase_rs` architecture:

1. **`RpcBuilder`**: A fluent builder pattern that parallels the existing `QueryBuilder` for consistent developer experience
2. **`SupabaseClient` Extension**: Added `rpc()` method as the main entry point
3. **Reused Components**: Leverages existing `Query` infrastructure for filtering and `Headers` for schema support
4. **Feature-Gated**: RPC functionality is behind the `rpc` feature flag for optional inclusion

### Key Features
- **Full Parameter Support**: Type-safe serialization of function arguments using `serde`
- **Multiple Execution Modes**: Support for void, scalar, single row, and set-returning functions
- **Result Filtering**: Post-execution filtering of returned data sets using standard PostgREST query parameters
- **Schema Support**: Multi-schema support via `Content-Profile` and `Accept-Profile` headers
- **Type Generation**: Automatic generation of argument structs for RPC functions
- **Comprehensive Testing**: Integration tests covering all execution modes and edge cases

### Design Philosophy
- **Consistent API**: Follows the same fluent pattern as other SDK operations
- **Type Safety**: Leverages Rust's type system for compile-time validation
- **Performance**: Efficient serialization and HTTP request handling
- **Flexibility**: Supports all PostgREST RPC features including filtering

## 🔧 Feature Flags

RPC functionality is feature-gated. To enable it, add the `rpc` feature to your `Cargo.toml`:

```toml
[dependencies]
supabase_rs = { version = "0.5.1", features = ["rpc"] }
```

### Available Features
| Feature | Description | Default |
|---------|-------------|---------|
| `rpc` | Enables RPC functionality | ❌ Disabled |
| `storage` | File operations with Supabase Storage | ❌ Disabled |
| `rustls` | Use rustls instead of OpenSSL for TLS | ❌ Disabled |
| `nightly` | Experimental GraphQL support | ❌ Disabled |

### Default Features
The default feature set includes `native_tls` and `nightly`. To use RPC without nightly features:

```toml
supabase_rs = { version = "0.5.1", features = ["rpc", "native_tls"], default-features = false }
```

## 📚 API Reference

### `SupabaseClient` Extension

#### `rpc()`
```rust
pub fn rpc<T>(&self, function_name: &str, params: T) -> crate::rpc::RpcBuilder
where
    T: serde::Serialize,
```

**Description**: Creates a new `RpcBuilder` for calling a PostgreSQL function.

**Parameters**:
- `function_name`: Name of the PostgreSQL function to call
- `params`: Function arguments (must implement `Serialize`)

**Returns**: `RpcBuilder` for further chaining and execution

**Example**:
```rust
let builder = client.rpc("my_function", json!({ "param": "value" }));
```

### `RpcBuilder` Methods

#### Execution Methods

##### `execute()`
```rust
pub async fn execute(self) -> Result<Vec<Value>>
```

**Description**: Executes the RPC call expecting an array of results (SETOF records).

**Returns**: `Result<Vec<Value>>` - Array of JSON objects representing returned rows

**Use Case**: Functions that return multiple rows:
- `RETURNS SETOF table_name`
- `RETURNS TABLE(...)`
- `RETURNS SETOF record`

##### `execute_single()`
```rust
pub async fn execute_single(self) -> Result<Value>
```

**Description**: Executes the RPC call expecting a single result (scalar or single row).

**Returns**: `Result<Value>` - Single returned value (scalar, object, or null)

**Use Case**: Functions that return a single value:
- `RETURNS integer`, `RETURNS text`, etc. (scalar)
- `RETURNS table_name` (single row)
- `RETURNS record` (single composite)

##### `execute_void()`
```rust
pub async fn execute_void(self) -> Result<()>
```

**Description**: Executes the RPC call expecting no return value (void function).

**Returns**: `Result<()>` - Success if function executed without error

**Use Case**: Functions that return `void` or have no return value.

#### Filter Methods

All filter methods return `Self` for method chaining.

##### Equality Filters
```rust
pub fn eq(self, column: &str, value: &str) -> Self
pub fn neq(self, column: &str, value: &str) -> Self
```

**Description**: Filter results where column equals/does not equal value.

##### Comparison Filters
```rust
pub fn gt(self, column: &str, value: &str) -> Self
pub fn lt(self, column: &str, value: &str) -> Self
pub fn gte(self, column: &str, value: &str) -> Self
pub fn lte(self, column: &str, value: &str) -> Self
```

**Description**: Filter results using greater-than, less-than comparisons.

##### Set Operations
```rust
pub fn in_<T>(self, column: &str, values: &[T]) -> Self
where
    T: ToString,
```

**Description**: Filter results where column matches any of the given values.

##### Text Search
```rust
pub fn text_search(self, column: &str, value: &str) -> Self
```

**Description**: Full-text search on column.

##### Pagination & Sorting
```rust
pub fn limit(self, limit: usize) -> Self
pub fn offset(self, offset: usize) -> Self
pub fn range(self, from: usize, to: usize) -> Self
pub fn order(self, column: &str, ascending: bool) -> Self
```

**Description**: Control result set size, position, and ordering.

##### Column Selection
```rust
pub fn columns(self, columns: Vec<&str>) -> Self
```

**Description**: Select specific columns from results.

##### Counting
```rust
pub fn count(self) -> Self
```

**Description**: Request exact row count with results.

## 🚀 Usage Examples

### Basic RPC Calls

#### Scalar Function
```rust
use supabase_rs::SupabaseClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SupabaseClient::new(
        std::env::var("SUPABASE_URL")?,
        std::env::var("SUPABASE_KEY")?,
    )?;

    // Call a function that returns a single value
    let count = client.rpc("count_users", json!({}))
        .execute_single()
        .await?;
    
    println!("User count: {}", count.as_i64().unwrap());
    Ok(())
}
```

#### Set-Returning Function
```rust
// Get all active users
let users = client.rpc("get_active_users", json!({ "active": true }))
    .execute()
    .await?;

for user in users {
    println!("User: {} ({})", user["name"], user["email"]);
}
```

#### Void Function
```rust
// Clean up old sessions
client.rpc("cleanup_old_sessions", json!({ "days": 30 }))
    .execute_void()
    .await?;
```

### Advanced Filtering

#### Filtering Results
```rust
// Get active users over 18, sorted by name, limited to 10
let active_adults = client.rpc("get_users", json!({}))
    .eq("status", "active")
    .gte("age", "18")
    .order("name", true)
    .limit(10)
    .execute()
    .await?;
```

#### Complex Filtering
```rust
// Get posts in specific categories with text search
let posts = client.rpc("search_posts", json!({ "query": "rust" }))
    .in_("category", &["tech", "programming", "rust"])
    .text_search("content", "async await")
    .eq("published", "true")
    .order("created_at", false)
    .range(0, 24)  // Pagination: first 25 items
    .execute()
    .await?;
```

#### Column Selection
```rust
// Select only specific columns for efficiency
let user_summaries = client.rpc("get_users", json!({}))
    .columns(vec!["id", "name", "email"])
    .limit(50)
    .execute()
    .await?;
```

### Type-Safe Parameters

#### Using Structs
```rust
use serde::Serialize;

#[derive(Serialize)]
struct CreateUserParams {
    name: String,
    email: String,
    age: u32,
}

let params = CreateUserParams {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    age: 28,
};

let user_id = client.rpc("create_user", params)
    .execute_single()
    .await?;
```

#### Using Generated Types
```rust
// Assuming types were generated with type_gen
use supabase_types::rpc::CreateUserArgs;

let args = CreateUserArgs {
    name: "Bob".to_string(),
    email: "bob@example.com".to_string(),
    age: 32,
};

let result = client.rpc("create_user", args)
    .execute_single()
    .await?;
```

### Schema Support

#### Non-Public Schema
```rust
// Connect to a custom schema
let client = SupabaseClient::new(
    std::env::var("SUPABASE_URL")?,
    std::env::var("SUPABASE_KEY")?,
)?
.schema("custom_schema");

// Call function in custom schema
let result = client.rpc("custom_function", json!({}))
    .execute()
    .await?;
```

## 🔧 Type Generation Guide

### Overview
The type generation system automatically creates Rust structs for RPC function arguments, providing type safety and IDE support.

### Generating Types

#### Basic Generation
```rust
use supabase_rs::type_gen::generate_supabase_types;

#[tokio::main]
async fn main() {
    generate_supabase_types(
        "postgres",          // Database username
        "password",          // Database password
        true,                // Singularize struct names
        &["users", "posts"], // Tables to include (empty for all)
    ).await;
}
```

#### With Schema Support
```rust
use supabase_rs::type_gen::generate_supabase_types_with_schema;

#[tokio::main]
async fn main() {
    generate_supabase_types_with_schema(
        "postgres",
        "password",
        true,
        &["users", "posts"],
        "public",  // Schema name
    ).await;
}
```

### Generated Output

#### RPC Argument Structs
For a function `create_user(name text, age integer)`, the generator creates:

```rust
pub mod rpc {
    use serde::Serialize;

    #[derive(Debug, Serialize, Clone)]
    pub struct CreateUserArgs {
        pub name: String,
        pub age: i32,
    }
}
```

#### Usage with Generated Types
```rust
use supabase_types::rpc::CreateUserArgs;

let args = CreateUserArgs {
    name: "Alice".to_string(),
    age: 28,
};

let result = client.rpc("create_user", args)
    .execute_single()
    .await?;
```

### Type Mapping

PostgreSQL types are mapped to Rust types as follows:

| PostgreSQL Type | Rust Type | Notes |
|----------------|-----------|-------|
| `integer` | `i32` | |
| `bigint` | `i64` | |
| `smallint` | `i16` | |
| `text`, `varchar`, `char` | `String` | |
| `boolean` | `bool` | |
| `real`, `double precision` | `f64` | |
| `numeric`, `decimal` | `Decimal` | Requires `rust_decimal` |
| `timestamp without time zone` | `NaiveDateTime` | Requires `chrono` |
| `timestamp with time zone` | `DateTime<Utc>` | Requires `chrono` |
| `date` | `NaiveDate` | Requires `chrono` |
| `uuid` | `Uuid` | Requires `uuid` |
| `json`, `jsonb` | `Value` | Requires `serde_json` |

### Integration with Existing Code

The generator automatically:
1. Creates `src/supabase_types.rs` with all generated types
2. Adds `pub mod supabase_types;` to `src/lib.rs` if not present
3. Organizes RPC argument structs in a `rpc` module

## 🧪 Testing Guide

### Test Setup

#### Environment Configuration
Create a `.env` file for testing:
```env
SUPABASE_URL=https://your-test-project.supabase.co
SUPABASE_KEY=your-test-key
SUPABASE_RS_NO_NIGHTLY_MSG=true
```

#### Database Setup
Run the RPC test setup script:
```sql
-- src/tests/setup_rpc.sql
-- This creates test functions for RPC testing
```

### Running Tests

#### All Tests
```bash
cargo test --features rpc
```

#### RPC-Specific Tests
```bash
cargo test test_rpc --features rpc
cargo test test_rpc_single --features rpc
cargo test test_rpc_void --features rpc
cargo test test_rpc_with_filters --features rpc
cargo test test_rpc_type_generation --features rpc
```

#### With Output
```bash
cargo test --features rpc -- --nocapture
```

### Test Categories

#### 1. Basic RPC Functionality
Tests basic RPC calls with different return types:
- Scalar functions (`test_echo`)
- Set-returning functions (`test_get_test_rows`)
- Void functions (`test_void_func`)

#### 2. Filter Integration
Tests RPC with filter methods:
- Equality filters (`.eq()`)
- Range filters (`.gte()`, `.lte()`)
- Pagination (`.limit()`, `.offset()`)
- Sorting (`.order()`)

#### 3. Type Generation
Tests type generation integration:
- Parameter struct generation
- Default parameter handling
- Type mapping validation

#### 4. Error Handling
Tests error scenarios:
- Non-existent functions
- Invalid parameters
- Schema errors
- Network failures

### Writing Custom Tests

#### Example Test Structure
```rust
#[tokio::test]
async fn test_custom_rpc() -> Result<(), String> {
    let client = create_test_client();
    
    // Test scalar function
    let result = client.rpc("test_add_numbers", json!({"a": 5, "b": 3}))
        .execute_single()
        .await
        .map_err(|e| format!("RPC failed: {:?}", e))?;
    
    assert_eq!(result.as_i64().unwrap(), 8);
    Ok(())
}
```

#### Test Client Creation
```rust
fn create_test_client() -> SupabaseClient {
    SupabaseClient::new(
        std::env::var("SUPABASE_URL").expect("SUPABASE_URL required"),
        std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY required"),
    ).expect("Failed to create client")
}
```

## 🚚 Migration Guide

### For Existing Users

#### From Previous Versions
RPC support is additive and non-breaking. Existing code continues to work without modification.

#### Enabling RPC
1. Update `Cargo.toml`:
   ```toml
   supabase_rs = { version = "0.5.1", features = ["rpc"] }
   ```

2. Update imports if using generated types:
   ```rust
   // Before (if using custom types)
   use my_types::UserParams;
   
   // After (using generated types)
   use supabase_types::rpc::CreateUserArgs;
   ```

#### Code Migration Examples

##### Before (Manual JSON)
```rust
let params = json!({
    "name": "Alice",
    "email": "alice@example.com",
    "age": 28
});

// Manual HTTP call
let response = client.client.post("/rest/v1/rpc/create_user")
    .json(&params)
    .send()
    .await?;
```

##### After (Using RPC Builder)
```rust
use supabase_types::rpc::CreateUserArgs;

let params = CreateUserArgs {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    age: 28,
};

let result = client.rpc("create_user", params)
    .execute_single()
    .await?;
```

### Breaking Changes

#### None
The RPC implementation introduces no breaking changes to existing APIs.

### Deprecations

#### None
No APIs have been deprecated.

## ⚠️ Known Limitations

### Current Limitations

#### 1. Parameter Mode Support
- **Status**: Partial
- **Details**: Only `IN` and `INOUT` parameters are fully supported. `OUT` parameters are excluded from generated structs as they are not passed as arguments.
- **Workaround**: For functions with `OUT` parameters, use manual JSON parameter construction.

#### 2. Default Parameter Detection
- **Status**: Limited
- **Details**: The type generator cannot reliably detect default parameter values from `information_schema.parameters`.
- **Workaround**: Manually handle default parameters in application code.

#### 3. Complex Return Types
- **Status**: Basic support
- **Details**: Complex PostgreSQL types (arrays, composites, domains) are mapped to `String` or `Value`.
- **Workaround**: Use `serde_json::Value` and manual parsing for complex types.

#### 4. Transaction Support
- **Status**: Not supported
- **Details**: RPC calls cannot be executed within database transactions via the current API.
- **Workaround**: Use PostgREST's transaction support directly or implement at the application level.

#### 5. Performance Considerations
- **Status**: Optimized for common cases
- **Details**: Large result sets may impact memory usage. Streaming responses are not supported.
- **Workaround**: Use pagination (`.limit()`, `.range()`) and filtering to manage result sizes.

#### 6. Error Detail Propagation
- **Status**: Basic
- **Details**: PostgreSQL error details may be lost in HTTP error responses.
- **Workaround**: Check PostgREST logs or implement custom error handling for detailed diagnostics.

### Future Improvements

#### Planned Enhancements
1. **Streaming Support**: Add support for streaming large result sets
2. **Enhanced Type Mapping**: Better support for PostgreSQL arrays and custom types
3. **Transaction Integration**: Support for transactional RPC calls
4. **Batch Operations**: Execute multiple RPC calls in a single request
5. **Advanced Parameter Handling**: Support for variadic functions and polymorphic types

#### Community Contributions
These limitations represent opportunities for community contributions. Refer to the [Contributing Guide](CONTRIBUTING.md) for information on how to help improve the RPC implementation.

## 📋 Summary

The RPC implementation in `supabase_rs` provides a comprehensive, type-safe interface for calling PostgreSQL functions via PostgREST. Key features include:

- **Fluent API**: Consistent with existing SDK patterns
- **Type Safety**: Compile-time validation through generated types
- **Flexible Execution**: Support for void, scalar, single row, and set-returning functions
- **Advanced Filtering**: Post-execution filtering using standard PostgREST query parameters
- **Schema Support**: Multi-schema support through proper header management
- **Comprehensive Testing**: Full test coverage across all execution modes

### Getting Started Checklist
- [ ] Enable the `rpc` feature in `Cargo.toml`
- [ ] Set up environment variables for your Supabase project
- [ ] Generate types for your database functions (optional but recommended)
- [ ] Start calling RPC functions using the fluent builder API

### Additional Resources
- [Supabase Documentation](https://supabase.io/docs) - Official Supabase documentation
- [PostgREST API Reference](https://postgrest.org/en/stable/api.html) - Detailed API reference
- [GitHub Repository](https://github.com/floris-xlx/supabase_rs) - Source code and issue tracking
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines for the project

### Support
For issues, feature requests, or questions:
1. Check the [GitHub Issues](https://github.com/floris-xlx/supabase_rs/issues) for existing discussions
2. Review the [RPC Design Documents](RPC_DESIGN_V2.md) for implementation details
3. Submit a new issue with detailed reproduction steps if needed

---

*Documentation last updated: January 2026*
*RPC Implementation Version: 1.0*
*Supabase RS Version: 0.5.1*