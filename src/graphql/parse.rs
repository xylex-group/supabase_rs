use crate::graphql::error_types::table_name_does_not_end_with_collection;
use anyhow::{Error, Result};
use serde_json::Value;

pub fn parse_outer(query: &Value) -> bool {
    if let Some(query_str) = query.as_str() {
        let query_str = query_str.trim();

        let start_1: &str = r#"{"#;
        let end_1: &str = "}";

        query_str.starts_with(start_1) && query_str.ends_with(end_1)
    } else {
        true
    }
}

/// Get the table name from the query
///
/// # Arguments
/// - `query` - A JSON Value containing the query
///
/// # Returns
/// A `Result` containing the table name as a `String` if successful, or an `Error` if the outer structure is invalid
///
pub fn get_table_name(query: &Value) -> Result<String, Error> {
    if parse_outer(query) {
        let query_str: &str = query["query"].as_str().unwrap_or("");
        // Remove all the { } and then get the first alphanumeric word from the query
        let query_str: String = query_str.replace(['{', '}'], "").trim().to_owned();
        let query_str: Vec<&str> = query_str.split_whitespace().collect();
        let mut table_name: String = query_str.first().unwrap_or(&"").to_string();

        // remove all beyond the last alphanumeric char
        if let Some(pos) = table_name.find(|c: char| !c.is_alphanumeric()) {
            table_name = table_name[..pos].to_string();
        }

        // if the table name doesnt end with Collection, add it
        if !table_name.ends_with("Collection") {
            table_name_does_not_end_with_collection(&table_name);
        }

        Ok(table_name)
    } else {
        Err(Error::msg("Invalid outer structure"))
    }
}
