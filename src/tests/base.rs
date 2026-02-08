//! This module contains tests for the base module
//!
//! It includes tests for various CRUD operations using the `SupabaseClient`.
//! Each test is designed to interact with a Supabase database using the client's methods
//! such as `insert`, `insert_if_unique`, `select`, `select_filter`,
//! as well as the crate helper methods `first` and `single`.
//!
//! ## Tests
//!
//! - `insert`: Tests the insertion of a new row.
//! - `insert_if_unique`: Tests the insertion of a unique row.
//! - `select`: Tests the selection of rows.
//! - `select_filter`: Tests the selection of rows with a specific filter.
//! - `select_with_count`: Tests the selection of rows with a count.
//! - `select_with_count_and_filter`: Tests the selection of rows with a count and a filter.
//! - `delete`: Tests the deletion of a row.
//! - `upsert`: Tests the upsertion of a row.
//! - `update`: Tests the update of a row.
//! - `first`: Tests the crate helper method `.first()` for fetching the first row.
//! - `single`: Tests the crate helper method `.single()` for fetching exactly one row.
//!
//! These tests ensure that the basic functionalities of interacting with a Supabase database
//! are working as expected.
//!

#[cfg(test)]
mod methods {
    // import local method tests
    use crate::tests::methods::{
        delete::delete as test_delete,
        insert::insert as test_insert,
        insert_if_unique_numeric::insert_if_unique_numeric as test_insert_if_unique_numeric,
        insert_if_unique_string::insert_if_unique_string as test_insert_if_unique_string,
        insert_numeric::insert_numeric as test_insert_numeric,
        insert_string::insert_string as test_insert_string,
        query::test_query,
        schema_tests::{
            run_all_schema_tests, test_custom_schema_zeus, test_default_schema,
            test_schema_chaining, test_schema_with_delete, test_schema_with_insert,
            test_schema_with_select, test_schema_with_update, test_schema_with_upsert,
        },
        select::select as test_select,
        select_filter::select_filter as test_select_filter,
        select_first::select_first as test_select_first,
        select_single::select_single as test_select_single,
        select_stacked_queries::select_stacked_queries as test_select_stacked_queries,
        select_with_columns::select_with_columns as test_select_with_columns,
        select_with_count::select_with_count as test_select_with_count,
        update_with_column::update_with_column as test_update_with_column,
        upsert_numeric::upsert_numeric as test_upsert_numeric,
        upsert_string::upsert_string as test_upsert_string,
    };

    /// Tests the `select_first` method of the crate.    
    #[tokio::test]
    async fn select_first() {
        test_select_first().await;
    }

    /// Tests the `select_single` method of the crate.    
    #[tokio::test]
    async fn select_single() {
        test_select_single().await;
    }
    /// Tests the `insert` method of `SupabaseClient`.
    #[tokio::test]
    async fn insert() {
        test_insert().await;
    }

    /// Tests the `insert` with a string method of `SupabaseClient`.
    #[tokio::test]
    async fn insert_string() {
        test_insert_string().await;
    }

    /// Tests the `insert` with a number method of `SupabaseClient`.
    #[tokio::test]
    async fn insert_numeric() {
        test_insert_numeric().await;
    }

    /// Tests the `insert_if_unique` method of `SupabaseClient`.
    #[tokio::test]
    async fn insert_if_unique_string() {
        test_insert_if_unique_string().await;
    }

    /// Tests the `insert_if_unique` method of `SupabaseClient`.
    #[tokio::test]
    async fn insert_if_unique_numeric() {
        test_insert_if_unique_numeric().await;
    }

    /// Tests the `select` method of `SupabaseClient`.
    #[tokio::test]
    async fn select() {
        test_select().await;
    }

    /// Tests the `select_filter` method of `SupabaseClient`.
    #[tokio::test]
    async fn select_filter() {
        test_select_filter().await;
    }

    /// Tests the `select_filter` method of `SupabaseClient`.
    #[tokio::test]
    async fn select_with_columns() {
        test_select_with_columns().await;
    }

    /// Tests the `select_filter` method of `SupabaseClient`.
    #[tokio::test]
    async fn select_with_count() {
        test_select_with_count().await;
    }

    // Tests the `select_filter` method of `SupabaseClient`.
    // #[tokio::test]
    // async fn select_with_count_and_filter() {
    //     test_select_with_count_and_filter().await;
    // }

    /// Tests the `select_filter` method of `SupabaseClient`.
    #[tokio::test]
    async fn delete() {
        test_delete().await;
    }

    /// Tests the `upsert` method of `SupabaseClient`.
    #[tokio::test]
    async fn upsert_string() {
        test_upsert_string().await;
    }

    /// Tests the `upsert` method of `SupabaseClient`.
    #[tokio::test]
    async fn upsert_numeric() {
        test_upsert_numeric().await;
    }

    /// Tests the `update_with_column` method of `SupabaseClient` with a string value.
    #[tokio::test]
    async fn update_with_column() {
        test_update_with_column().await;
    }

    /// Tests the `select_stacked_queries` method of `SupabaseClient`.
    /// This test is used to test the chaining of multiple `eq` methods in a single query.
    #[tokio::test]
    async fn select_stacked_queries() {
        test_select_stacked_queries().await;
    }

    /// Tests the `query` method of `SupabaseClient`.
    /// This test is used to test the query builder.
    #[tokio::test]
    async fn query() {
        test_query().await;
    }

    // Schema functionality tests

    /// Tests that the default schema is set to "public"
    #[tokio::test]
    async fn schema_default() {
        test_default_schema().await;
    }

    /// Tests that custom schema "zeus" can be set
    #[tokio::test]
    async fn schema_custom_zeus() {
        test_custom_schema_zeus().await;
    }

    /// Tests that schema method is chainable and immutable
    #[tokio::test]
    async fn schema_chaining() {
        test_schema_chaining().await;
    }

    /// Tests schema functionality with select operation (Accept-Profile header)
    #[tokio::test]
    async fn schema_with_select() {
        test_schema_with_select().await;
    }

    /// Tests schema functionality with insert operation (Content-Profile header)
    #[tokio::test]
    async fn schema_with_insert() {
        test_schema_with_insert().await;
    }

    /// Tests schema functionality with update operation (Content-Profile header)
    #[tokio::test]
    async fn schema_with_update() {
        test_schema_with_update().await;
    }

    /// Tests schema functionality with upsert operation (Content-Profile header)
    #[tokio::test]
    async fn schema_with_upsert() {
        test_schema_with_upsert().await;
    }

    /// Tests schema functionality with delete operation (Content-Profile header)
    #[tokio::test]
    async fn schema_with_delete() {
        test_schema_with_delete().await;
    }

    /// Runs all schema tests in sequence
    #[tokio::test]
    async fn schema_all_tests() {
        run_all_schema_tests().await;
    }
}
