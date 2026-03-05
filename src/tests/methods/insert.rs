use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::json;

pub async fn insert() {
    /// Performs an insert operation in an isolated scope.
    async fn insert_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example
        let response_inner: Result<String, String> = supabase_client
            .insert(
                "users",
                json!({
                    "email": "test@example.com",
                    "username": "testuser",
                    "age": 25
                }),
            )
            .await;

        match response_inner {
            Ok(_) => Ok(()),
            Err(error) => {
                eprintln!("\x1b[31mError: {:?}\x1b[0m", error);
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

    let response: Result<(), String> = insert_inner(supabase_client).await;

    response.expect("Insert operation failed");
}
