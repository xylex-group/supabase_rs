use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::json;

pub async fn insert_if_unique_numeric() {
    /// Performs an insert_if_unique operation in an isolated scope.
    async fn insert_if_unique_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example
        let random_number: u64 = rand::random::<u64>();

        let response_inner: Result<String, String> = supabase_client
            .insert_if_unique(
                "users",
                json!({
                    "age": random_number % 100,
                    "username": format!("unique_num_{}", random_number)
                }),
            )
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
    let response: Result<(), String> = insert_if_unique_inner(supabase_client).await;

    assert!(response.is_ok());
}
