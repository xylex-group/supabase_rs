//! # Supabase SDK for Rust
//!
//! An unofficial, lightweight Rust SDK for [Supabase](https://supabase.io/) that provides a clean,
//! type-safe interface for interacting with Supabase's REST and GraphQL APIs.
//!
//! This crate focuses on developer experience with a fluent, chainable API design that feels natural
//! in Rust while maintaining compatibility with Supabase's PostgREST conventions.
//!
//! ## 🚀 Core Features
//!
//! ### Database Operations
//! - **[`Insert`](insert)**: Add new rows with automatic ID generation and conflict handling
//! - **[`Insert if unique`](insert)**: Conditional inserts with uniqueness validation
//! - **[`Update`](update)**: Modify existing rows by ID or custom columns
//! - **[`Upsert`](update)**: Insert or update with conflict resolution
//! - **[`Select`](select)**: Retrieve data with advanced filtering and pagination
//! - **[`Delete`](delete)**: Remove rows by ID or custom criteria
//!
//! ### Query Building
//! - **Fluent API**: Chain filters, sorts, and pagination naturally
//! - **Type Safety**: Leverage Rust's type system for compile-time guarantees
//! - **Performance**: Built-in connection pooling and efficient query construction
//!
//! ### Advanced Features
//! - **[`Storage`](storage)**: File upload/download operations (feature-gated)
//! - **[`GraphQL`](graphql)**: Advanced querying with GraphQL (experimental)
//! - **[`Realtime`](realtime)**: Live data subscriptions (planned)
//!
//! ## 🎯 Feature Flags
//!
//! | Feature | Description | Stability |
//! |---------|-------------|-----------|
//! | `storage` | File operations with Supabase Storage | ✅ Stable |
//! | `rustls` | Use rustls instead of OpenSSL for TLS | ✅ Stable |
//! | `nightly` | Experimental GraphQL support | ⚠️ Experimental |
//!
//! ### Feature Flag Details
//!
//! - **`storage`**: Enables the [`storage`] module for file upload/download operations
//! - **`rustls`**: Forces the HTTP client to use `rustls` instead of OpenSSL (recommended for Alpine Linux)
//! - **`nightly`**: Unlocks experimental GraphQL capabilities with detailed debugging
//!
//! ## ⚠️ Nightly Features
//!
//! Nightly features are experimental and may introduce breaking changes without notice.
//! Use with caution in production environments.
//!
//! To disable nightly warning messages:
//! ```env
//! SUPABASE_RS_NO_NIGHTLY_MSG=true
//! ```
//!
//! ## 🏗️ Architecture Overview
//!
//! The SDK is built around a central [`SupabaseClient`] that manages:
//! - HTTP connection pooling via [`reqwest::Client`]
//! - Authentication headers and API key management
//! - Endpoint URL construction and routing
//! - Request/response serialization
//!
//! ### Module Organization
//!
//! ```text
//! supabase_rs/
//! ├── lib.rs           # Main client and public API
//! ├── insert.rs        # Insert operations and bulk operations
//! ├── update.rs        # Update and upsert operations
//! ├── select.rs        # Query execution and response handling
//! ├── delete.rs        # Delete operations
//! ├── query_builder/   # Fluent query building
//! │   ├── builder.rs   # QueryBuilder implementation
//! │   ├── filter.rs    # Filter operations (eq, gt, lt, etc.)
//! │   └── sort.rs      # Sorting and ordering
//! ├── storage/         # File operations (feature-gated)
//! ├── graphql/         # GraphQL support (experimental)
//! ├── errors.rs        # Error types and handling
//! └── request/         # HTTP request utilities
//! ```
//!
//! ## 📦 Installation
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! supabase_rs = "0.4.14"
//!
//! # With optional features
//! supabase_rs = { version = "0.4.14", features = ["storage", "rustls"] }
//! ```
//!
//! ## 🚀 Quick Start
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize client
//!     let client = SupabaseClient::new(
//!         std::env::var("SUPABASE_URL")?,
//!         std::env::var("SUPABASE_KEY")?,
//!     )?;
//!
//!     // Insert data
//!     let id = client.insert("users", json!({
//!         "name": "John Doe",
//!         "email": "john@example.com"
//!     })).await?;
//!
//!     // Query data
//!     let users = client
//!         .select("users")
//!         .eq("name", "John Doe")
//!         .limit(10)
//!         .execute()
//!         .await?;
//!
//!     println!("Found {} users", users.len());
//!     Ok(())
//! }
//! ```
//!
//! ## 📚 Core Concepts
//!
//! ### Client Initialization
//!
//! The [`SupabaseClient`] is the main entry point for all operations. It's designed to be:
//! - **Clone-friendly**: Cheap to clone, shares connection pool
//! - **Thread-safe**: Can be used across async tasks
//! - **Connection-pooled**: Reuses HTTP connections efficiently
//!
//! ### Query Builder Pattern
//!
//! The SDK uses a fluent query builder pattern for constructing complex queries:
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::Value;
//!
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! let results: Vec<Value> = client
//!     .from("posts")                    // Start with table
//!     .columns(vec!["id", "title"])     // Select specific columns
//!     .eq("status", "published")        // Add filters
//!     .gte("created_at", "2024-01-01")  // Multiple filters
//!     .order("created_at", false)       // Sort by date, newest first
//!     .limit(20)                        // Limit results
//!     .execute()                        // Execute query
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Error Handling Philosophy
//!
//! The SDK uses `Result<T, String>` for most operations to provide clear error messages:
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! match client.insert("users", json!({"email": "test@example.com"})).await {
//!     Ok(id) => println!("Created user with ID: {}", id),
//!     Err(err) => {
//!         if err.contains("409") {
//!             println!("User already exists");
//!         } else {
//!             println!("Unexpected error: {}", err);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 🔐 Authentication & Setup
//!
//! The SDK requires two pieces of information to connect to your Supabase project:
//! - **Project URL**: Your unique Supabase project URL
//! - **API Key**: Either your anon key (client-side) or service role key (server-side)
//!
//! ### Environment Configuration
//!
//! Set up your environment variables in a `.env` file:
//! ```env
//! SUPABASE_URL=https://your-project.supabase.co
//! SUPABASE_KEY=your-anon-or-service-role-key
//! ```
//!
//! ### Key Types and Usage
//!
//! | Key Type | Use Case | Permissions |
//! |----------|----------|-------------|
//! | **Anon Key** | Client-side apps | Respects RLS policies |
//! | **Service Role** | Server-side apps | Bypasses RLS, full access |
//!
//! ## 📖 Complete Examples
//!
//! ### Client Initialization Patterns
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use dotenv::dotenv;
//!
//! // Basic initialization with error handling
//! fn create_client() -> Result<SupabaseClient, Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     
//!     let client = SupabaseClient::new(
//!         std::env::var("SUPABASE_URL")?,
//!         std::env::var("SUPABASE_KEY")?,
//!     )?;
//!     
//!     Ok(client)
//! }
//!
//! // For applications that need shared client instances
//! use std::sync::Arc;
//!
//! fn create_shared_client() -> Arc<SupabaseClient> {
//!     let client = SupabaseClient::new(
//!         std::env::var("SUPABASE_URL").expect("SUPABASE_URL required"),
//!         std::env::var("SUPABASE_KEY").expect("SUPABASE_KEY required"),
//!     ).expect("Failed to create Supabase client");
//!     
//!     Arc::new(client)
//! }
//! ```
//!
//! ### Insert Operations
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::json;
//!
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! // Basic insert with automatic ID generation
//! let user_id = client.insert("users", json!({
//!     "name": "Alice Johnson",
//!     "email": "alice@example.com",
//!     "age": 28
//! })).await?;
//!
//! println!("Created user with ID: {}", user_id);
//!
//! // Insert with uniqueness check (prevents duplicates)
//! let unique_id = client.insert_if_unique("users", json!({
//!     "email": "unique@example.com",
//!     "username": "unique_user"
//! })).await?;
//!
//! // Bulk insert for multiple records
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct NewUser {
//!     name: String,
//!     email: String,
//! }
//!
//! let users = vec![
//!     NewUser { name: "Bob".to_string(), email: "bob@example.com".to_string() },
//!     NewUser { name: "Carol".to_string(), email: "carol@example.com".to_string() },
//! ];
//!
//! client.bulk_insert("users", users).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Update & Upsert Operations
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! // Update existing record by ID
//! client.update("users", "123", json!({
//!     "name": "Alice Smith",
//!     "last_login": "2024-01-15T10:30:00Z"
//! })).await?;
//!
//! // Update by custom column
//! client.update_with_column_name(
//!     "users",
//!     "email",                    // Column to match
//!     "alice@example.com",        // Value to match
//!     json!({ "verified": true })
//! ).await?;
//!
//! // Upsert (insert or update if exists)
//! client.upsert("settings", "user_123", json!({
//!     "theme": "dark",
//!     "notifications": true
//! })).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Query Operations
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::Value;
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! // Basic select with filtering
//! let active_users: Vec<Value> = client
//!     .select("users")
//!     .eq("status", "active")
//!     .order("created_at", false)     // Newest first
//!     .limit(50)
//!     .execute()
//!     .await?;
//!
//! // Select specific columns (more efficient)
//! let user_emails: Vec<Value> = client
//!     .from("users")
//!     .columns(vec!["id", "email", "name"])
//!     .gte("age", "18")               // Adults only
//!     .execute()
//!     .await?;
//!
//! // Complex filtering with multiple conditions
//! let filtered_posts: Vec<Value> = client
//!     .select("posts")
//!     .eq("published", "true")
//!     .in_("category", &["tech", "science", "programming"])
//!     .text_search("content", "rust programming")
//!     .limit(10)
//!     .execute()
//!     .await?;
//!
//! // Pagination using range (recommended)
//! let page_1: Vec<Value> = client
//!     .from("articles")
//!     .range(0, 24)                   // First 25 items (0-24 inclusive)
//!     .order("published_at", false)
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Delete Operations
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! // Delete by ID
//! client.delete("users", "123").await?;
//!
//! // Delete by custom column
//! client.delete_without_defined_key("sessions", "token", "abc123").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Count Operations
//!
//! > **⚠️ Performance Warning**: Count operations can be expensive on large tables.
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # async fn example(client: SupabaseClient) -> Result<(), String> {
//! // Count all records (expensive)
//! let total = client
//!     .select("users")
//!     .count()
//!     .execute()
//!     .await?;
//!
//! // Count with filters (more efficient)
//! let active_count = client
//!     .select("users")
//!     .eq("status", "active")
//!     .count()
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🔗 Module Documentation
//!
//! For detailed documentation on specific functionality:
//!
//! - **[`insert`]** - Insert operations and bulk operations
//! - **[`update`]** - Update and upsert operations  
//! - **[`select`]** - Query execution and response handling
//! - **[`delete`]** - Delete operations
//! - **[`query_builder`]** - Fluent query building API
//! - **[`storage`]** - File operations (requires `storage` feature)
//! - **[`graphql`]** - GraphQL support (requires `nightly` feature)
//! - **[`errors`]** - Error types and handling utilities
//!
//! ## 🚀 What's Next
//!
//! This SDK is actively maintained and continuously improved. Upcoming features include:
//! - Enhanced Realtime subscriptions
//! - Advanced authentication helpers
//! - Improved type generation utilities
//! - Performance optimizations
//!
//! ## 🤝 Contributing
//!
//! Contributions are welcome! Please check our [GitHub repository](https://github.com/floris-xlx/supabase_rs)
//! for contribution guidelines and open issues.

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

use rand::prelude::ThreadRng;
use rand::RngExt;
use reqwest::Client;

pub mod delete;
pub mod errors;
pub mod insert;
pub mod query;
pub mod query_builder;
pub mod request;
pub mod routing;
pub mod select;
pub mod success;
pub mod tests;
pub mod type_gen;
pub mod update;

// Re-export commonly used types
pub use success::SupabaseErrorResponse;

pub mod graphql;
pub mod nightly;

// This is locked by feature flag `storage` & `realtime`
pub mod realtime;
pub mod storage;

use errors::Result;

/// The main client for interacting with Supabase services.
///
/// `SupabaseClient` provides a unified interface for all Supabase operations including
/// database CRUD operations, file storage, and GraphQL queries. It manages HTTP connections,
/// authentication, and request routing automatically.
///
/// # Architecture
///
/// The client is built around several key components:
/// - **Connection Pool**: Managed by an internal `reqwest::Client` for efficient HTTP reuse
/// - **Authentication**: Automatic header management with API key and bearer token
/// - **Endpoint Routing**: Smart URL construction for different Supabase services
/// - **Error Handling**: Consistent error types across all operations
///
/// # Thread Safety & Performance
///
/// - **Clone-friendly**: Cloning is cheap and shares the underlying connection pool
/// - **Thread-safe**: Can be safely used across async tasks and threads
/// - **Connection pooling**: Automatically reuses HTTP connections for better performance
/// - **Memory efficient**: Minimal overhead per clone
///
/// # TLS Configuration
///
/// - **Default**: Uses the system's native TLS implementation (OpenSSL on most platforms)
/// - **With `rustls` feature**: Uses rustls for TLS (recommended for Alpine Linux/Docker)
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,no_run
/// use supabase_rs::SupabaseClient;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = SupabaseClient::new(
///     "https://your-project.supabase.co",
///     "your-secret-key",
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// ## Multi-threaded Usage
/// ```rust,no_run
/// use supabase_rs::SupabaseClient;
/// use std::sync::Arc;
/// use tokio::task;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Arc::new(SupabaseClient::new(
///     std::env::var("SUPABASE_URL")?,
///     std::env::var("SUPABASE_KEY")?,
/// )?);
///
/// // Clone for use in another task
/// let client_clone = Arc::clone(&client);
/// let handle = task::spawn(async move {
///     client_clone.select("users").execute().await
/// });
///
/// // Original client can still be used
/// let _users = client.select("posts").execute().await?;
/// let _result = handle.await??;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SupabaseClient {
    url: String,
    api_key: String,
    schema: String,
    client: reqwest::Client,
}

impl SupabaseClient {
    /// Creates a new `SupabaseClient` instance with the provided project URL and API key.
    ///
    /// This method initializes the HTTP client with appropriate TLS configuration based on
    /// enabled features and sets up the authentication credentials for all subsequent requests.
    ///
    /// # Arguments
    ///
    /// * `supabase_url` - Your Supabase project URL (e.g., "https://your-project.supabase.co")
    /// * `private_key` - Your Supabase API key (anon key for client-side, service role for server-side)
    ///
    /// # Returns
    ///
    /// Returns `Result<SupabaseClient, ErrorTypes>` where:
    /// - `Ok(SupabaseClient)` - Successfully initialized client ready for use
    /// - `Err(ErrorTypes)` - Initialization failed (typically due to HTTP client setup issues)
    ///
    /// # TLS Configuration
    ///
    /// - **Default**: Uses native TLS (OpenSSL on most platforms)
    /// - **With `rustls` feature**: Uses rustls-tls for cross-platform compatibility
    ///
    /// # Examples
    ///
    /// ## Basic Initialization
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = SupabaseClient::new(
    ///     "https://your-project.supabase.co",
    ///     "your-anon-or-service-key",
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With Environment Variables
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    /// use dotenv::dotenv;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// dotenv().ok();
    ///
    /// let client = SupabaseClient::new(
    ///     std::env::var("SUPABASE_URL")?,
    ///     std::env::var("SUPABASE_KEY")?,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    ///
    /// # fn main() {
    /// match SupabaseClient::new("invalid-url", "key") {
    ///     Ok(client) => println!("Client created successfully"),
    ///     Err(e) => eprintln!("Failed to create client: {:?}", e),
    /// }
    /// # }
    /// ```
    pub fn new(supabase_url: impl Into<String>, private_key: impl Into<String>) -> Result<Self> {
        #[cfg(feature = "rustls")]
        let client = Client::builder().use_rustls_tls().build()?;

        #[cfg(not(feature = "rustls"))]
        let client = Client::new();

        Ok(Self {
            url: supabase_url.into(),
            api_key: private_key.into(),
            schema: "public".to_string(), // default schema
            client,
        })
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = schema.to_string();
        self
    }

    /// Returns the base URL of the Supabase project and table.
    ///
    /// # Arguments
    /// * `table_name` - The name of the table that will be used.
    ///
    /// # Returns
    /// Returns a string containing the endpoint URL.
    ///
    /// The default format is `"{url}/rest/v1/{table}"`. If the environment variable
    /// `SUPABASE_RS_DONT_REST_V1_URL=true` is set, it becomes `"{url}/{table}"`.
    fn endpoint(&self, table_name: &str) -> String {
        let dont_use_rest_v1: bool = std::env::var("SUPABASE_RS_DONT_REST_V1_URL")
            .map(|val| val.to_lowercase() == "true")
            .unwrap_or(false);

        if dont_use_rest_v1 {
            format!("{}/{}", self.url, table_name)
        } else {
            format!("{}/rest/v1/{}", self.url, table_name)
        }
    }
}

/// Generates a random 64-bit signed integer within a larger range.
///
/// This is used by insert helpers that need a default `id` value.
/// The range is `[0, i64::MAX)`, uniform from `rand`.
///
/// # Examples
/// ```
/// let id = supabase_rs::generate_random_id();
/// assert!(id >= 0);
/// ```
pub fn generate_random_id() -> i64 {
    let mut rng: ThreadRng = rand::rng();
    rng.random_range(0..i64::MAX)
}

/// Returns an identifier string `{package-name}/{package-version}` used for a `Client-Info` header.
pub(crate) fn client_info() -> String {
    format!("{}/{PKG_VERSION}", PKG_NAME.replace("_", "-"))
}
