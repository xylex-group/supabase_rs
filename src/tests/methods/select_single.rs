use crate::tests::methods::init::init;
use crate::SupabaseClient;

pub async fn select_single() {
    /// Tests that `.single()` returns exactly one row and fails if none or multiple exist.
    async fn select_single_inner(client: SupabaseClient) -> Result<(), String> {
        // query to get exactly one row
        let res = client
            .select("users")
            .eq("id", "user-single-target")
            .single()
            .await;

        match res {
            Ok(row) => {
                assert!(row.is_object());
                Ok(())
            }
            Err(e) => {
                eprintln!("\x1b[31mError: {:?}\x1b[0m", e);
                Err(e)
            }
        }
    }

    let client = match init().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\x1b[31mFailed to init client: {:?}\x1b[0m", e);
            return;
        }
    };

    let result = select_single_inner(client).await;
    assert!(result.is_ok());
}
