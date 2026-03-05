use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::json;

pub async fn insert_if_unique_string() {
    /// Performs an insert_if_unique operation in an isolated scope.
    async fn insert_if_unique_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example
        let random_string: String = rand::random::<u64>().to_string();

        let response_inner: Result<String, String> = supabase_client
            .insert_if_unique(
                "users",
                json!({
                    "username": format!("unique_str_{}", random_string),
                    "email": format!("unique_{}@test.com", random_string)
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

    response.expect("insert_if_unique_inner failed");
}
