pub mod base;

pub mod methods {
    pub mod delete;
    // Unit tests that don't require network. New lightweight suites.
    pub mod init;
    pub mod insert;
    pub mod insert_if_unique_numeric;
    pub mod insert_if_unique_string;
    pub mod insert_numeric;
    pub mod insert_string;
    pub mod query;
    pub mod rpc;
    pub mod select;
    pub mod select_filter;
    pub mod select_first;
    pub mod select_single;
    pub mod select_stacked_queries;
    pub mod select_with_columns;
    pub mod select_with_count;
    pub mod select_with_count_and_filter;
    pub mod update_with_column;
    pub mod upsert_numeric;
    pub mod upsert_string;
    // Schema functionality tests
    pub mod schema_tests;
    // New: pure unit tests
    pub mod unit_filters_and_sort;
    pub mod unit_query_build;
    pub mod unit_utils;
}

#[cfg(test)]
pub fn create_test_supabase_client() -> crate::Result<crate::SupabaseClient> {
    dotenv::dotenv().ok();

    let supabase_url = std::env::var("SUPABASE_URL")?;
    let supabase_key = std::env::var("SUPABASE_KEY")?;

    crate::SupabaseClient::new(supabase_url, supabase_key)
}
