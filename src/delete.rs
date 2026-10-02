//! # Delete Operations
//!
//! This module provides comprehensive functionality for removing records from Supabase tables.
//! It supports deletion by ID, custom column matching, and includes safety measures for
//! preventing accidental data loss.
//!
//! ## 🎯 Core Features
//!
//! - **[`delete`]**: Remove records by ID (most common)
//! - **[`delete_without_defined_key`]**: Remove records by custom column matching
//! - **Safety Measures**: Built-in safeguards against accidental bulk deletions
//! - **Error Handling**: Clear feedback for failed operations
//!
//! ## 🏗️ Operation Types
//!
//! | Method | Targeting | Safety Level | Use Case |
//! |--------|-----------|--------------|----------|
//! | `delete` | By ID | ✅ Safe | Standard record removal |
//! | `delete_without_defined_key` | By custom column | ⚠️ Use carefully | Flexible targeting |
//!
//! ## ⚠️ Safety Considerations
//!
//! - **Single Record Focus**: Both methods target individual records
//! - **No Bulk Delete**: Prevents accidental mass deletions
//! - **Explicit Targeting**: Requires specific column/value pairs
//! - **Error Feedback**: Clear messages for failed operations
//!
//! ## 📖 Usage Examples
//!
//! ### Basic Delete Operations
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//!
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Delete by ID (most common and safest)
//! client.delete("users", "123").await?;
//! println!("✅ User deleted successfully");
//!
//! // Delete with error handling
//! match client.delete("posts", "456").await {
//!     Ok(_) => println!("✅ Post deleted"),
//!     Err(err) => {
//!         if err.contains("404") {
//!             println!("⚠️ Post not found (may already be deleted)");
//!         } else {
//!             println!("❌ Delete failed: {}", err);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Delete by Custom Column
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Delete session by token
//! client.delete_without_defined_key("sessions", "token", "abc123xyz").await?;
//!
//! // Delete user by email (use with caution)
//! client.delete_without_defined_key("users", "email", "user@example.com").await?;
//!
//! // Delete expired records
//! client.delete_without_defined_key("temp_data", "expires_at", "2024-01-01").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🚨 Error Handling and Recovery
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Comprehensive error handling for delete operations
//! async fn safe_delete(client: &SupabaseClient, table: &str, id: &str) -> Result<(), String> {
//!     match client.delete(table, id).await {
//!         Ok(_) => {
//!             println!("✅ Record deleted successfully");
//!             Ok(())
//!         },
//!         Err(err) => {
//!             if err.contains("404") {
//!                 println!("⚠️ Record not found (may already be deleted)");
//!                 Ok(()) // Treat as success - desired state achieved
//!             } else if err.contains("403") {
//!                 println!("🚫 Permission denied - check RLS policies");
//!                 Err("Insufficient permissions for delete operation".to_string())
//!             } else if err.contains("409") {
//!                 println!("⚠️ Cannot delete - record has dependent references");
//!                 Err("Delete blocked by foreign key constraints".to_string())
//!             } else {
//!                 println!("❌ Unexpected delete error: {}", err);
//!                 Err(err)
//!             }
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 🛡️ Best Practices
//!
//! ### Safe Deletion Patterns
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // ✅ Good: Verify record exists before deletion
//! let users = client.select("users").eq("id", "123").execute().await?;
//! if !users.is_empty() {
//!     client.delete("users", "123").await?;
//!     println!("User deleted after verification");
//! } else {
//!     println!("User not found, no deletion needed");
//! }
//!
//! // ✅ Good: Use specific column matching for safety
//! client.delete_without_defined_key("sessions", "user_id", "123").await?;
//!
//! // ⚠️ Consider: Soft deletes for important data
//! // Instead of hard delete, mark as deleted
//! // client.update("users", "123", json!({"deleted_at": "2024-01-15T10:30:00Z"})).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 🔄 Alternative Patterns
//!
//! ### Soft Delete Implementation
//!
//! For critical data, consider implementing soft deletes:
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Soft delete - mark as deleted instead of removing
//! client.update("users", "123", json!({
//!     "deleted_at": "2024-01-15T10:30:00Z",
//!     "deleted_by": "admin_user_456"
//! })).await?;
//!
//! // Query active records only
//! let active_users = client
//!     .select("users")
//!     .eq("deleted_at", "is.null")
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use crate::request::headers::HeadersTypes;
use crate::SupabaseClient;
use reqwest::Response;
use serde_json::json;

impl SupabaseClient {
    /// Deletes a row in the specified table based on the provided ID.
    ///
    /// # Arguments
    /// * `table_name` - A string slice that holds the name of the table from which to delete.
    /// * `id` - A string slice that holds the ID of the row to delete.
    /// * `body` - A JSON value containing the body of the request, typically specifying conditions for deletion.
    ///
    /// # Returns
    /// This method returns a `Result<(), String>`. On success, it returns `Ok(())`, and on failure, it returns
    /// `Err(String)` with an error message.
    ///
    /// # Examples
    /// ```
    /// use serde_json::json;
    /// use supabase_rs::SupabaseClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = SupabaseClient::new(
    ///         "your_supabase_url".to_string(),
    ///         "your_supabase_key".to_string()
    ///     ).unwrap();
    ///     let result = client.delete("your_table_name", "row_id").await;
    ///     match result {
    ///         Ok(_) => println!("Row deleted successfully"),
    ///         Err(e) => println!("Failed to delete row: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn delete(
        &self,
        table_name: &str,
        id: &str,
        //body: Value
    ) -> Result<(), String> {
        // Construct the endpoint URL for the delete operation
        let endpoint: String = self.endpoint(table_name);
        let endpoint: String = format!("{endpoint}?id=eq.{id}");

        #[cfg(feature = "nightly")]
        use crate::nightly::print_nightly_warning;
        #[cfg(feature = "nightly")]
        print_nightly_warning();

        let body: serde_json::Value = json!({}); // this is temporary, will be used for more complex queries

        // Send the delete request and handle the response
        let response: Response = match self
            .client
            .delete(&endpoint)
            .header(HeadersTypes::ApiKey, &self.api_key)
            .header(
                HeadersTypes::Authorization,
                format!("Bearer {}", self.api_key),
            )
            .header(HeadersTypes::ContentType, "application/json")
            .header(HeadersTypes::ClientInfo, &crate::client_info())
            .header(HeadersTypes::ContentProfile.as_str(), self.schema.as_str())
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => return Err(error.to_string()),
        };

        // Check the HTTP status code of the response
        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().to_string())
        }
    }

    pub async fn delete_without_defined_key(
        &self,
        table_name: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        // Construct the endpoint URL for the delete operation with dynamic key
        let endpoint: String = self.endpoint(table_name);
        let endpoint: String = format!("{endpoint}?{key}=eq.{value}");

        #[cfg(feature = "nightly")]
        use crate::nightly::print_nightly_warning;
        #[cfg(feature = "nightly")]
        print_nightly_warning();

        let body: serde_json::Value = json!({});

        // Send the delete request and handle the response
        let response: Response = match self
            .client
            .delete(&endpoint)
            .header(HeadersTypes::ApiKey, &self.api_key)
            .header(
                HeadersTypes::Authorization,
                format!("Bearer {}", self.api_key),
            )
            .header(HeadersTypes::ContentType, "application/json")
            .header(HeadersTypes::ClientInfo, &crate::client_info())
            .header(HeadersTypes::ContentProfile.as_str(), self.schema.as_str())
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => return Err(error.to_string()),
        };

        // Check the HTTP status code of the response
        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().to_string())
        }
    }
}
