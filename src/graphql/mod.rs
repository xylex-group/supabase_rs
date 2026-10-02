#![cfg(feature = "graphql")]

//! # GraphQL Support for Supabase
//!
//! This module provides experimental GraphQL capabilities for advanced querying and data manipulation.
//! It complements the REST API with more flexible query structures and relational data fetching.
//!
//! > **⚠️ Experimental Feature**: This module requires the `graphql` feature flag and is not recommended for production use.
//!
//! ## 🎯 Core Features
//!
//! - **[`Query`]**: Execute GraphQL queries for complex data retrieval
//! - **[`crate::graphql::request::Request`]**: Build and send GraphQL requests with variables
//! - **[`RootTypes`]**: Type-safe operation classification (Query, Mutation, Subscription)
//! - **Variable Support**: Dynamic query parameters with type safety
//! - **Error Handling**: Structured GraphQL error responses
//!
//! ## 🏗️ GraphQL vs REST
//!
//! | Aspect | REST | GraphQL |
//! |--------|------|---------|
//! | **Data Fetching** | Fixed endpoints | Flexible queries |
//! | **Relations** | Multiple requests | Single request |
//! | **Caching** | HTTP caching | Query-based caching |
//! | **Complexity** | Simple | More complex |
//! | **Performance** | Predictable | Variable |
//! | **Stability** | ✅ Stable | ⚠️ Experimental |
//!
//! ## 📖 Usage Examples
//!
//! ### Basic GraphQL Query
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use supabase_rs::graphql::{request::Request, RootTypes};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = SupabaseClient::new(
//!     std::env::var("SUPABASE_URL")?,
//!     std::env::var("SUPABASE_KEY")?,
//! )?;
//!
//! let request = Request::new(
//!     client,
//!     json!({
//!         "query": r#"
//!             {
//!                 usersCollection(first: 10) {
//!                     edges {
//!                         node {
//!                             id
//!                             email
//!                             created_at
//!                         }
//!                     }
//!                     pageInfo {
//!                         hasNextPage
//!                         endCursor
//!                     }
//!                 }
//!             }
//!         "#
//!     }),
//!     RootTypes::Query
//! );
//!
//! let response = request.send().await?;
//! println!("GraphQL Response: {:#?}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ### GraphQL with Variables
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use supabase_rs::graphql::{request::Request, RootTypes};
//! # use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string())?;
//! let query_with_variables = Request::new(
//!     client,
//!     json!({
//!         "query": r#"
//!             query GetUsersByAge($minAge: Int!, $limit: Int!) {
//!                 usersCollection(
//!                     filter: { age: { gte: $minAge } }
//!                     first: $limit
//!                 ) {
//!                     edges {
//!                         node {
//!                             id
//!                             name
//!                             age
//!                             email
//!                         }
//!                     }
//!                 }
//!             }
//!         "#,
//!         "variables": {
//!             "minAge": 18,
//!             "limit": 50
//!         }
//!     }),
//!     RootTypes::Query
//! );
//!
//! let response = query_with_variables.send().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Complex Relational Queries
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use supabase_rs::graphql::{request::Request, RootTypes};
//! # use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string())?;
//! // Fetch users with their posts and comments in a single query
//! let complex_query = Request::new(
//!     client,
//!     json!({
//!         "query": r#"
//!             {
//!                 usersCollection(first: 5) {
//!                     edges {
//!                         node {
//!                             id
//!                             name
//!                             email
//!                             postsCollection(first: 3) {
//!                                 edges {
//!                                     node {
//!                                         id
//!                                         title
//!                                         content
//!                                         commentsCollection(first: 2) {
//!                                             edges {
//!                                                 node {
//!                                                     id
//!                                                     content
//!                                                     author
//!                                                 }
//!                                             }
//!                                         }
//!                                     }
//!                                 }
//!                             }
//!                         }
//!                     }
//!                 }
//!             }
//!         "#
//!     }),
//!     RootTypes::Query
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ## 🔧 Error Handling
//!
//! GraphQL operations return detailed error information:
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use supabase_rs::graphql::{request::Request, RootTypes};
//! # use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string())?;
//! let request = Request::new(client, json!({"query": "invalid query"}), RootTypes::Query);
//!
//! match request.send().await {
//!     Ok(response) => {
//!         // Check for GraphQL errors in response
//!         if let Some(errors) = response.get("errors") {
//!             eprintln!("GraphQL errors: {:#?}", errors);
//!         } else {
//!             println!("Success: {:#?}", response["data"]);
//!         }
//!     },
//!     Err(err) => {
//!         eprintln!("Request failed: {}", err);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 📚 GraphQL Conventions
//!
//! ### Table Naming
//! All Supabase tables in GraphQL end with `Collection`:
//! - Database table: `users` → GraphQL: `usersCollection`
//! - Database table: `posts` → GraphQL: `postsCollection`
//!
//! ### Query Structure
//! ```graphql
//! {
//!   tableCollection(first: 10, filter: {...}) {
//!     edges {
//!       node {
//!         # Your fields here
//!       }
//!     }
//!     pageInfo {
//!       hasNextPage
//!       endCursor
//!     }
//!   }
//! }
//! ```
//!
//! ## 🚀 Migration from REST
//!
//! ### When to Use GraphQL
//! - **Complex Relations**: Fetching nested data in single request
//! - **Flexible Fields**: Dynamic field selection based on client needs
//! - **Advanced Filtering**: Complex filter combinations
//! - **Real-time Subscriptions**: Live data updates (when available)
//!
//! ### When to Stick with REST
//! - **Simple CRUD**: Basic insert/update/delete operations
//! - **Production Stability**: REST API is stable and well-tested
//! - **Performance Predictability**: Known query performance characteristics
//! - **Caching**: HTTP caching is simpler with REST endpoints
pub mod client;
pub mod error_types;
pub mod parse;
pub mod parsing;
pub mod query;
pub mod request;
pub mod utils;

use serde_json::Value;

/// #### Query
///
/// Represents a GraphQL query.
#[derive(Debug)]
pub struct Query {
    pub query: Value,
}

/// GraphQL operation types for request classification.
///
/// Defines the type of GraphQL operation being performed. This ensures proper
/// routing and validation of requests to the Supabase GraphQL endpoint.
///
/// # Operation Types
///
/// - **`Query`**: Read operations for data retrieval (✅ Supported)
/// - **`Mutation`**: Write operations for data modification (🚧 Planned)
/// - **`Subscription`**: Real-time data subscriptions (🚧 Planned)
/// - **`Fragment`**: Reusable query fragments (🚧 Planned)
///
/// # Current Support
///
/// > **Note**: Currently only `Query` operations are fully supported. Other types are
/// > reserved for future implementation.
///
/// # Examples
///
/// ```text
/// use supabase_rs::graphql::RootTypes;
///
/// // Specify operation type for request
/// let operation_type = RootTypes::Query;
/// println!("Using operation: {:?}", operation_type);
///
/// // Use in request construction
/// // let request = Request::new(client, query_json, RootTypes::Query);
/// ```
///
/// Only `Query` is supported; other operation kinds are reserved and currently experimental.
#[derive(Debug)]
pub enum RootTypes {
    /// Data retrieval operations (fully supported)
    Query,
    /// Data modification operations (planned)
    Mutation,
    /// Real-time data subscriptions (planned)
    Subscription,
    /// Reusable query fragments (planned)
    Fragment,
}

#[derive(Debug)]
pub enum GraphQLOperators {
    First,
    Last,
    Before,
    After,
    Filter,
    OrderBy,
}
