use serde_json::json;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};

use crate::SupabaseClient;

fn mock_response(status: &str, content_type: &str, body: &str) -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local HTTP server");
    let address = listener.local_addr().expect("local address");
    let response = format!(
        "HTTP/1.1 {status}\r\n{content_type}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept request");
        let mut reader = BufReader::new(stream);
        let mut request = String::new();
        loop {
            let read = reader
                .read_line(&mut request)
                .expect("read request headers");
            if read == 0 || request.ends_with("\r\n\r\n") {
                break;
            }
        }
        let content_length = request
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("valid Content-Length"))
            })
            .unwrap_or(0);
        let mut request_body = vec![0; content_length];
        reader
            .read_exact(&mut request_body)
            .expect("read request body");
        request.push_str(&String::from_utf8_lossy(&request_body));
        reader
            .get_mut()
            .write_all(response.as_bytes())
            .expect("write mock response");
        request
    });
    (format!("http://{address}"), server)
}

fn client(url: String) -> SupabaseClient {
    SupabaseClient::builder(url, "contract-test-key")
        .schema("tenant")
        .build()
        .expect("valid test client")
}

#[tokio::test]
async fn select_http_contract() {
    let (url, server) = mock_response(
        "200 OK",
        "Content-Type: application/json\r\n",
        "[{\"id\":1}]",
    );
    let rows = client(url)
        .select("users")
        .eq("name", "Ada")
        .execute()
        .await
        .expect("select response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(rows, vec![json!({"id": 1})]);
    assert!(request.starts_with("get /rest/v1/users?name=eq.ada http/1.1\r\n"));
    assert!(request.contains("apikey: contract-test-key\r\n"));
    assert!(request.contains("authorization: bearer contract-test-key\r\n"));
    assert!(request.contains("accept-profile: tenant\r\n"));
}

#[tokio::test]
async fn select_encodes_filter_keys_and_values() {
    let (url, server) = mock_response("200 OK", "Content-Type: application/json\r\n", "[]");
    client(url)
        .select("users")
        .eq("name&select", "Ada&role=eq.admin")
        .execute()
        .await
        .expect("select response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert!(request
        .starts_with("get /rest/v1/users?name%26select=eq.ada%26role%3deq.admin http/1.1\r\n"));
}

#[tokio::test]
async fn insert_http_contract() {
    let (url, server) = mock_response(
        "201 Created",
        "Content-Type: application/json\r\n",
        "[{\"id\":\"u1\"}]",
    );
    let id = client(url)
        .insert("users", json!({"name": "Ada"}))
        .await
        .expect("insert response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(id, "u1");
    assert!(request.starts_with("post /rest/v1/users http/1.1\r\n"));
    assert!(request.contains("content-profile: tenant\r\n"));
    assert!(request.contains("prefer: return=representation\r\n"));
    assert!(request.ends_with("{\"name\":\"ada\"}"));
}

#[tokio::test]
async fn update_http_contract() {
    let (url, server) = mock_response(
        "200 OK",
        "Content-Type: application/json\r\nContent-Range: 0-0/1\r\n",
        "[{\"id\":\"u1\"}]",
    );
    let result = client(url)
        .update("users", "u1", json!({"name": "Ada Lovelace"}))
        .await
        .expect("update response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(result.affected, 1);
    assert!(request.starts_with("patch /rest/v1/users?id=eq.u1 http/1.1\r\n"));
    assert!(request.contains("content-profile: tenant\r\n"));
    assert!(request.contains("prefer: count=exact,return=representation\r\n"));
    assert!(request.ends_with("{\"name\":\"ada lovelace\"}"));
}

#[tokio::test]
async fn update_encodes_filter_values_and_requests_affected_rows() {
    let (url, server) = mock_response(
        "200 OK",
        "Content-Type: application/json\r\nContent-Range: */0\r\n",
        "[]",
    );
    let result = client(url)
        .update_with_column_name(
            "users",
            "id",
            "123&role=eq.admin",
            json!({"name": "Updated"}),
        )
        .await
        .expect("update response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(result.affected, 0);
    assert!(request.starts_with("patch /rest/v1/users?id=eq.123%26role%3deq.admin http/1.1\r\n"));
    assert!(request.contains("prefer: count=exact,return=representation\r\n"));
}

#[tokio::test]
async fn delete_http_contract() {
    let (url, server) = mock_response(
        "200 OK",
        "Content-Type: application/json\r\nContent-Range: 0-0/1\r\n",
        "[{\"id\":\"u1\"}]",
    );
    let result = client(url)
        .delete("users", "u1")
        .await
        .expect("delete response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(result.affected, 1);
    assert!(request.starts_with("delete /rest/v1/users?id=eq.u1 http/1.1\r\n"));
    assert!(request.contains("content-profile: tenant\r\n"));
    assert!(request.contains("prefer: count=exact,return=representation\r\n"));
    assert!(request.ends_with("{}"));
}

#[tokio::test]
async fn delete_encodes_filter_values_and_requests_affected_rows() {
    let (url, server) = mock_response(
        "200 OK",
        "Content-Type: application/json\r\nContent-Range: */0\r\n",
        "[]",
    );
    let result = client(url)
        .delete_without_defined_key("users", "id", "123&role=eq.admin")
        .await
        .expect("delete response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(result.affected, 0);
    assert!(request.starts_with("delete /rest/v1/users?id=eq.123%26role%3deq.admin http/1.1\r\n"));
    assert!(request.contains("prefer: count=exact,return=representation\r\n"));
}

#[cfg(feature = "rpc")]
#[tokio::test]
async fn rpc_http_contract() {
    let (url, server) = mock_response("200 OK", "Content-Type: application/json\r\n", "7");
    let result = client(url)
        .rpc("sum_values", json!({"left": 3, "right": 4}))
        .execute_single()
        .await
        .expect("RPC response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert_eq!(result, json!(7));
    assert!(request.starts_with("post /rest/v1/rpc/sum_values http/1.1\r\n"));
    assert!(request.contains("content-profile: tenant\r\n"));
    assert!(request.contains("accept-profile: tenant\r\n"));
    assert!(request.ends_with("{\"left\":3,\"right\":4}"));
}

#[cfg(feature = "rpc")]
#[tokio::test]
async fn rpc_encodes_filter_keys_and_values() {
    let (url, server) = mock_response("200 OK", "Content-Type: application/json\r\n", "[]");
    client(url)
        .rpc("list_users", json!({}))
        .eq("name&select", "Ada&role=eq.admin")
        .execute()
        .await
        .expect("RPC response");
    let request = server.join().expect("HTTP server").to_ascii_lowercase();

    assert!(request.starts_with(
        "post /rest/v1/rpc/list_users?name%26select=eq.ada%26role%3deq.admin http/1.1\r\n"
    ));
}
