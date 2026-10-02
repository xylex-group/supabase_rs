use crate::tests::methods::init::init;
use crate::SupabaseClient;
use serde_json::json;

pub async fn upsert_numeric() {
    /// Performs a select_filter operation in an isolated scope.
    async fn upsert_inner(supabase_client: SupabaseClient) -> crate::Result<()> {
        // Usage example

        let id: String = "user-upsert-target".to_owned();

        let response_inner = supabase_client
            .upsert(
                "users",
                &id,
                json!({
                    "age": 99
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
    let response = upsert_inner(supabase_client).await;

    response.expect("Upsert numeric operation should succeed");
}
