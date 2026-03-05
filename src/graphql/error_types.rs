//! # Error Types
//!
//! This module contains functions that generate error messages for various error types.
//!
//! ## Functions
//! - `illegal_table_name`: Generates an error message for an illegal table name.
//! - `illegal_field_name`: Generates an error message for an illegal field name.
//! - `table_does_not_exist`: Generates an error message indicating that a table does not exist.
//! - `field_does_not_exist_on_table`: Generates an error message indicating that a field does not exist on a table.
//! - `table_name_does_not_end_with_collection`: Generates an error message indicating that a table name does not end with "Collection".
//! - `failed_to_parse_json`: Generates an error message indicating that JSON parsing failed.
//! - `print_red`: Prints a message in red color for errors.
//!     
//!

use anyhow::Error as AnyError;

/// Generates an error message for an illegal table name.
///
/// This function checks if the table name starts with a numeric character and generates an error message accordingly.
///
/// # Arguments
///
/// * `table_name` - A string slice that holds the name of the table.
///
/// # Returns
///
/// A `String` containing the error message.
pub fn illegal_table_name(table_name: &str) -> String {
    let amount_of_numbers: usize = table_name.chars().take_while(|c| c.is_numeric()).count();
    let arrow_amount: String = "^".repeat(amount_of_numbers);

    let is_plural: &str = if amount_of_numbers > 1 { "s" } else { "" };

    let error: String = format!(
        "\x1b[1;31mYour Query is invalid!!, there is a number as the first character{is_plural} in the table name: \n\n\x1b[1;34m{table_name}\x1b[0m\n\x1b[1;34m{arrow_amount}\x1b[0m\n\x1b[1;31mremove these numbers and try again\x1b[0m"
    );

    print_red(&error);

    "Illegal table name".to_owned()
}

/// Generates an error message for an illegal field name.
///
/// This function checks if the field name starts with a numeric character and generates an error message accordingly.
///
/// # Arguments
///
/// * `field_name` - A string slice that holds the name of the field.
///
/// # Returns
///
/// A `String` containing the error message.
pub fn illegal_field_name(field_name: &str) -> String {
    let amount_of_numbers: usize = field_name.chars().take_while(|c| c.is_numeric()).count();
    let arrow_amount: String = "^".repeat(amount_of_numbers);

    let is_plural: &str = if amount_of_numbers > 1 { "s" } else { "" };

    let error: String = format!(
        "\x1b[1;31mYour Query is invalid!!, there is a number as the first character{is_plural} in the field name: \n\n\x1b[1;34m{field_name}\x1b[0m\n\x1b[1;34m{arrow_amount}\x1b[0m\n\x1b[1;31mremove these numbers and try again\x1b[0m"
    );

    print_red(&error);

    "Illegal field name".to_owned()
}

/// Generates an error message indicating that a table does not exist.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the table.
///
/// # Returns
///
/// A `String` containing the error message.
pub fn table_does_not_exist(name: &str) -> String {
    let error: String = format!("\x1b[1;31mTable does not exist: {name}\x1b[0m");
    print_red(&error);

    "Table does not exist".to_owned()
}

/// Generates an error message indicating that a field does not exist on a table.
///
/// # Arguments
///
/// * `field` - A string slice that holds the name of the field.
/// * `table` - A string slice that holds the name of the table.
///
/// # Returns
///
/// A `String` containing the error message.
pub fn field_does_not_exist_on_table(field: &str, table: &str) -> String {
    let error: String =
        format!("\x1b[1;31mField does not exist on table: \n{table} -> {field}\x1b[0m");

    print_red(&error);

    "Field does not exist on table".to_owned()
}

/// Generates an error message indicating that a table name does not end with "Collection".
///
/// # Arguments
///
/// * `table_name` - A string slice that holds the name of the table.
///
/// # Returns
///
/// A `String` containing the error message.
pub fn table_name_does_not_end_with_collection(table_name: &str) -> String {
    let error: String = format!(
        "\x1b[1;31mTable name does not end with \x1b[1;34m`Collection`\x1b[1;31m: {table_name}\x1b[0m"
    )
    .to_string();
    let arrow_amount: String = "^".repeat(table_name.len());

    print_red(&error);
    println!("\x1b[1;34m{table_name}\x1b[0m\x1b[1;32mCollection\x1b[0m");
    println!("\x1b[1;34m{arrow_amount}\x1b[0m");
    print_red("Add Collection to the end of the table name and try again");

    "Table name does not end with Collection".to_owned()
}

/// Generates an error message indicating that JSON parsing failed.
///
/// # Arguments
///
/// * `error` - A `String` containing the error message from the JSON parser.
///
/// # Returns
///
/// An `AnyError` containing the formatted error message.
pub fn failed_to_parse_json(error: String) -> AnyError {
    let error: String = format!("Failed to parse JSON: \n{error}");

    print_red(&error);

    AnyError::msg(error)
}

/// Prints a message in red color for errors.
///
/// # Arguments
///
/// * `error` - A string slice that holds the error message.
pub fn print_red(error: &str) {
    println!("\x1b[1;31m{error}\x1b[0m");
}
