//! Response handling for Supabase API calls.
//!
//! This module handles both successful and error responses from the Supabase API,
//! providing structured error parsing that includes helpful hints for resolving issues.
//!
//! ## Error Response Structure
//!
//! When a Supabase API call fails, this module will attempt to parse the JSON error
//! response to extract:
//! - **code**: Error code (e.g., "42703" for column not found)
//! - **message**: Main error description
//! - **details**: Additional error context (optional)
//! - **hint**: Helpful suggestions for fixing the error (optional)
//!
//! ## Example Error Response
//!
//! ```json
//! {
//!   "code": "42703",
//!   "details": null,
//!   "hint": "Perhaps you meant to reference the column \"jortt_invoices.amount_side\".",
//!   "message": "column jortt_invoices.account_side does not exist"
//! }
//! ```
//!
//! This would result in an error message like:
//! ```text
//! Error 42703 (400): column jortt_invoices.account_side does not exist
//! Hint: Perhaps you meant to reference the column "jortt_invoices.amount_side".
//! ```

use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::errors::{
    api_key_missing_error, authorization_failed_error, invalid_query_error, unknown_error,
};

/// Represents a structured error response from the Supabase API.
///
/// This struct captures all error information that Supabase returns, including
/// the new `hint` field that provides suggestions for fixing query errors.
#[derive(Debug, Serialize, Deserialize)]
pub struct SupabaseErrorResponse {
    /// The error code (e.g., "42703" for column does not exist)
    pub code: Option<String>,
    /// The main error message
    pub message: String,
    /// Additional details about the error
    pub details: Option<String>,
    /// Helpful hint for resolving the error (e.g., "Perhaps you meant to reference the column...")
    pub hint: Option<String>,
}

/// Handles the response from the Supabase API.
pub async fn handle_response(response: Response) -> Result<Vec<Value>, String> {
    if response.status().is_success() {
        let headers: &reqwest::header::HeaderMap = response.headers();
        let content_range_option: Option<&str> =
            headers.get("content-range").and_then(|v| v.to_str().ok());

        let mut total_records: Option<i32> = None;

        if let Some(content_range) = content_range_option {
            total_records = content_range
                .split('/')
                .nth(1)
                .and_then(|v| v.parse::<i32>().ok());
        }

        let mut records: Vec<Value> = match response.json::<Vec<Value>>().await {
            Ok(records) => records,
            Err(error) => return Err(error.to_string()),
        };

        if let Some(count) = total_records {
            records.push(json!({"total_records_count": count}));
        }
        Ok(records)
    } else {
        // Store the status code before consuming the response
        let status_code = response.status().as_u16();

        // Try to parse the structured error response from Supabase
        let error_text = response.text().await.unwrap_or_default();

        // Attempt to parse as structured error response
        if let Ok(error_response) = serde_json::from_str::<SupabaseErrorResponse>(&error_text) {
            // Format the error message with all available information
            let mut error_msg = format!("Error: {}", error_response.message);

            if let Some(code) = &error_response.code {
                error_msg = format!(
                    "Error {} ({}): {}",
                    code, status_code, error_response.message
                );
            }

            if let Some(details) = &error_response.details {
                error_msg.push_str(&format!("\nDetails: {}", details));
            }

            if let Some(hint) = &error_response.hint {
                error_msg.push_str(&format!("\nHint: {}", hint));
            }

            return Err(error_msg);
        }

        // Fallback to original error handling if JSON parsing fails
        let error_message = match status_code {
            401 => authorization_failed_error()
                .await
                .map_err(|e| e.to_string()),
            403 => api_key_missing_error().await.map_err(|e| e.to_string()),
            400 => invalid_query_error().await.map_err(|e| e.to_string()),
            _ => unknown_error().await.map_err(|e| e.to_string()),
        };

        // If we have error text but couldn't parse it as structured error, include it
        if !error_text.is_empty() {
            let fallback_error = error_message
                .expect_err("Expected an error, but got Ok when handling Supabase error response");
            return Err(format!("{}\nResponse: {}", fallback_error, error_text));
        }

        // Convert the error to the expected type
        Err(error_message
            .expect_err("Expected an error, but got Ok when handling Supabase error response"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_supabase_error_response_with_hint() {
        let error_json = json!({
            "code": "42703",
            "details": null,
            "hint": "Perhaps you meant to reference the column \"jortt_invoices.amount_side\".",
            "message": "column jortt_invoices.account_side does not exist"
        });

        let error_response: SupabaseErrorResponse = serde_json::from_value(error_json).unwrap();

        assert_eq!(error_response.code, Some("42703".to_owned()));
        assert_eq!(
            error_response.message,
            "column jortt_invoices.account_side does not exist"
        );
        assert_eq!(error_response.details, None);
        assert_eq!(
            error_response.hint,
            Some(
                "Perhaps you meant to reference the column \"jortt_invoices.amount_side\"."
                    .to_owned()
            )
        );
    }

    #[test]
    fn test_supabase_error_response_without_hint() {
        let error_json = json!({
            "code": "23505",
            "message": "duplicate key value violates unique constraint",
            "details": "Key (email)=(test@example.com) already exists."
        });

        let error_response: SupabaseErrorResponse = serde_json::from_value(error_json).unwrap();

        assert_eq!(error_response.code, Some("23505".to_owned()));
        assert_eq!(
            error_response.message,
            "duplicate key value violates unique constraint"
        );
        assert_eq!(
            error_response.details,
            Some("Key (email)=(test@example.com) already exists.".to_owned())
        );
        assert_eq!(error_response.hint, None);
    }
}
