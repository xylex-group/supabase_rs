use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::Value;

pub async fn select_with_columns() {
    /// Performs a select_with_columns operation in an isolated scope.
    async fn select_filter_columns_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example
        let response_inner: Result<Vec<Value>, String> = supabase_client
            .select("users")
            .columns(["email", "username"].to_vec())
            .eq("username", "alice")
            .execute()
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
    let response: Result<(), String> = select_filter_columns_inner(supabase_client).await;

    response.expect("Select with columns operation should succeed");
}
