use crate::tests::methods::init::init;
use crate::SupabaseClient;

pub async fn select_with_columns() {
    /// Performs a select_with_columns operation in an isolated scope.
    async fn select_filter_columns_inner(supabase_client: SupabaseClient) -> crate::Result<()> {
        // Usage example
        let response_inner = supabase_client
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
    let response = select_filter_columns_inner(supabase_client).await;

    response.expect("Select with columns operation should succeed");
}
