use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::{json, Value};

pub async fn update_with_column() {
    /// Performs a select_filter operation in an isolated scope.
    async fn update_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example

        let username: String = "update_me".to_string();

        let updated_body: Value = json!({
            "age": 29
        });

        let response_inner: Result<String, String> = supabase_client
            .update_with_column_name("users", "username", &username, updated_body)
            .await;

        match response_inner {
            Ok(_) => Ok(()),
            Err(error) => {
                println!("Error: {:?}", error);
                Err(error)
            }
        }
    }

    let supabase_client: SupabaseClient = match init().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!(
                "\x1b[31mFailed to initialize Supabase client: {:?}\x1b[0m",
                e
            );
            return;
        }
    };
    let response: Result<(), String> = update_inner(supabase_client).await;

    assert!(response.is_ok());
}
