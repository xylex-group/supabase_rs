//! Async Rust client for Supabase PostgREST.
//!
//! CRUD operations and fluent PostgREST queries, including embedded joins, are available by default.
//! RPC, Storage, GraphQL, and type generation are opt-in Cargo features. GraphQL remains experimental.
//!
//! ```toml
//! [dependencies]
//! supabase_rs = "0.8"
//! ```
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = SupabaseClient::new(
//!     std::env::var("SUPABASE_URL")?,
//!     std::env::var("SUPABASE_KEY")?,
//! )?;
//! let rows = client.select("users").eq("active", "true").execute().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Select counts are response metadata, available through `execute_with_count`; they are not rows.
//! Errors use [`Error`] and preserve HTTP status and structured provider details where available.
//!
//! Feature flags: `rpc`, `storage`, `graphql` (experimental), `typegen`, `rustls`, and `native_tls`.
//! The `nightly` flag remains as an alias for `graphql`.
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

use rand::prelude::ThreadRng;
use rand::RngExt;
use reqwest::{Client, Url};

pub mod delete;
pub mod errors;
pub mod insert;
pub mod query;
pub mod query_builder;
pub mod request;
pub mod routing;
pub mod select;
pub mod success;
#[cfg(test)]
pub mod tests;
#[cfg(feature = "typegen")]
pub mod type_gen;
pub mod update;

// Re-export commonly used types
pub use success::{ResponseData, SupabaseErrorResponse};

#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "storage")]
pub mod storage;

pub use errors::{ApiError, Error, Result};

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
///     "<https://your-project.supabase.co>",
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
#[derive(Clone)]
pub struct SupabaseClient {
    url: Url,
    api_key: String,
    schema: String,
    rest_prefix: Option<String>,
    client: reqwest::Client,
}

impl std::fmt::Debug for SupabaseClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SupabaseClient")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .field("schema", &self.schema)
            .field("rest_prefix", &self.rest_prefix)
            .finish_non_exhaustive()
    }
}

/// Builder for configuring a Supabase client.
pub struct SupabaseClientBuilder {
    supabase_url: String,
    api_key: String,
    schema: String,
    rest_prefix: Option<String>,
    client: Option<Client>,
}

impl SupabaseClient {
    /// Creates a new `SupabaseClient` instance with the provided project URL and API key.
    ///
    /// This method initializes the HTTP client with appropriate TLS configuration based on
    /// enabled features and sets up the authentication credentials for all subsequent requests.
    ///
    /// # Arguments
    ///
    /// * `supabase_url` - Your Supabase project URL (e.g., "<https://your-project.supabase.co>")
    /// * `private_key` - Your Supabase API key (anon key for client-side, service role for server-side)
    ///
    /// # Returns
    ///
    /// Returns `Result<SupabaseClient, Error>` where:
    /// - `Ok(SupabaseClient)` - Successfully initialized client ready for use
    /// - `Err(Error)` - Initialization failed (typically due to HTTP client setup issues)
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
    ///     "<https://your-project.supabase.co>",
    ///     "your-anon-or-service-key",
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With Environment Variables
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        SupabaseClientBuilder::new(supabase_url, private_key).build()
    }

    /// Starts building a client with explicit endpoint and HTTP settings.
    pub fn builder(
        supabase_url: impl Into<String>,
        private_key: impl Into<String>,
    ) -> SupabaseClientBuilder {
        SupabaseClientBuilder::new(supabase_url, private_key)
    }

    pub(crate) fn endpoint_with_segments(&self, suffix: &[&str]) -> String {
        self.build_endpoint(self.rest_prefix.as_deref(), suffix)
    }

    #[cfg(any(feature = "graphql", feature = "storage"))]
    pub(crate) fn service_endpoint_with_segments(&self, suffix: &[&str]) -> String {
        self.build_endpoint(None, suffix)
    }

    fn build_endpoint(&self, prefix: Option<&str>, suffix: &[&str]) -> String {
        let mut endpoint = self.url.clone();
        let mut segments = endpoint
            .path_segments_mut()
            .expect("client URL is validated as a hierarchical URL");
        segments.pop_if_empty();
        let prefix_segments = prefix.into_iter().flat_map(|value| value.split('/'));
        for segment in prefix_segments.chain(suffix.iter().copied()) {
            segments.push(segment);
        }
        drop(segments);
        endpoint.to_string()
    }
}

impl SupabaseClientBuilder {
    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = schema.to_owned();
        self
    }

    /// Sets the REST path prefix. Pass an empty string to use the project root.
    pub fn rest_prefix(mut self, prefix: &str) -> Self {
        self.rest_prefix = (!prefix.is_empty()).then(|| prefix.trim_matches('/').to_owned());
        self
    }

    /// Uses a caller-provided reusable HTTP client.
    pub fn http_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Validates configuration and creates the client.
    pub fn build(self) -> Result<SupabaseClient> {
        let mut url = Url::parse(&self.supabase_url)
            .map_err(|error| Error::Configuration(format!("invalid project URL: {error}")))?;
        if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
            return Err(Error::Configuration(
                "project URL must use http or https and include a host".to_owned(),
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(Error::Configuration(
                "project URL must not contain embedded credentials".to_owned(),
            ));
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(Error::Configuration(
                "project URL must not include a query or fragment".to_owned(),
            ));
        }
        if self.api_key.is_empty() {
            return Err(Error::Configuration("API key must not be empty".to_owned()));
        }
        let prefix = self.rest_prefix.map(|prefix| {
            prefix
                .trim_matches('/')
                .split('/')
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });
        if prefix.as_ref().is_some_and(|segments| {
            segments.is_empty() || segments.iter().any(|segment| segment.is_empty())
        }) {
            return Err(Error::Configuration(
                "REST prefix must contain non-empty path segments".to_owned(),
            ));
        }
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        let rest_prefix = prefix.map(|segments| segments.join("/"));

        #[cfg(feature = "rustls")]
        let client = match self.client {
            Some(client) => client,
            None => Client::builder().use_rustls_tls().build()?,
        };

        #[cfg(not(feature = "rustls"))]
        let client = self.client.unwrap_or_else(Client::new);

        Ok(SupabaseClient {
            url,
            api_key: self.api_key,
            schema: self.schema,
            rest_prefix,
            client,
        })
    }
}

impl SupabaseClientBuilder {
    /// Creates a builder using the standard `rest/v1` endpoint prefix.
    pub fn new(supabase_url: impl Into<String>, private_key: impl Into<String>) -> Self {
        Self {
            supabase_url: supabase_url.into(),
            api_key: private_key.into(),
            schema: "public".to_owned(),
            rest_prefix: Some("rest/v1".to_owned()),
            client: None,
        }
    }
}

impl SupabaseClient {
    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = schema.to_owned();
        self
    }

    /// Calls a Postgres RPC function.
    ///
    /// # Arguments
    /// * `function_name` - The name of the RPC function to call.
    /// * `params` - The arguments to pass to the function. Can be a struct, map, or `json!({})`.
    ///
    /// # Returns
    /// Returns a `RpcBuilder` for further chaining (filtering) or execution.
    #[cfg(feature = "rpc")]
    pub fn rpc<T>(&self, function_name: &str, params: T) -> crate::rpc::RpcBuilder
    where
        T: serde::Serialize,
    {
        crate::rpc::RpcBuilder::new(self.clone(), function_name, params)
    }

    /// Returns the base URL of the Supabase project and table.
    ///
    /// # Arguments
    /// * `table_name` - The name of the table that will be used.
    ///
    /// # Returns
    /// Returns a string containing the endpoint URL.
    ///
    fn endpoint(&self, table_name: &str) -> String {
        self.endpoint_with_segments(&[table_name])
    }

    /// Returns the RPC endpoint URL for a given function name.
    ///
    /// # Arguments
    /// * `function_name` - The name of the RPC function to call.
    ///
    /// # Returns
    /// Returns a string containing the RPC endpoint URL.
    ///
    #[cfg(feature = "rpc")]
    pub(crate) fn rpc_endpoint(&self, function_name: &str) -> String {
        self.endpoint_with_segments(&["rpc", function_name])
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
