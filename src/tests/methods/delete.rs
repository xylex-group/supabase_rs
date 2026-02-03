use crate::tests::methods::init::init;
use crate::SupabaseClient;

pub async fn delete() {
    /// Performs a select_filter operation in an isolated scope.
    async fn delete_inner(supabase_client: SupabaseClient) -> Result<(), String> {
        // Usage example
        let response_inner: Result<(), String> =
            supabase_client.delete("users", "user-delete-target").await;

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
    let response: Result<(), String> = delete_inner(supabase_client).await;

    assert!(response.is_ok());
}
