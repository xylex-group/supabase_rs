# supabase_rs

An unofficial, lightweight Rust SDK for interacting with the Supabase REST and GraphQL APIs. This SDK provides a clean, chainable query-builder interface with comprehensive CRUD operations, advanced filtering capabilities, and optional modules for Storage and Realtime functionality.

## Key Features

- **Pure REST API by default** with optional nightly GraphQL support
- **Fluent Query Builder** for intuitive filtering, ordering, limiting, and text search
- **Complete CRUD Operations** with Insert, Update, Upsert, and Delete helpers
- **Type-Safe Operations** with Rust's strong type system
- **Connection Pooling** built-in with `reqwest::Client`
- **Feature-Flagged Modules** for Storage and Realtime (opt-in)
- **Comprehensive Error Handling** with detailed error types
- **Async/Await Support** throughout the entire API
- **Clone-Friendly Client** for multi-threaded applications

## Table of Contents

- [Installation](#installation)
- [Features and Flags](#features-and-flags)
- [Quickstart](#quickstart)
- [Database Operations](#database-operations)
  - [Basic CRUD](#basic-crud)
  - [Advanced Querying](#advanced-querying)
  - [Bulk Operations](#bulk-operations)
  - [Error Handling](#error-handling)
- [Storage Operations](#storage-operations)
- [GraphQL Support](#graphql-support)
- [Performance & Best Practices](#performance--best-practices)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Migration Guide](#migration-guide)
- [Contributing](#contributing)
- [Contributors](#contributors)

# Installation

Add the crate to your project using Cargo:

```toml
[dependencies]
supabase_rs = "0.4.14"

# With optional features
supabase_rs = { version = "0.4.14", features = ["storage", "rustls"] }
```

### Feature Combinations

```toml
# Basic REST API only (default)
supabase_rs = "0.4.14"

# With Storage support
supabase_rs = { version = "0.4.14", features = ["storage"] }

# With rustls instead of OpenSSL (recommended for cross-platform)
supabase_rs = { version = "0.4.14", features = ["rustls"] }

# With experimental GraphQL support (nightly)
supabase_rs = { version = "0.4.14", features = ["nightly"] }

# All features enabled
supabase_rs = { version = "0.4.14", features = ["storage", "rustls", "nightly"] }
```

### Environment Setup

Create a `.env` file in your project root:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-anon-or-service-role-key

# Optional: Disable nightly warning messages
SUPABASE_RS_NO_NIGHTLY_MSG=true

# Optional: Use alternative endpoint format
SUPABASE_RS_DONT_REST_V1_URL=false
```

> **Tip**: Use your service role key for server-side applications and anon key for client-side applications with Row Level Security (RLS) enabled.

## Features and Flags

### Core Features

| Feature | Description | Stability | Use Case |
|---------|-------------|-----------|----------|
| **Default** | REST API operations with native TLS | ✅ Stable | Production applications |
| `storage` | File upload/download operations | ✅ Stable | Applications with file management |
| `rustls` | Use rustls instead of OpenSSL | ✅ Stable | Cross-platform deployments, Alpine Linux |
| `nightly` | Experimental GraphQL support | ⚠️ Experimental | Advanced querying, development |

### Feature Flag Details

- **`storage`**: Enables the Storage module for file operations with Supabase Storage buckets
- **`rustls`**: Replaces OpenSSL with rustls for TLS connections (recommended for Docker/Alpine)
- **`nightly`**: Unlocks GraphQL query capabilities (experimental, may have breaking changes)

### Nightly Feature Configuration

The nightly feature shows warning messages by default. To disable them:

```env
SUPABASE_RS_NO_NIGHTLY_MSG=true
```

> **⚠️ Warning**: Nightly features are experimental and may introduce breaking changes without notice. Use with caution in production environments.

## Quickstart

### Basic Client Setup

```rust
use supabase_rs::SupabaseClient;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize the Supabase client
    let client = SupabaseClient::new(
        std::env::var("SUPABASE_URL")?,
        std::env::var("SUPABASE_KEY")?,
    )?;
    
    // The client is ready to use!
    println!("✅ Supabase client initialized successfully");
    
    Ok(())
}
```

### Helper Function for Reusable Client

```rust
use supabase_rs::SupabaseClient;

/// Creates a configured Supabase client instance
/// 
/// # Panics
/// Panics if SUPABASE_URL or SUPABASE_KEY environment variables are not set
fn create_client() -> SupabaseClient {
    SupabaseClient::new(
        std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set"),
        std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY must be set"),
    ).expect("Failed to create Supabase client")
}
```

### Multi-threaded Usage

```rust
use supabase_rs::SupabaseClient;
use std::sync::Arc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(create_client());
    
    // Clone is cheap - shares the underlying connection pool
    let client_clone = Arc::clone(&client);
    
    let handle = task::spawn(async move {
        // Use client_clone in another task
        let _result = client_clone.select("users").execute().await;
    });
    
    handle.await?;
    Ok(())
}
```

## Database Operations

### Basic CRUD

#### Insert Operations

```rust
use serde_json::json;
use supabase_rs::SupabaseClient;

let client = create_client();

// Basic insert - returns the new row's ID
let id = client.insert("pets", json!({
    "name": "scooby",
    "breed": "great_dane",
    "age": 7
})).await?;

println!("Inserted pet with ID: {}", id);
```

#### Insert with Unique Constraint Checking

```rust
// Insert only if the record doesn't already exist
// Checks all provided fields for uniqueness
let id = client.insert_if_unique("users", json!({
    "email": "user@example.com",
    "username": "john_doe"
})).await?;

// Returns error if a user with this email OR username already exists
```

#### Bulk Insert Operations

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Pet {
    name: String,
    breed: String,
    age: i32,
}

let pets = vec![
    Pet { name: "Buddy".to_string(), breed: "golden_retriever".to_string(), age: 3 },
    Pet { name: "Luna".to_string(), breed: "border_collie".to_string(), age: 2 },
];

// Insert multiple records in a single request
client.bulk_insert("pets", pets).await?;
```

#### Update Operations

```rust
// Update by ID (default)
client.update("pets", "123", json!({
    "name": "scooby-doo",
    "age": 8
})).await?;

// Update by custom column
client.update_with_column_name(
    "users",
    "email",           // Column to match on
    "user@example.com", // Value to match
    json!({ "last_login": "2024-01-15T10:30:00Z" })
).await?;
```

#### Upsert Operations

```rust
// Insert or update if exists
client.upsert("pets", "123", json!({
    "name": "scooby-doo",
    "breed": "great_dane"
})).await?;

// Upsert without predefined ID (uses Supabase's conflict resolution)
client.upsert_without_defined_key("settings", json!({
    "user_id": "456",
    "theme": "dark",
    "notifications": true
})).await?;
```

#### Delete Operations

```rust
// Delete by ID
client.delete("pets", "123").await?;

// Delete by custom column
client.delete_without_defined_key("sessions", "token", "abc123").await?;
```

### Advanced Querying

#### Complex Filtering

```rust
use serde_json::Value;

let client = create_client();

// Multiple filters with chaining
let adult_pets: Vec<Value> = client
    .select("pets")
    .gte("age", "2")                    // Age >= 2
    .neq("breed", "unknown")            // Breed != "unknown"
    .text_search("description", "friendly") // Full-text search
    .limit(20)
    .order("created_at", false)         // Newest first
    .execute()
    .await?;
```

#### Column Selection and Pagination

```rust
// Select specific columns with pagination
let users: Vec<Value> = client
    .from("users")
    .columns(vec!["id", "email", "created_at"])
    .range(0, 49)                       // Get first 50 records (0-49 inclusive)
    .order("created_at", true)          // Oldest first
    .execute()
    .await?;

// Using offset-based pagination
let page_2: Vec<Value> = client
    .from("users")
    .columns(vec!["id", "email"])
    .limit(25)
    .offset(25)                         // Skip first 25 records
    .execute()
    .await?;
```

#### Advanced Filter Operations

```rust
// IN operator for multiple values
let specific_breeds: Vec<Value> = client
    .select("pets")
    .in_("breed", &["golden_retriever", "labrador", "poodle"])
    .execute()
    .await?;

// Null checking
let pets_without_age: Vec<Value> = client
    .select("pets")
    .eq("age", "is.null")
    .execute()
    .await?;
```

### Bulk Operations

#### Batch Processing

```rust
use futures::future::try_join_all;

// Process multiple operations concurrently
let client = create_client();
let operations = vec![
    client.select("users").limit(100).execute(),
    client.select("pets").limit(100).execute(),
    client.select("orders").limit(100).execute(),
];

let results = try_join_all(operations).await?;
println!("Fetched {} datasets", results.len());
```

### Error Handling

#### Comprehensive Error Management

```rust
use serde_json::json;

match client.insert("users", json!({ "email": "test@example.com" })).await {
    Ok(id) => {
        println!("✅ User created with ID: {}", id);
    },
    Err(error) => {
        if error.contains("409") {
            println!("⚠️ User already exists with this email");
            // Handle duplicate entry
        } else if error.contains("401") {
            println!("🔐 Authentication failed - check your API key");
        } else if error.contains("403") {
            println!("🚫 Insufficient permissions for this operation");
        } else {
            println!("❌ Unexpected error: {}", error);
        }
    }
}
```

#### Retry Logic Example

```rust
use tokio::time::{sleep, Duration};

async fn insert_with_retry(
    client: &SupabaseClient,
    table: &str,
    data: serde_json::Value,
    max_retries: u32
) -> Result<String, String> {
    for attempt in 1..=max_retries {
        match client.insert(table, data.clone()).await {
            Ok(id) => return Ok(id),
            Err(err) if attempt < max_retries => {
                println!("Attempt {} failed: {}. Retrying...", attempt, err);
                sleep(Duration::from_millis(1000 * attempt as u64)).await;
            },
            Err(err) => return Err(format!("Failed after {} attempts: {}", max_retries, err)),
        }
    }
    unreachable!()
}
```

### Count Operations

> ** Performance Note**: Count operations are expensive and can be slow on large tables. Use sparingly and consider caching results.

```rust
// Count all records (expensive)
let total_users = client
    .select("users")
    .count()
    .execute()
    .await?;

// Count with filters (more efficient)
let active_users = client
    .select("users")
    .eq("status", "active")
    .count()
    .execute()
    .await?;
```

## Storage Operations

> ** Requirement**: Enable the `storage` feature in your `Cargo.toml`

The Storage module provides comprehensive file management capabilities for Supabase Storage buckets.

### File Download Operations

```rust
use supabase_rs::storage::SupabaseStorage;

// Initialize storage client
let storage = SupabaseStorage {
    supabase_url: std::env::var("SUPABASE_URL").unwrap(),
    bucket_name: "avatars".to_string(),
    filename: "user-123-avatar.jpg".to_string(),
};

// Download file to memory
let file_bytes = storage.download().await?;
println!("Downloaded {} bytes", file_bytes.len());

// Download file directly to disk
storage.save("./downloads/avatar.jpg").await?;
```

### Advanced Storage Patterns

```rust
// Batch download multiple files
let files = vec!["file1.jpg", "file2.png", "file3.pdf"];
let mut downloads = Vec::new();

for filename in files {
    let storage = SupabaseStorage {
        supabase_url: env::var("SUPABASE_URL").unwrap(),
        bucket_name: "documents".to_string(),
        filename: filename.to_string(),
    };
    downloads.push(storage.download());
}

let results = try_join_all(downloads).await?;
```

## GraphQL Support

> **⚠️ Experimental**: Enable the `nightly` feature for GraphQL support. This is experimental and not production-ready.

GraphQL and REST operations can be mixed using the same client instance.

### Basic GraphQL Query

```rust
use supabase_rs::graphql::{Request, RootTypes};
use serde_json::json;

let client = create_client();

let graphql_request = Request::new(
    client,
    json!({
        "query": r#"
            {
                usersCollection(first: 10) {
                    edges {
                        node {
                            id
                            email
                            created_at
                        }
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        "#
    }),
    RootTypes::Query
);

let response = graphql_request.send().await?;
println!("GraphQL Response: {:#?}", response);
```

### GraphQL with Variables

```rust
let query_with_variables = Request::new(
    client,
    json!({
        "query": r#"
            query GetUsersByAge($minAge: Int!) {
                usersCollection(filter: { age: { gte: $minAge } }) {
                    edges {
                        node {
                            id
                            email
                            age
                        }
                    }
                }
            }
        "#,
        "variables": {
            "minAge": 18
        }
    }),
    RootTypes::Query
);
```

### Mixing REST and GraphQL

```rust
// Use REST for simple operations
let new_user_id = client.insert("users", json!({
    "email": "newuser@example.com",
    "age": 25
})).await?;

// Use GraphQL for complex relational queries
let user_with_posts = Request::new(
    client.clone(),
    json!({
        "query": format!(r#"
            {{
                usersCollection(filter: {{ id: {{ eq: {} }} }}) {{
                    edges {{
                        node {{
                            id
                            email
                            postsCollection {{
                                edges {{
                                    node {{
                                        title
                                        content
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        "#, new_user_id)
    }),
    RootTypes::Query
).send().await?;
```

## Performance & Best Practices

### Client Management

```rust
// ✅ Good: Reuse client instances (they're cheap to clone)
let client = create_client();
let client_clone = client.clone(); // Shares connection pool

// ❌ Avoid: Creating new clients repeatedly
// let client1 = SupabaseClient::new(...)?; // Don't do this in loops
```

### Query Optimization

```rust
// ✅ Good: Use specific column selection
let users = client
    .from("users")
    .columns(vec!["id", "email"])  // Only fetch needed columns
    .limit(100)                    // Always use reasonable limits
    .execute()
    .await?;

// ✅ Good: Use range for pagination (more efficient than offset)
let page = client
    .from("users")
    .range(0, 99)                  // Get 100 records
    .execute()
    .await?;

// ⚠️ Use sparingly: Count operations are expensive
let count = client.select("users").count().execute().await?;
```

### Batch Operations

```rust
// ✅ Good: Use bulk_insert for multiple records
client.bulk_insert("logs", vec![
    json!({"level": "info", "message": "Started"}),
    json!({"level": "info", "message": "Processing"}),
]).await?;

// ❌ Avoid: Individual inserts in loops
// for item in items {
//     client.insert("table", item).await?; // Inefficient
// }
```

### Connection Pool Configuration

```rust
// For high-throughput applications, consider custom reqwest client
use reqwest::ClientBuilder;
use std::time::Duration;

let http_client = ClientBuilder::new()
    .pool_max_idle_per_host(10)
    .timeout(Duration::from_secs(30))
    .build()?;

// Note: Custom client configuration requires modifying SupabaseClient::new()
```

## Testing

This repository includes comprehensive test coverage with both integration and unit tests.

### Test Categories

- **Integration Tests**: Test against live Supabase instances
- **Unit Tests**: Test individual components in isolation
- **Performance Tests**: Benchmark query performance

### Running Tests

```bash
# Run all tests (requires SUPABASE_URL and SUPABASE_KEY)
cargo test

# Run only unit tests (no network required)
cargo test unit_

# Run specific test module
cargo test select_

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

### Test Environment Setup

Create a `.env.test` file for testing:

```env
SUPABASE_URL=https://your-test-project.supabase.co
SUPABASE_KEY=your-test-key
SUPABASE_RS_NO_NIGHTLY_MSG=true
```

### Writing Custom Tests

```rust
use supabase_rs::SupabaseClient;
use serde_json::json;

#[tokio::test]
async fn test_user_operations() -> Result<(), String> {
    let client = SupabaseClient::new(
        std::env::var("SUPABASE_URL").unwrap(),
        std::env::var("SUPABASE_KEY").unwrap(),
    ).unwrap();
    
    // Test insert
    let user_id = client.insert("users", json!({
        "email": "test@example.com",
        "name": "Test User"
    })).await?;
    
    // Test select
    let users = client
        .select("users")
        .eq("id", &user_id)
        .execute()
        .await?;
    
    assert!(!users.is_empty());
    
    // Cleanup
    client.delete("users", &user_id).await?;
    
    Ok(())
}
```

## Troubleshooting

### Common Issues and Solutions

#### Authentication Errors

```
Error: 401 Unauthorized
```

**Solutions:**
- Verify your `SUPABASE_URL` and `SUPABASE_KEY` are correct
- Ensure you're using the right key type (anon vs service role)
- Check if your API key has expired

#### Permission Errors

```
Error: 403 Forbidden
```

**Solutions:**
- Review your Row Level Security (RLS) policies
- Ensure your API key has sufficient permissions
- Check if the table/operation requires service role key

#### Connection Issues

```
Error: Connection timeout / Network error
```

**Solutions:**
- Check your internet connection
- Verify the Supabase URL is accessible
- Consider increasing timeout values
- Check if you're behind a corporate firewall

#### Duplicate Entry Errors

```
Error 409: Duplicate entry
```

**Solutions:**
- Use `insert_if_unique()` instead of `insert()`
- Check your unique constraints
- Handle duplicates gracefully in your application logic

### Performance Issues

#### Slow Queries

**Symptoms:**
- Queries taking longer than expected
- High memory usage

**Solutions:**
```rust
// Use column selection to reduce data transfer
let users = client
    .from("users")
    .columns(vec!["id", "email"])  // Only fetch needed columns
    .limit(100)                    // Always limit results
    .execute()
    .await?;

// Use pagination instead of fetching all records
let page = client
    .from("large_table")
    .range(0, 999)                 // Get 1000 records at a time
    .execute()
    .await?;
```

#### Memory Usage

**High memory consumption solutions:**
- Use streaming for large datasets
- Implement pagination
- Process data in batches
- Use specific column selection

### Debugging

#### Enable Debug Logging

```rust
// Add to your Cargo.toml
[dependencies]
env_logger = "0.10"

// In your main function
env_logger::init();
```

#### Nightly Feature Debugging

```env
# Enable detailed endpoint logging
SUPABASE_RS_NO_NIGHTLY_MSG=false
```

## 📈 Migration Guide

### From v0.3.x to v0.4.x

#### Breaking Changes

1. **Method Signatures**: Some methods now return `Result<T, String>` instead of `Result<T, Error>`
2. **Client Creation**: `new()` method now returns `Result<SupabaseClient, ErrorTypes>`

#### Migration Steps

```rust
// Old (v0.3.x)
let client = SupabaseClient::new(url, key); // Could panic

// New (v0.4.x)
let client = SupabaseClient::new(url, key)?; // Returns Result
```

### From v0.2.x to v0.3.x

#### Query Builder Changes

```rust
// Old
client.select("table").filter("column", "value")

// New
client.select("table").eq("column", "value")
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/floris-xlx/supabase_rs.git
   cd supabase_rs
   ```

2. **Set up environment**
   ```bash
   cp .env.example .env
   # Edit .env with your Supabase credentials
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Check formatting and linting**
   ```bash
   cargo fmt
   cargo clippy
   ```

### Contribution Guidelines

- **Code Style**: Follow Rust standard formatting (`cargo fmt`)
- **Documentation**: Add comprehensive docs for all public APIs
- **Testing**: Include tests for new functionality
- **Performance**: Consider performance implications of changes
- **Compatibility**: Maintain backward compatibility when possible

### Areas for Contribution

- **Core Features**: Improve existing CRUD operations
- **Storage**: Enhance file upload capabilities  
- **GraphQL**: Stabilize GraphQL support
- **Documentation**: Improve examples and guides
- **Testing**: Add more comprehensive test coverage
- **Performance**: Optimize query building and execution

## Contributors

Special thanks to all contributors who have helped improve this project:

- [**Hadi**](https://github.com/hadi-xlx) — Improved & fixed the schema-to-type generator
- [**Izyuumi**](https://github.com/izyuumi) — Improved row ID routing with updating methods  
- [**koya1616**](https://github.com/koya1616) — README fixes and documentation improvements
- [**strykejern**](https://github.com/strykejern) — Refactoring & warning fixes

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- [API Documentation](https://docs.rs/supabase_rs)
- [Issue Tracker](https://github.com/floris-xlx/supabase_rs/issues)
- [Changelog](CHANGELOG.md)
- [Supabase Documentation](https://supabase.io/docs)

---

<div align="center">
  <strong>Built with ❤️ for the Rust community</strong>
</div>

