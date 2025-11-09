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
    /// Returns a `Result<String, String>`:
    /// - `Ok(String)` containing the ID of the row if found.
    /// - `Err(String)` containing an error message if the query fails or if no matching row is found.
    ///
    /// ## Examples
    /// ```rust
    /// # use supabase_rs::SupabaseClient;
    /// #[tokio::main]
    /// async fn main() {
    ///     let supabase_client = SupabaseClient::new(
    ///         "your_supabase_url".to_string(),
    ///         "your_supabase_key".to_string()
    ///     ).unwrap();
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
    ) -> Result<String, String> {
        let response: Result<Vec<Value>, String> = self
            .select(&table_name)
            .eq(&column_name, &email)
            .execute()
            .await;

        match response {
            Ok(response) => {
                if !response.is_empty() {
                    let id: String = response[0]["id"].to_string();
                    Ok(id)
                } else {
                    Err("No matching record found".to_owned())
                }
            }
            Err(error) => Err(error),
        }
    }
}
