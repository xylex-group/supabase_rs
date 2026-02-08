//! # Insert Operations
//!
//! This module provides comprehensive functionality for inserting new records into Supabase tables.
//! It supports single inserts, bulk operations, and conditional inserts with automatic conflict detection.
//!
//! ## 🎯 Core Features
//!
//! - **[`insert`]**: Standard insert with automatic ID generation
//! - **[`insert_if_unique`]**: Conditional insert that prevents duplicates
//! - **[`bulk_insert`]**: Efficient bulk operations for multiple records
//! - **[`insert_with_generated_id`]**: Insert with client-side ID generation
//!
//! ## 🏗️ Operation Types
//!
//! | Method | ID Handling | Conflict Behavior | Performance | Use Case |
//! |--------|-------------|-------------------|-------------|----------|
//! | `insert` | Auto-generated or provided | Fails on conflict | ✅ Fast | Standard inserts |
//! | `insert_if_unique` | Auto-generated | Checks uniqueness first | ⚠️ Slower | Prevent duplicates |
//! | `bulk_insert` | Auto-generated or provided | Fails on any conflict | ✅ Fast | Multiple records |
//! | `insert_with_generated_id` | Client-side random | Fails on conflict | ✅ Fast | Custom ID control |
//!
//! ## 📊 Data Serialization
//!
//! All insert methods accept any type that implements `serde::Serialize`:
//! - Raw JSON values (`serde_json::Value`)
//! - Structs with `#[derive(Serialize)]`
//! - Maps, vectors, and primitive types
//! - Custom serializable types
//!
//! ## 🔧 Error Handling
//!
//! Insert operations return `Result<String, String>` where:
//! - **Success**: `Ok(String)` contains the ID of the inserted record
//! - **Failure**: `Err(String)` contains a descriptive error message
//!
//! ### Common Error Scenarios
//! - **409 Conflict**: Duplicate entry violates unique constraint
//! - **401 Unauthorized**: Invalid or missing API key
//! - **403 Forbidden**: Insufficient permissions (check RLS policies)
//! - **422 Unprocessable**: Invalid data format or missing required fields
//!
//! ## 📖 Usage Examples
//!
//! ### Basic Insert Operations
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Simple insert with JSON
//! let user_id = client.insert("users", json!({
//!     "name": "Alice Johnson",
//!     "email": "alice@example.com",
//!     "age": 28,
//!     "verified": false
//! })).await?;
//!
//! println!("Created user with ID: {}", user_id);
//! # Ok(())
//! # }
//! ```
//!
//! ### Structured Data Insert
//!
//! ```rust,no_run
//! use serde::Serialize;
//! use supabase_rs::SupabaseClient;
//!
//! #[derive(Serialize)]
//! struct User {
//!     name: String,
//!     email: String,
//!     age: u32,
//!     verified: bool,
//! }
//!
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! let new_user = User {
//!     name: "Bob Smith".to_string(),
//!     email: "bob@example.com".to_string(),
//!     age: 35,
//!     verified: true,
//! };
//!
//! let user_id = client.insert("users", new_user).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Conditional Insert (Prevent Duplicates)
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Insert only if no existing record matches ALL provided fields
//! match client.insert_if_unique("users", json!({
//!     "email": "unique@example.com",
//!     "username": "unique_user"
//! })).await {
//!     Ok(id) => println!("Created unique user with ID: {}", id),
//!     Err(err) if err.contains("409") => {
//!         println!("User already exists with this email or username");
//!     },
//!     Err(err) => println!("Insert failed: {}", err),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Bulk Insert Operations
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Insert multiple records efficiently
//! let users = vec![
//!     json!({"name": "User 1", "email": "user1@example.com"}),
//!     json!({"name": "User 2", "email": "user2@example.com"}),
//!     json!({"name": "User 3", "email": "user3@example.com"}),
//! ];
//!
//! client.bulk_insert("users", users).await?;
//! println!("Successfully inserted multiple users");
//! # Ok(())
//! # }
//! ```
//!
//! ## ⚡ Performance Considerations
//!
//! ### Choosing the Right Insert Method
//!
//! 1. **`insert`**: Fastest option, use when you're confident about data uniqueness
//! 2. **`insert_if_unique`**: Slower due to pre-check query, use when duplicates are likely
//! 3. **`bulk_insert`**: Most efficient for multiple records, single HTTP request
//!
//! ### Best Practices
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // ✅ Good: Batch multiple inserts
//! let records = vec![json!({"name":"tom"})/* ... multiple records ... */];
//! client.bulk_insert("logs", records).await?;
//!
//! // ❌ Avoid: Individual inserts in loops
//! // for record in records {
//! //     client.insert("logs", record).await?; // Inefficient!
//! // }
//! # Ok(())
//! # }
//! ```

use crate::request::headers::HeadersTypes;
use crate::{generate_random_id, SupabaseClient};
use reqwest::Response;
use serde_json::{json, Value};

impl SupabaseClient {
    /// Inserts a new row into the specified table with a randomly generated ID for column `id`.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table.
    /// * `body` - A JSON value containing the data to be inserted.
    ///
    /// # Example
    /// ```ignore
    /// // Initialize the Supabase client
    /// use supabase_rs::SupabaseClient;
    /// let client = SupabaseClient::new("your_supabase_url", "your_supabase_key");
    ///
    /// // This will insert a new row into the table
    /// let insert_result = client.insert(
    ///   "your_table_name",
    ///   json!(
    ///     {"column_name": "value"}
    ///   )
    /// ).await;
    /// ```
    ///
    ///
    /// # Returns
    /// This method returns a `Result<String, String>`. On success, it returns `Ok(String)` with the new row's ID,
    /// and on failure, it returns `Err(String)` with an error message.
    pub async fn insert_with_generated_id(
        &self,
        table_name: &str,
        mut body: Value,
    ) -> Result<String, String> {
        let endpoint: String = self.endpoint(table_name);

        #[cfg(feature = "nightly")]
        use crate::nightly::print_nightly_warning;
        #[cfg(feature = "nightly")]
        print_nightly_warning();

        let new_id: i64 = generate_random_id();
        body["id"] = json!(new_id);

        let response: Response = match self
            .client
            .post(&endpoint)
            .header(HeadersTypes::ApiKey, &self.api_key)
            .header(
                HeadersTypes::Authorization,
                format!("Bearer {}", &self.api_key),
            )
            .header(HeadersTypes::ContentType, "application/json")
            .header(HeadersTypes::ClientInfo, &crate::client_info())
            .header(HeadersTypes::ContentProfile.as_str(), self.schema.as_str())
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        if response.status().is_success() {
            Ok(new_id.to_string())
        } else if response.status().as_u16() == 409 {
            println!("\x1b[31mError 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.\x1b[0m");

            Err("Error 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.".to_owned())
        } else {
            println!("\x1b[31mError: {:?}\x1b[0m", response);
            Err(response.status().to_string())
        }
    }

    /// Inserts a new row into the specified table.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table.
    /// * `body` - A JSON value containing the data to be inserted.
    ///
    /// # Example
    /// ```ignore
    /// // Initialize the Supabase client
    /// let client = SupabaseClient::new("your_supabase_url", "your_supabase_key");
    ///
    /// // This will insert a new row into the table
    /// let insert_result = client.insert(
    ///   "your_table_name",
    ///   json!(
    ///     {
    ///         "id": "your_id", // Optional
    ///         "column_name": "value"
    ///     }
    ///   )
    /// ).await;
    /// ```
    ///
    /// # Returns
    /// This method returns a `Result<String, String>`. On success, it returns `Ok(String)` with the new row's ID,
    /// and on failure, it returns `Err(String)` with an error message.
    pub async fn insert<T>(&self, table_name: &str, body: T) -> Result<String, String>
    where
        T: serde::Serialize,
    {
        let body = match serde_json::to_value(body) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to serialize body: {}", e)),
        };

        let endpoint: String = self.endpoint(table_name);

        #[cfg(feature = "nightly")]
        use crate::nightly::print_nightly_warning;
        #[cfg(feature = "nightly")]
        print_nightly_warning();

        let response: Response = match self
            .client
            .post(&endpoint)
            .header(HeadersTypes::ApiKey, &self.api_key)
            .header(
                HeadersTypes::Authorization,
                format!("Bearer {}", &self.api_key),
            )
            .header(HeadersTypes::ContentType, "application/json")
            .header(HeadersTypes::ClientInfo, &crate::client_info())
            .header(HeadersTypes::Prefer, "return=representation")
            .header(HeadersTypes::ContentProfile.as_str(), self.schema.as_str())
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        if response.status().is_success() {
            let res_text: String = match response.text().await {
                Ok(text) => text,
                Err(e) => return Err(format!("Failed to get response text: {}", e)),
            };
            let id: String = match serde_json::from_str::<Vec<Value>>(&res_text) {
                Ok(json) => json[0]["id"].to_string(),
                Err(e) => return Err(format!("Failed to parse response text: {}", e)),
            };
            Ok(id)
        } else if response.status().as_u16() == 409 {
            println!("\x1b[31mError 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.\x1b[0m");

            Err("Error 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.".to_owned())
        } else {
            println!("\x1b[31mError: {:?}\x1b[0m", response);
            Err(response.status().to_string())
        }
    }

    /// Inserts a new row into the specified table with a user-defined ID or Supabase backend generated ID.
    /// This method is identical to the `insert` method.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table.
    /// * `body` - A JSON value containing the data to be inserted.
    ///
    /// # Example
    /// ```ignore
    /// // Initialize the Supabase client
    /// let client = SupabaseClient::new("your_supabase_url", "your_supabase_key");
    ///
    /// // This will insert a new row into the table
    /// let insert_result = client.insert(
    ///   "your_table_name",
    ///   json!(
    ///     {
    ///         "id": "your_id", // Optional
    ///         "column_name": "value"
    ///     }
    ///   )
    /// ).await;
    /// ```
    ///
    /// # Returns
    /// This method returns a `Result<(), String>`. On success, it returns `Ok(())`,
    /// and on failure, it returns `Err(String)` with an error message.
    pub async fn insert_without_defined_key<T>(
        &self,
        table_name: &str,
        body: T,
    ) -> Result<String, String>
    where
        T: serde::Serialize,
    {
        self.insert(table_name, body).await
    }

    /// Inserts a row into the specified table if the value is unique and does not exist in the table already.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table.
    /// * `body` - A JSON value containing the data to be inserted.
    ///
    /// ## Example
    /// ```
    /// # use serde_json::json;
    /// # use supabase_rs::SupabaseClient;
    /// #[tokio::main]
    /// async fn main() {
    ///     // Initialize the Supabase client
    ///     let client = SupabaseClient::new("your_supabase_url".to_string(), "your_supabase_key".to_string()).unwrap();
    ///
    ///     // This will insert a new row into the table if the value is unique
    ///     let unique_insert_result = client.insert_if_unique(
    ///         "your_table_name",
    ///         json!({"unique_column_name": "unique_value"})
    ///     ).await;
    /// }
    /// ```
    ///
    /// # Returns
    /// This method returns a `Result<String, String>`. On success, it returns `Ok(String)` with the new row's ID,
    /// and on failure, it returns `Err(String)` with an error message indicating a duplicate entry.
    pub async fn insert_if_unique<T>(&self, table_name: &str, body: T) -> Result<String, String>
    where
        T: serde::Serialize + Clone,
    {
        let body = match serde_json::to_value(body.clone()) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to serialize body: {}", e)),
        };

        let conditions: &serde_json::Map<String, Value> = match body.as_object() {
            Some(map) => map,
            None => {
                println!("\x1b[31mFailed to parse body as JSON object\x1b[0m");
                return Err("Failed to parse body as JSON object".to_owned());
            }
        };

        // Check if any row in the table matches all the column-value pairs in the body
        let mut query: crate::query::QueryBuilder = self.select(table_name);
        for (column_name, column_value) in conditions {
            // turn column_value into a string before passing it to the query
            // ONLY if it's NOT a string
            let column_value_str: String = match column_value {
                Value::String(s) => s.clone(),
                Value::Null
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Array(_)
                | Value::Object(_) => column_value.to_string(),
            };

            // our query is sensitive to the type of the column value
            query = query.eq(column_name, column_value_str.as_str());
        }

        let response: Result<Vec<Value>, String> = query.execute().await;

        // If no existing row matches all conditions, proceed with the insert
        if let Ok(results) = response {
            if results.is_empty() {
                return self.insert(table_name, body).await;
            }
        } else {
            println!("\x1b[31mFailed to execute select query\x1b[0m");
            return Err("Failed to execute select query".to_owned());
        }

        Err("Error 409: Duplicate entry. The values you're trying to insert may already exist in a column with a UNIQUE constraint".to_owned())
    }

    /// Inserts new rows into the specified table in bulk.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table.
    /// * `body` - A vector of serializable values to be inserted.
    ///
    /// # Example
    /// ```ignore
    /// // Initialize the Supabase client
    /// # use serde_json::{json, Value};
    /// # use serde::Serialize;
    ///
    /// // A struct that implements the Serialize trait
    /// #[derive(Serialize)]
    /// pub struct User {
    ///   name: String,
    /// }
    ///
    /// let client = SupabaseClient::new("your_supabase_url", "your_supabase_key");
    ///
    /// // Create the body of the request as a vector of JSON values
    /// let body: Vec<Value> = vec![
    ///     json!({"column_name": "value"}),
    ///     json!({"column_name": "value"}),
    ///     User { name: "Alice".to_string() },
    /// ];
    ///
    /// // This will insert a new row into the table
    /// let insert_result = client.insert("your_table_name", body).await;
    /// ```
    ///
    /// # Returns
    /// This method returns a `Result<(), String>`. On success, it returns `Ok(())`,
    /// and on failure, it returns `Err(String)` with an error message.
    pub async fn bulk_insert<T>(&self, table_name: &str, body: Vec<T>) -> Result<(), String>
    where
        T: serde::Serialize,
    {
        let Ok(body) = serde_json::to_value(body) else {
            return Err("Failed to serialize body".to_owned());
        };
        let endpoint: String = self.endpoint(table_name);

        #[cfg(feature = "nightly")]
        use crate::nightly::print_nightly_warning;
        #[cfg(feature = "nightly")]
        print_nightly_warning();

        let response: Response = match self
            .client
            .post(&endpoint)
            .header(HeadersTypes::ApiKey, &self.api_key)
            .header(
                HeadersTypes::Authorization,
                format!("Bearer {}", &self.api_key),
            )
            .header(HeadersTypes::ContentType, "application/json")
            .header(HeadersTypes::ClientInfo, &crate::client_info())
            .header(HeadersTypes::ContentProfile.as_str(), self.schema.as_str())
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        if response.status().is_success() {
            Ok(())
        } else if response.status().as_u16() == 409 {
            println!("\x1b[31mError 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.\x1b[0m");

            Err("Error 409: Duplicate entry. The value you're trying to insert may already exist in a column with a UNIQUE constraint.".to_owned())
        } else {
            println!("\x1b[31mError: {:?}\x1b[0m", response);
            Err(response.status().to_string())
        }
    }
}
