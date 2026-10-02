use crate::errors::{Error, Result};
use crate::SupabaseClient;

use serde_json::Value;

impl SupabaseClient {
    /// Retrieves the ID of a row from a specified table based on a matching email address.
    ///
    /// ## Arguments
    /// * `supabase_client` - An instance of `SupabaseClient` used to interact with the database.
    /// * `email` - A `String` representing the email address to match in the query.
    /// * `table_name` - A `String` specifying the name of the table to query.
    /// * `column_name` - A `String` specifying the name of the column to match against the email.
    ///
    /// ## Returns
    /// Returns the ID of the matching row, or a typed SDK error if the query fails or no row matches.
    ///
    /// ## Examples
    /// ```rust,no_run
    /// # use supabase_rs::SupabaseClient;
    /// #[tokio::main]
    /// async fn main() {
    ///     let supabase_client = SupabaseClient::new("http://localhost", "your_supabase_key").unwrap();
    ///     let email = "example@email.com".to_string();
    ///     let table_name = "users".to_string();
    ///     let column_name = "email".to_string();
    ///     match supabase_client.get_id(email, table_name, column_name).await {
    ///         Ok(id) => println!("Found ID: {}", id),
    ///         Err(e) => println!("Error: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn get_id(
        &self,
        email: String,
        table_name: String,
        column_name: String,
    ) -> Result<String> {
        let response: Result<Vec<Value>> = self
            .select(&table_name)
            .eq(&column_name, &email)
            .execute()
            .await;

        match response {
            Ok(response) => response
                .first()
                .and_then(|row| row.get("id"))
                .map(|id| match id {
                    Value::String(id) => id.clone(),
                    Value::Number(id) => id.to_string(),
                    Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => {
                        id.to_string()
                    }
                })
                .ok_or_else(|| Error::InvalidInput("no matching record found".to_owned())),
            Err(error) => Err(error),
        }
    }
}
