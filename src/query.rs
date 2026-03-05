//! # Query Building and Execution
//!
//! This module provides the core query building infrastructure for constructing complex
//! database queries using a fluent, chainable API. It implements the PostgREST query
//! specification for seamless integration with Supabase.
//!
//! ## 🏗️ Architecture
//!
//! The query system is built around several key components:
//! - **[`QueryBuilder`]**: Fluent API for chaining operations
//! - **[`Query`]**: Internal query state and parameter management
//! - **[`Filter`]**: Individual filter conditions with type-safe operators
//! - **[`Sort`]**: Sorting specifications with direction control
//!
//! ## 🎯 Design Philosophy
//!
//! - **Fluent Interface**: Method chaining for intuitive query construction
//! - **Type Safety**: Compile-time guarantees for query validity
//! - **Performance**: Efficient query string generation and HTTP request batching
//! - **Flexibility**: Support for complex filtering and sorting combinations
//!
//! ## 🔍 Query Components
//!
//! ### Filter Operations
//! | Operator | PostgREST | Description | Example |
//! |----------|-----------|-------------|---------|
//! | `Equals` | `eq` | Exact match | `age.eq=25` |
//! | `NotEquals` | `neq` | Not equal | `status.neq=inactive` |
//! | `GreaterThan` | `gt` | Greater than | `score.gt=100` |
//! | `LessThan` | `lt` | Less than | `price.lt=50` |
//! | `GreaterThanOrEquals` | `gte` | Greater or equal | `created_at.gte=2024-01-01` |
//! | `LessThanOrEquals` | `lte` | Less or equal | `updated_at.lte=2024-12-31` |
//!
//! ### Sort Operations
//! | Order | PostgREST | Description |
//! |-------|-----------|-------------|
//! | `Ascending` | `asc` | Smallest to largest |
//! | `Descending` | `desc` | Largest to smallest |
//!
//! ## 📖 Usage Examples
//!
//! ### Basic Query Construction
//!
//! ```rust,no_run
//! use supabase_rs::query::{Filter, Operator, Query, Sort, SortOrder};
//!
//! # fn example() -> Result<(), String> {
//! // Build a query manually (low-level API)
//! let mut query = Query::new();
//!
//! // Add a filter: age > 18
//! let age_filter = Filter {
//!     column: "age".to_string(),
//!     operator: Operator::GreaterThan,
//!     value: "18".to_string(),
//! };
//! query.add_filter(age_filter);
//!
//! // Add sorting: name ascending
//! let name_sort = Sort {
//!     column: "name".to_string(),
//!     order: SortOrder::Ascending,
//! };
//! query.add_sort(name_sort);
//!
//! // Build the query string
//! let query_string = query.build();
//! println!("Generated query: {}", query_string);
//! # Ok(())
//! # }
//! ```
//!
//! ### Fluent API Usage (Recommended)
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::Value;
//!
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Use the fluent QueryBuilder API (high-level, recommended)
//! let results: Vec<Value> = client
//!     .select("users")                    // Start query
//!     .gte("age", "18")                   // Add filter
//!     .eq("verified", "true")             // Add another filter
//!     .order("created_at", false)         // Sort by created_at desc
//!     .limit(50)                          // Limit results
//!     .execute()                          // Execute query
//!     .await?;
//! # Ok(())
//! # }
//! ```

// local imports
use crate::SupabaseClient;

// Re-export join types for structured nested selects
pub use crate::query_builder::join::{JoinModifier, JoinSpec};

/// Comparison operators for query filters.
///
/// These operators map directly to PostgREST's filter operators and provide
/// type-safe query construction with compile-time validation.
///
/// # PostgREST Mapping
/// Each variant corresponds to a specific PostgREST operator:
/// - `Equals` → `eq` - Exact equality match
/// - `NotEquals` → `neq` - Inequality match  
/// - `GreaterThan` → `gt` - Numeric/date comparison
/// - `LessThan` → `lt` - Numeric/date comparison
/// - `GreaterThanOrEquals` → `gte` - Inclusive range start
/// - `LessThanOrEquals` → `lte` - Inclusive range end
///
/// # Examples
///
/// ```rust
/// use supabase_rs::query::{Filter, Operator};
///
/// // Age exactly 25
/// let exact_age = Filter {
///     column: "age".to_string(),
///     operator: Operator::Equals,
///     value: "25".to_string(),
/// };
///
/// // Price range: $10 to $100
/// let min_price = Filter {
///     column: "price".to_string(),
///     operator: Operator::GreaterThanOrEquals,
///     value: "10.00".to_string(),
/// };
/// ```
#[derive(Debug)]
pub enum Operator {
    /// Exact equality comparison (`eq` in PostgREST)
    Equals,
    /// Inequality comparison (`neq` in PostgREST)
    NotEquals,
    /// Greater than comparison (`gt` in PostgREST)
    GreaterThan,
    /// Less than comparison (`lt` in PostgREST)
    LessThan,
    /// Greater than or equal comparison (`gte` in PostgREST)
    GreaterThanOrEquals,
    /// Less than or equal comparison (`lte` in PostgREST)
    LessThanOrEquals,
}

/// Sort order specification for query results.
///
/// Determines the direction of sorting for query results. Maps directly to
/// PostgREST's ordering syntax for consistent behavior.
///
/// # Examples
///
/// ```rust
/// use supabase_rs::query::{Sort, SortOrder};
///
/// // Sort by creation date, newest first
/// let recent_first = Sort {
///     column: "created_at".to_string(),
///     order: SortOrder::Descending,
/// };
///
/// // Sort by name alphabetically
/// let alphabetical = Sort {
///     column: "name".to_string(),
///     order: SortOrder::Ascending,
/// };
/// ```
#[derive(Debug)]
pub enum SortOrder {
    /// Sort in ascending order (A-Z, 0-9, oldest-newest)
    Ascending,
    /// Sort in descending order (Z-A, 9-0, newest-oldest)
    Descending,
}

/// A filter condition for database queries.
///
/// Represents a single filter criterion that can be applied to a query.
/// Combines a column name, comparison operator, and target value into a
/// PostgREST-compatible filter expression.
///
/// # Examples
///
/// ```rust
/// use supabase_rs::query::{Filter, Operator};
///
/// // Filter for active users
/// let active_filter = Filter {
///     column: "status".to_string(),
///     operator: Operator::Equals,
///     value: "active".to_string(),
/// };
///
/// // Filter for adults (age >= 18)
/// let adult_filter = Filter {
///     column: "age".to_string(),
///     operator: Operator::GreaterThanOrEquals,
///     value: "18".to_string(),
/// };
/// ```
#[derive(Debug)]
pub struct Filter {
    /// The database column name to filter on
    pub column: String,
    /// The comparison operator to use
    pub operator: Operator,
    /// The value to compare against (will be URL-encoded)
    pub value: String,
}

/// Sorting specification for query results.
///
/// Defines how query results should be ordered by specifying a column name
/// and sort direction. Multiple sorts can be applied to create complex ordering.
///
/// # Examples
///
/// ```rust
/// use supabase_rs::query::{Sort, SortOrder};
///
/// // Sort by creation date, newest first
/// let by_date = Sort {
///     column: "created_at".to_string(),
///     order: SortOrder::Descending,
/// };
///
/// // Sort by name alphabetically
/// let by_name = Sort {
///     column: "name".to_string(),
///     order: SortOrder::Ascending,
/// };
/// ```
#[derive(Debug)]
pub struct Sort {
    /// The database column name to sort by
    pub column: String,
    /// The sort direction (ascending or descending)
    pub order: SortOrder,
}

/// Internal query state container.
///
/// Manages the complete state of a database query including parameters, filters,
/// sorting criteria, and pagination settings. This struct is typically managed
/// internally by [`QueryBuilder`] and not used directly by end users.
///
/// # Query Components
///
/// - **Parameters**: Key-value pairs for basic query parameters (select, limit, etc.)
/// - **Filters**: Structured filter conditions with type-safe operators
/// - **Sorts**: Sorting specifications with column and direction
/// - **Range**: Optional pagination range for efficient data retrieval
///
/// # Examples
///
/// ```rust
/// use supabase_rs::query::{Query, Filter, Operator, Sort, SortOrder};
///
/// let mut query = Query::new();
///
/// // Add parameters
/// query.add_param("select", "id,name,email");
/// query.add_param("limit", "50");
///
/// // Add filters
/// query.add_filter(Filter {
///     column: "age".to_string(),
///     operator: Operator::GreaterThanOrEquals,
///     value: "18".to_string(),
/// });
///
/// // Add sorting
/// query.add_sort(Sort {
///     column: "name".to_string(),
///     order: SortOrder::Ascending,
/// });
///
/// // Set pagination range
/// query.set_range(0, 49); // First 50 records
///
/// let query_string = query.build();
/// ```
#[derive(Debug, Default)]
pub struct Query {
    /// Query parameters as key-value pairs (select, limit, offset, etc.)
    pub params: Vec<(String, String)>,
    /// Filter conditions to apply to the query
    pub filters: Vec<Filter>,
    /// Sorting criteria for result ordering
    pub sorts: Vec<Sort>,
    /// Optional pagination range (from_index, to_index)
    pub range: Option<(usize, usize)>,
}

/// Fluent query builder for constructing and executing database queries.
///
/// `QueryBuilder` provides a chainable API for building complex queries against Supabase tables.
/// It encapsulates query state and provides type-safe methods for filtering, sorting, and pagination.
///
/// # Design Pattern
///
/// The builder uses the fluent interface pattern, allowing operations to be chained together:
/// ```text
/// client.select("table").eq("column", "value").limit(10).execute()
/// ```
///
/// # Performance Characteristics
///
/// - **Lazy Evaluation**: Query is built but not executed until `.execute()` is called
/// - **Efficient Serialization**: Parameters are collected and serialized once
/// - **Connection Reuse**: Leverages the underlying client's connection pool
/// - **Memory Efficient**: Minimal allocations during query construction
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,no_run
/// use supabase_rs::SupabaseClient;
/// use serde_json::Value;
///
/// # async fn example() -> Result<(), String> {
/// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
/// let users: Vec<Value> = client
///     .select("users")
///     .eq("active", "true")
///     .order("created_at", false)
///     .limit(25)
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Complex Queries
/// ```rust,no_run
/// # use supabase_rs::SupabaseClient;
/// # use serde_json::Value;
/// # async fn example() -> Result<(), String> {
/// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
/// let products: Vec<Value> = client
///     .from("products")
///     .columns(vec!["id", "name", "price"])
///     .gte("price", "10.00")
///     .lte("price", "100.00")
///     .in_("category", &["electronics", "books"])
///     .text_search("description", "wireless")
///     .order("price", true)
///     .range(0, 19)
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct QueryBuilder {
    /// The Supabase client instance for executing queries
    pub client: SupabaseClient,
    /// Internal query state and parameters
    pub query: Query,
    /// Target table name for the query
    pub table_name: String,
}
