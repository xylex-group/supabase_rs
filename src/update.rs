//! # Update and Upsert Operations
//!
//! This module provides comprehensive functionality for modifying existing records in Supabase tables.
//! It supports standard updates, upserts (insert or update), and flexible column-based targeting.
//!
//! ## 🎯 Core Features
//!
//! - **[`update`]**: Modify existing records by ID
//! - **[`update_with_column_name`]**: Update records using custom column matching
//! - **[`upsert`]**: Insert new record or update if it exists
//! - **[`upsert_without_defined_key`]**: Upsert with automatic conflict resolution
//!
//! ## 🏗️ Operation Types
//!
//! | Method | Targeting | Behavior | Return Type | Use Case |
//! |--------|-----------|----------|-------------|----------|
//! | `update` | By ID | Updates existing record | `Result<String, String>` | Standard updates |
//! | `update_with_column_name` | By custom column | Updates matching record | `Result<String, String>` | Flexible targeting |
//! | `upsert` | By ID | Insert or update | `Result<String, String>` | Idempotent operations |
//! | `upsert_without_defined_key` | Auto-detect | Insert or update | `Result<(), String>` | Conflict resolution |
//!
//! ## 🔧 Conflict Resolution
//!
//! ### Update vs Upsert Decision Matrix
//!
//! | Scenario | Recommended Method | Reason |
//! |----------|-------------------|---------|
//! | Record definitely exists | `update` | Fastest, fails fast if missing |
//! | Record may or may not exist | `upsert` | Handles both cases gracefully |
//! | Bulk operations with mixed states | `upsert_without_defined_key` | Automatic conflict handling |
//! | Need to update by non-ID field | `update_with_column_name` | Flexible targeting |
//!
//! ## 📖 Usage Examples
//!
//! ### Basic Update Operations
//!
//! ```rust,no_run
//! use supabase_rs::SupabaseClient;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Update by ID (most common)
//! let updated_id = client.update("users", "123", json!({
//!     "name": "Alice Smith",
//!     "last_login": "2024-01-15T10:30:00Z",
//!     "login_count": 42
//! })).await?;
//!
//! println!("Updated user with ID: {}", updated_id);
//! # Ok(())
//! # }
//! ```
//!
//! ### Update by Custom Column
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Update user by email instead of ID
//! client.update_with_column_name(
//!     "users",
//!     "email",                    // Column to match on
//!     "alice@example.com",        // Value to match
//!     json!({
//!         "verified": true,
//!         "verification_date": "2024-01-15T10:30:00Z"
//!     })
//! ).await?;
//!
//! // Update session by token
//! client.update_with_column_name(
//!     "sessions",
//!     "token",
//!     "abc123xyz",
//!     json!({ "last_accessed": "2024-01-15T10:30:00Z" })
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Upsert Operations
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // Upsert with explicit ID
//! let result_id = client.upsert("user_preferences", "user_123", json!({
//!     "theme": "dark",
//!     "language": "en",
//!     "notifications": true
//! })).await?;
//!
//! // Upsert without predefined key (uses Supabase's conflict resolution)
//! client.upsert_without_defined_key("analytics", json!({
//!     "user_id": "123",
//!     "event": "page_view",
//!     "timestamp": "2024-01-15T10:30:00Z",
//!     "page": "/dashboard"
//! })).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## ⚡ Performance Best Practices
//!
//! ### Efficient Update Patterns
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! // ✅ Good: Update only changed fields
//! client.update("users", "123", json!({
//!     "last_login": "2024-01-15T10:30:00Z"  // Only update what changed
//! })).await?;
//!
//! // ✅ Good: Use upsert for idempotent operations
//! client.upsert("settings", "user_123", json!({
//!     "theme": "dark"  // Safe to run multiple times
//! })).await?;
//!
//! // ⚠️ Consider: Batch updates when possible
//! // For multiple updates, consider using transactions or bulk operations
//! # Ok(())
//! # }
//! ```
//!
//! ## 🚨 Error Handling Strategies
//!
//! ```rust,no_run
//! # use supabase_rs::SupabaseClient;
//! # use serde_json::json;
//! # async fn example() -> Result<(), String> {
//! # let client = SupabaseClient::new("url".to_string(), "key".to_string()).unwrap();
//! match client.update("users", "123", json!({"name": "New Name"})).await {
//!     Ok(id) => println!("✅ Updated user {}", id),
//!     Err(err) => {
//!         if err.contains("404") {
//!             println!("⚠️ User not found, consider using upsert");
//!         } else if err.contains("403") {
//!             println!("🚫 Permission denied, check RLS policies");
//!         } else {
//!             println!("❌ Update failed: {}", err);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
use crate::request::headers::HeadersTypes;
use crate::SupabaseClient;
use reqwest::Response;
use serde_json::{json, Value};

impl SupabaseClient {
    /// Updates a row in the table, based on the id
    pub async fn update(&self, table_name: &str, id: &str, body: Value) -> Result<String, String> {
        Self::update_with_column_name(self, table_name, "id", id, body).await
    }

    /// Updates a row in the table, based on the column name
    pub async fn update_with_column_name(
        &self,
        table_name: &str,
        column_name: &str,
        id: &str,
        body: Value,
    ) -> Result<String, String> {
        // endpoint and client construction
        let endpoint: String = self.endpoint(table_name);
        let endpoint: String = format!("{endpoint}?{column_name}=eq.{id}");

        let response: Response = match self
            .client
            .patch(&endpoint)
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
            Err(error) => return Err(error.to_string()),
        };

        if response.status().is_success() {
            Ok(id.to_owned())
        } else {
            Err(response.status().to_string())
        }
    }

    /// Creates a row in the table, or updates if the id already exists
    pub async fn upsert(
        &self,
        table_name: &str,
        id: &str,
        mut body: Value,
    ) -> Result<String, String> {
        body["id"] = json!(id);
        match self.upsert_without_defined_key(table_name, body).await {
            Ok(_) => Ok(id.to_owned()),
            Err(e) => Err(e),
        }
    }

    /// Creates a row in the table, or updates if the row already exists
    ///
    /// This method does not require a defined key in the body unlike the `upsert` method.
    pub async fn upsert_without_defined_key(
        &self,
        table_name: &str,
        body: Value,
    ) -> Result<(), String> {
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
            .header(HeadersTypes::Prefer.as_str(), "resolution=merge-duplicates")
            .header(HeadersTypes::Prefer.as_str(), "return=representation")
            .body(body.to_string())
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };

        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.status().to_string())
        }
    }
}
