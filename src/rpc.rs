#![cfg(feature = "rpc")]

//! # Remote Procedure Call (RPC) Support
//!
//! This module provides support for calling PostgreSQL functions via PostgREST's RPC endpoints.
//! It enables executing stored procedures, functions, and custom SQL operations with full
//! parameter support and result filtering capabilities.
//!
//! ## 🏗️ Architecture
//!
//! The RPC implementation is built around several key components:
//! - **[`RpcBuilder`]**: Fluent API for constructing and executing RPC calls
//! - **Parameter Handling**: Type-safe serialization of function arguments
//! - **Header Management**: Schema-aware headers for multi-schema support
//! - **Result Filtering**: Post-execution filtering of returned data sets
//!
//! ## 🎯 Design Philosophy
//!
//! - **Consistent API**: Follows the same fluent pattern as other SDK operations
//! - **Type Safety**: Leverages Rust's type system for compile-time validation
//! - **Performance**: Efficient serialization and HTTP request handling
//! - **Flexibility**: Supports all PostgREST RPC features including filtering
//!
//! ## 📖 Usage Examples
//!
//! ### Basic RPC Call
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), String> {
//! let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//!
//! // Call a function that returns a set of records
//! let results = client.rpc("get_active_users", json!({ "active": true }))
//!     .execute()
//!     .await?;
//!
//! // Call a function that returns a single value
//! let count = client.rpc("count_users", json!({}))
//!     .execute_single()
//!     .await?;
//!
//! // Call a void function
//! client.rpc("cleanup_old_sessions", json!({ "days": 30 }))
//!     .execute_void()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### RPC with Filtering
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Filter results after function execution
//! let filtered = client.rpc("get_users", json!({}))
//!     .eq("status", "active")
//!     .gte("age", "18")
//!     .order("name", true)
//!     .limit(10)
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use serde::Serialize;
use serde_json::{json, Value};

use crate::errors::Result;
use crate::query::Query;
use crate::request::headers::HeadersTypes;
use crate::request::Headers;
use crate::success::handle_response;
use crate::SupabaseClient;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Response;

/// Builder for constructing and executing RPC calls.
///
/// `RpcBuilder` provides a fluent interface for calling PostgreSQL functions via PostgREST's
/// RPC endpoints. It supports parameter passing, result filtering, and multiple execution modes.
///
/// # Execution Modes
///
/// RPC functions can return different types of results:
/// - **Set of records**: Use `.execute()` for array results
/// - **Single value/object**: Use `.execute_single()` for scalar/row results
/// - **Void**: Use `.execute_void()` for functions with no return value
///
/// # Filtering Support
///
/// PostgREST allows filtering the results of set-returning functions using standard query
/// parameters. The `RpcBuilder` reuses the existing `Query` infrastructure to provide
/// the same filtering capabilities as regular table queries.
///
/// # Examples
///
/// ## Basic Usage
/// ```rust,no_run
/// use supabase_rs::SupabaseClient;
/// use serde_json::json;
///
/// # async fn example() -> Result<(), String> {
/// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
/// let results = client.rpc("my_function", json!({ "param": "value" }))
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ## With Filtering
/// ```rust,no_run
/// # use supabase_rs::SupabaseClient;
/// # use serde_json::json;
/// # async fn example() -> Result<(), String> {
/// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
/// let active_users = client.rpc("get_users", json!({}))
///     .eq("status", "active")
///     .limit(10)
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct RpcBuilder {
    /// The Supabase client instance for executing requests
    client: SupabaseClient,
    /// Name of the RPC function to call
    function_name: String,
    /// Serialized function parameters
    params: Value,
    /// Query object for filtering results (for set-returning functions)
    query: Query,
}

impl RpcBuilder {
    /// Creates a new `RpcBuilder` for calling a PostgreSQL function.
    ///
    /// This constructor is typically called by [`SupabaseClient::rpc`] and not used directly.
    /// It immediately serializes the parameters to `Value` to simplify the builder's type signature.
    ///
    /// # Arguments
    ///
    /// * `client` - The Supabase client instance
    /// * `function_name` - Name of the PostgreSQL function to call
    /// * `params` - Function arguments (must implement `Serialize`)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase_rs::{SupabaseClient, rpc::RpcBuilder};
    /// use serde_json::json;
    ///
    /// # fn example() -> Result<(), String> {
    /// let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
    /// let builder = RpcBuilder::new(client, "my_function", json!({ "param": "value" }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<T: Serialize>(client: SupabaseClient, function_name: &str, params: T) -> Self {
        Self {
            client,
            function_name: function_name.to_string(),
            params: serde_json::to_value(params).unwrap_or(json!({})),
            query: Query::new(),
        }
    }

    /// Executes the RPC call expecting an array of results (SETOF records).
    ///
    /// This method is for functions that return multiple rows, such as:
    /// - `RETURNS SETOF table_name`
    /// - `RETURNS TABLE(...)`
    /// - `RETURNS SETOF record`
    ///
    /// The results can be filtered using the query builder methods before execution.
    ///
    /// # Returns
    ///
    /// Returns `Result<Vec<Value>>` where:
    /// - `Ok(Vec<Value>)` - Array of JSON objects representing the returned rows
    /// - `Err(ErrorTypes)` - Request failed (network, authentication, function not found, etc.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), String> {
    /// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
    /// let users = client.rpc("get_active_users", json!({ "active": true }))
    ///     .execute()
    ///     .await?;
    ///
    /// for user in users {
    ///     println!("User: {}", user["name"]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(self) -> Result<Vec<Value>> {
        self.execute_internal(false).await
    }

    /// Executes the RPC call expecting a single result (scalar or single row).
    ///
    /// This method is for functions that return a single value, such as:
    /// - `RETURNS integer`, `RETURNS text`, etc. (scalar)
    /// - `RETURNS table_name` (single row)
    /// - `RETURNS record` (single composite)
    ///
    /// The method sets appropriate headers to request single-object response format
    /// from PostgREST when applicable.
    ///
    /// # Returns
    ///
    /// Returns `Result<Value>` where:
    /// - `Ok(Value)` - The single returned value (could be scalar, object, or null)
    /// - `Err(ErrorTypes)` - Request failed or returned multiple rows
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), String> {
    /// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
    /// // Scalar return
    /// let count = client.rpc("count_users", json!({}))
    ///     .execute_single()
    ///     .await?;
    /// let count_num = count.as_i64().unwrap();
    ///
    /// // Single row return
    /// let user = client.rpc("get_user_by_id", json!({ "id": 123 }))
    ///     .execute_single()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_single(self) -> Result<Value> {
        let results = self.execute_internal(true).await?;
        if results.len() == 1 {
            Ok(results.into_iter().next().unwrap())
        } else {
            Err(crate::errors::ErrorTypes::UnknownError)
        }
    }

    /// Executes the RPC call expecting no return value (void function).
    ///
    /// This method is for functions that return `void` or have no return value.
    /// It expects a 204 No Content response from PostgREST.
    ///
    /// # Returns
    ///
    /// Returns `Result<()>` where:
    /// - `Ok(())` - Function executed successfully
    /// - `Err(ErrorTypes)` - Request failed or returned unexpected content
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use supabase_rs::SupabaseClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> Result<(), String> {
    /// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
    /// client.rpc("cleanup_old_records", json!({ "days": 30 }))
    ///     .execute_void()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_void(self) -> Result<()> {
        let response = self.execute_request(true).await?;
        let status = response.status();

        if status == 204 {
            Ok(())
        } else {
            // For non-204 responses, treat as error
            let _error_body = response.text().await.unwrap_or_default();
            Err(crate::errors::ErrorTypes::UnknownError)
        }
    }

    /// Internal implementation shared by execute and execute_single.
    async fn execute_internal(self, single: bool) -> Result<Vec<Value>> {
        let response = self.execute_request(single).await?;
        let status = response.status();

        if !status.is_success() {
            // For non-success responses, treat as error
            let _error_body = response.text().await.unwrap_or_default();
            return Err(crate::errors::ErrorTypes::UnknownError);
        }

        // Parse response
        handle_response(response)
            .await
            .map_err(|_e| crate::errors::ErrorTypes::UnknownError)
    }

    /// Internal method to execute the HTTP request.
    async fn execute_request(self, single: bool) -> Result<Response> {
        // Build endpoint URL
        let url = self.client.rpc_endpoint(&self.function_name);

        // Build query string from filters
        let query_string = self.query.build();
        let endpoint = if query_string.is_empty() {
            url
        } else {
            format!("{}?{}", url, query_string)
        };

        // create headers with default values
        let mut headers = Headers::with_defaults(&self.client.api_key, &self.client.api_key);

        // Content-Type is always application/json for RPC
        headers.insert(HeadersTypes::ContentType.as_str(), "application/json");

        // Handle schema headers
        if self.client.schema != "public" {
            headers.insert(HeadersTypes::ContentProfile.as_str(), &self.client.schema);
            headers.insert(HeadersTypes::AcceptProfile.as_str(), &self.client.schema);
        }

        // Handle single object response if requested
        if single {
            headers.insert("Accept", "application/vnd.pgrst.object+json");
        }

        // convert headers to HeaderMap
        let mut header_map = HeaderMap::new();
        for (key, value) in headers.get_headers() {
            header_map.insert(
                HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| crate::errors::ErrorTypes::UnknownError)?,
                HeaderValue::from_str(&value)
                    .map_err(|_| crate::errors::ErrorTypes::UnknownError)?,
            );
        }

        // send the request
        let response = self
            .client
            .client
            .post(&endpoint)
            .headers(header_map)
            .json(&self.params)
            .send()
            .await
            .map_err(crate::errors::ErrorTypes::ReqwestError)?;

        Ok(response)
    }
}

// Filter methods - reuse Query functionality
impl RpcBuilder {
    /// Adds an equality filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column equals the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The value to match (will be stringified)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_rs::SupabaseClient;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), String> {
    /// # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
    /// let results = client.rpc("get_users", json!({}))
    ///     .eq("status", "active")
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("eq.{value}"));
        self
    }

    /// Adds a not-equals filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to exclude rows where
    /// the specified column equals the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The value to exclude (will be stringified)
    pub fn neq(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("neq.{value}"));
        self
    }

    /// Adds a greater-than filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column is greater than the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The minimum value (exclusive)
    pub fn gt(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("gt.{value}"));
        self
    }

    /// Adds a less-than filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column is less than the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The maximum value (exclusive)
    pub fn lt(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("lt.{value}"));
        self
    }

    /// Adds a greater-than-or-equals filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column is greater than or equal to the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The minimum value (inclusive)
    pub fn gte(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("gte.{value}"));
        self
    }

    /// Adds a less-than-or-equals filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column is less than or equal to the given value.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `value` - The maximum value (inclusive)
    pub fn lte(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("lte.{value}"));
        self
    }

    /// Adds an "in" filter to the RPC results.
    ///
    /// Filters the results of a set-returning function to only include rows where
    /// the specified column matches any of the given values.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to filter on
    /// * `values` - Array of values to match
    pub fn in_<T>(mut self, column: &str, values: &[T]) -> Self
    where
        T: ToString,
    {
        let list = values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        self.query.add_param(column, &format!("in.({})", list));
        self
    }

    /// Adds a full-text search filter to the RPC results.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to perform the text search on.
    /// * `value` - The value to search for within the column.
    pub fn text_search(mut self, column: &str, value: &str) -> Self {
        self.query.add_param(column, &format!("fts.{value}"));
        self
    }

    /// Adds a limit to the number of rows returned by the RPC results.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of rows to return.
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.add_param("limit", &limit.to_string());
        self
    }

    /// Adds an offset to the RPC results to skip a specified number of rows.
    ///
    /// # Arguments
    ///
    /// * `offset` - The number of rows to skip from the beginning of the result set.
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.add_param("offset", &offset.to_string());
        self
    }

    /// Adds a range to the RPC results for pagination using PostgREST range syntax.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting index (0-based) of the range.
    /// * `to` - The ending index (inclusive) of the range.
    pub fn range(mut self, from: usize, to: usize) -> Self {
        self.query.set_range(from, to);
        self
    }

    /// Adds sorting to the RPC results.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to sort by.
    /// * `ascending` - `true` for ascending order, `false` for descending.
    pub fn order(mut self, column: &str, ascending: bool) -> Self {
        let order_value: &str = if ascending { "asc" } else { "desc" };
        self.query
            .add_param("order", &format!("{column}.{order_value}"));
        self
    }

    /// Adds a parameter to count the exact number of rows that match the RPC results.
    pub fn count(mut self) -> Self {
        self.query.add_param("count", "exact");
        self
    }

    /// Selects specific columns from the RPC results.
    ///
    /// # Arguments
    ///
    /// * `columns` - Vector of column names to select.
    pub fn columns(mut self, columns: Vec<&str>) -> Self {
        let columns_str: String = columns.join(",");
        self.query.add_param("select", &columns_str);
        self
    }
}
