use crate::Result;
use crate::SupabaseClient;
use dotenv::dotenv;
use std::env::var;

pub async fn init() -> Result<SupabaseClient> {
    dotenv().ok();

    let supabase_url: String = var("SUPABASE_URL")?;
    let supabase_key: String = var("SUPABASE_KEY")?;

    SupabaseClient::new(supabase_url, supabase_key)
}

#[cfg(feature = "rpc")]
pub async fn setup_rpc_functions() -> Result<()> {
    use std::fs;
    use tokio_postgres::{Config, NoTls};

    dotenv().ok();

    // Try to get database credentials from environment variables
    // Default to local Supabase credentials
    let db_host = var("SUPABASE_DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let db_port = var("SUPABASE_DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_user = var("SUPABASE_DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = var("SUPABASE_DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let db_name = var("SUPABASE_DB_NAME").unwrap_or_else(|_| "postgres".to_string());

    // Read SQL file
    let sql_content = fs::read_to_string("src/tests/setup_rpc.sql").map_err(|e| {
        eprintln!("Failed to read SQL file: {}", e);
        crate::errors::ErrorTypes::UnknownError
    })?;

    // Connect to database
    let mut config = Config::new();
    config
        .host(&db_host)
        .port(db_port.parse().unwrap_or(5432))
        .user(&db_user)
        .password(&db_password)
        .dbname(&db_name);

    let (client, connection) = config.connect(NoTls).await.map_err(|e| {
        eprintln!("Failed to connect to database: {}", e);
        crate::errors::ErrorTypes::UnknownError
    })?;

    // Spawn connection driver
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Execute SQL
    client.batch_execute(&sql_content).await.map_err(|e| {
        eprintln!("Failed to execute RPC setup SQL: {}", e);
        crate::errors::ErrorTypes::UnknownError
    })?;

    Ok(())
}
