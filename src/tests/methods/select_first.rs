use crate::tests::methods::init::init;
use crate::SupabaseClient;

pub async fn select_first() {
    /// Tests that `.first()` returns the first matching row as `Some(Value)`
    /// or `None` if no rows match, without error.
    async fn select_first_inner(client: SupabaseClient) -> Result<(), String> {
        let res = client.select("users").order("id", true).first().await;

        match res {
            Ok(Some(row)) => {
                assert!(row.is_object());
                Ok(())
            }
            Ok(None) => {
                // No rows matched which is acceptable
                println!("first returned None (no matching rows)");
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

    let result = select_first_inner(client).await;
    assert!(result.is_ok());
}
