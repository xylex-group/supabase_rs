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

use reqwest::{header::CONTENT_TYPE, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::errors::{ApiError, Error, Result};

/// Response data and optional exact row count returned by PostgREST.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseData<T> {
    /// The decoded response body.
    pub data: T,
    /// Exact total row count from `Content-Range`, when present and valid.
    pub count: Option<u64>,
}

/// Number of rows affected by a single-table mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationResult {
    /// Number of rows affected, from PostgREST count metadata or returned rows.
    pub affected: u64,
}

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

/// Decodes a Supabase response while keeping protocol metadata out of the data body.
pub async fn handle_response(response: Response) -> Result<ResponseData<Vec<Value>>> {
    let payload = response_payload(response).await?;
    let data = if payload.body.is_empty() {
        Vec::new()
    } else {
        parse_json(&payload)?
    };
    Ok(ResponseData {
        data,
        count: payload.count,
    })
}

/// Decodes a successful JSON response without requiring an array body.
pub async fn handle_json_response(response: Response) -> Result<ResponseData<Value>> {
    let payload = response_payload(response).await?;
    let data = if payload.body.is_empty() {
        Value::Null
    } else {
        parse_json(&payload)?
    };
    Ok(ResponseData {
        data,
        count: payload.count,
    })
}

struct ResponsePayload {
    status: reqwest::StatusCode,
    content_type: Option<String>,
    count: Option<u64>,
    body: Vec<u8>,
}

async fn response_payload(response: Response) -> Result<ResponsePayload> {
    let status = response.status();
    let content_range = response
        .headers()
        .get("content-range")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = response.bytes().await?.to_vec();

    if !status.is_success() {
        return Err(provider_error(status, &body));
    }
    let count = match content_range {
        Some(value) => {
            parse_content_range_count(&value).map_err(|reason| Error::UnexpectedResponse {
                message: format!("malformed Content-Range header: {reason}"),
                status: Some(status.as_u16()),
                body: bounded_body(&body, 8 * 1024),
            })?
        }
        None => None,
    };

    Ok(ResponsePayload {
        status,
        content_type,
        count,
        body,
    })
}

fn parse_json<T: DeserializeOwned>(payload: &ResponsePayload) -> Result<T> {
    const MAX_ERROR_BODY_BYTES: usize = 8 * 1024;
    let is_json = payload.content_type.as_deref().is_some_and(|value| {
        let media_type = value.split(';').next().unwrap_or_default().trim();
        media_type.eq_ignore_ascii_case("application/json")
            || media_type.to_ascii_lowercase().ends_with("+json")
    });
    if !is_json {
        return Err(Error::UnexpectedResponse {
            message: format!("expected JSON content type, got {:?}", payload.content_type),
            status: Some(payload.status.as_u16()),
            body: bounded_body(&payload.body, MAX_ERROR_BODY_BYTES),
        });
    }
    Ok(serde_json::from_slice(&payload.body)?)
}

fn parse_content_range_count(
    content_range: &str,
) -> std::result::Result<Option<u64>, &'static str> {
    let (range, total) = content_range
        .rsplit_once('/')
        .ok_or("missing count separator")?;
    let total = if total == "*" {
        None
    } else {
        Some(total.parse::<u64>().map_err(|_| "invalid total")?)
    };
    if range == "*" {
        return Ok(total);
    }
    let (start, end) = range.split_once('-').ok_or("invalid row range")?;
    let start = start.parse::<u64>().map_err(|_| "invalid range start")?;
    let end = end.parse::<u64>().map_err(|_| "invalid range end")?;
    if start > end {
        return Err("range start exceeds range end");
    }
    Ok(total)
}

fn bounded_body(body: &[u8], max_bytes: usize) -> Option<String> {
    if body.is_empty() {
        return None;
    }
    let end = body.len().min(max_bytes);
    Some(String::from_utf8_lossy(&body[..end]).into_owned())
}

#[cfg(feature = "storage")]
pub(crate) async fn check_http_response(response: Response) -> Result<Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.bytes().await?;
    Err(provider_error(status, &body))
}

fn provider_error(status: reqwest::StatusCode, body: &[u8]) -> Error {
    const MAX_ERROR_BODY_BYTES: usize = 8 * 1024;
    let parsed = serde_json::from_slice::<SupabaseErrorResponse>(body).ok();
    Error::Api(Box::new(ApiError {
        status: status.as_u16(),
        code: parsed.as_ref().and_then(|error| error.code.clone()),
        message: parsed
            .as_ref()
            .map(|error| error.message.clone())
            .unwrap_or_else(|| format!("HTTP response {status}")),
        details: parsed.as_ref().and_then(|error| error.details.clone()),
        hint: parsed.and_then(|error| error.hint),
        raw_response: bounded_body(body, MAX_ERROR_BODY_BYTES),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn content_range_count_accepts_u64_and_rejects_malformed_ranges() {
        assert_eq!(
            parse_content_range_count("0-9/4294967296"),
            Ok(Some(4_294_967_296))
        );
        assert_eq!(parse_content_range_count("*/12"), Ok(Some(12)));
        assert_eq!(parse_content_range_count("0-9/*"), Ok(None));
        assert_eq!(
            parse_content_range_count("not-a-range/12").unwrap_err(),
            "invalid range start"
        );
        assert_eq!(
            parse_content_range_count("10-2/12").unwrap_err(),
            "range start exceeds range end"
        );
    }

    #[tokio::test]
    async fn malformed_content_range_is_a_typed_response_error() {
        let response = local_response(
            "200 OK",
            "Content-Type: application/json\r\nContent-Range: not-a-range/12\r\n",
            "[]",
        )
        .await;
        let error = handle_response(response)
            .await
            .expect_err("malformed Content-Range must fail");
        assert!(matches!(
            error,
            Error::UnexpectedResponse {
                status: Some(200),
                ..
            }
        ));
    }

    #[tokio::test]
    async fn filtered_and_unfiltered_counts_remain_response_metadata() {
        let (all_rows, all_request) = execute_local_count_query(false).await;
        let (filtered_rows, filtered_request) = execute_local_count_query(true).await;

        assert_eq!(all_rows.data, vec![json!({"id": 1})]);
        assert_eq!(all_rows.count, Some(7));
        assert!(all_request.starts_with("GET /users HTTP/1.1"));
        assert!(all_request
            .to_ascii_lowercase()
            .contains("prefer: count=exact"));

        assert_eq!(filtered_rows.data, vec![json!({"id": 1})]);
        assert_eq!(filtered_rows.count, Some(7));
        assert!(filtered_request.starts_with("GET /users?name=eq.alice HTTP/1.1"));
        assert!(filtered_request
            .to_ascii_lowercase()
            .contains("prefer: count=exact"));
    }

    #[tokio::test]
    async fn provider_error_keeps_status_code_details_and_hint() {
        let body = r#"{"code":"42501","message":"permission denied","details":"policy blocked the row","hint":"check RLS"}"#;
        let response =
            local_response("403 Forbidden", "Content-Type: application/json\r\n", body).await;

        let error = handle_response(response)
            .await
            .expect_err("403 should fail");
        let Error::Api(error) = error else {
            panic!("expected typed API error");
        };

        assert_eq!(error.status, 403);
        assert_eq!(error.code.as_deref(), Some("42501"));
        assert_eq!(error.message, "permission denied");
        assert_eq!(error.details.as_deref(), Some("policy blocked the row"));
        assert_eq!(error.hint.as_deref(), Some("check RLS"));
    }

    #[tokio::test]
    async fn malformed_provider_error_keeps_status_and_bounded_body() {
        let response = local_response(
            "401 Unauthorized",
            "Content-Type: application/json\r\n",
            "not-json",
        )
        .await;
        let error = handle_response(response)
            .await
            .expect_err("401 should fail");
        let Error::Api(error) = error else {
            panic!("expected typed API error");
        };

        assert_eq!(error.status, 401);
        assert_eq!(error.raw_response.as_deref(), Some("not-json"));
    }

    #[tokio::test]
    async fn empty_successful_response_is_not_a_json_error() {
        let response = local_response("204 No Content", "", "").await;
        let decoded = handle_response(response)
            .await
            .expect("empty success is valid");
        assert!(decoded.data.is_empty());
        assert_eq!(decoded.count, None);
    }

    #[tokio::test]
    async fn non_json_success_is_a_structured_unexpected_response() {
        let response = local_response("200 OK", "Content-Type: text/plain\r\n", "not json").await;
        let error = handle_response(response)
            .await
            .expect_err("wrong content type should fail");
        assert!(matches!(
            error,
            Error::UnexpectedResponse {
                status: Some(200),
                ..
            }
        ));
    }

    async fn local_response(status: &str, headers: &str, body: &str) -> Response {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("local address");
        let response = format!(
            "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept local request");
            let mut reader = BufReader::new(stream);
            let mut request = String::new();
            while !request.ends_with("\r\n\r\n") {
                if reader.read_line(&mut request).expect("read request line") == 0 {
                    break;
                }
            }
            reader
                .get_mut()
                .write_all(response.as_bytes())
                .expect("write local response");
        });
        let response = reqwest::get(format!("http://{address}"))
            .await
            .expect("send local request");
        server.join().expect("join local server");
        response
    }

    async fn execute_local_count_query(filtered: bool) -> (ResponseData<Vec<Value>>, String) {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("local address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept local request");
            let mut request = [0; 2048];
            let read = stream.read(&mut request).expect("read local request");
            let response_body = r#"[{"id":1}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Range: 0-0/7\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write local response");
            String::from_utf8_lossy(&request[..read]).into_owned()
        });

        let client = crate::SupabaseClient::builder(format!("http://{address}"), "key")
            .rest_prefix("")
            .build()
            .expect("valid test client");
        let mut query = client.from("users");
        if filtered {
            query = query.eq("name", "alice");
        }
        let result = query
            .count()
            .execute_with_count()
            .await
            .expect("count query");
        let request = server.join().expect("join local server");
        (result, request)
    }

    #[tokio::test]
    async fn successful_select_response_never_adds_count_metadata_as_a_row() {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("local address");
        let body = r#"[{"id":1}]"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Range: 0-0/42\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept local request");
            let mut reader = BufReader::new(stream);
            let mut request = String::new();
            while !request.ends_with("\r\n\r\n") {
                if reader.read_line(&mut request).expect("read request line") == 0 {
                    break;
                }
            }
            reader
                .get_mut()
                .write_all(response.as_bytes())
                .expect("write local response");
        });

        let response = reqwest::get(format!("http://{address}"))
            .await
            .expect("send local request");
        let response = handle_response(response).await.expect("decode rows");
        server.join().expect("join local server");

        assert_eq!(response.data, vec![json!({"id": 1})]);
        assert_eq!(response.count, Some(42));
    }

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
