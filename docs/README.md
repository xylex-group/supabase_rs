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
