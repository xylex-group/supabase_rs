use reqwest::header::HeaderMap;

use crate::{request::headers::default_headers, success::check_http_response, Result};

use super::SupabaseStorage;

impl SupabaseStorage {
    /// Downloads the object's bytes.
    pub async fn download(&self) -> Result<Vec<u8>> {
        let headers = if self.public {
            HeaderMap::new()
        } else {
            default_headers(&self.client.api_key, &self.client.api_key)?
        };
        let response = self
            .client
            .client
            .get(self.endpoint()?)
            .headers(headers)
            .send()
            .await?;
        let response = check_http_response(response).await?;
        Ok(response.bytes().await?.to_vec())
    }

    /// Downloads the object and writes it to a local file.
    pub async fn save(&self, file_path: impl AsRef<std::path::Path>) -> Result<()> {
        tokio::fs::write(file_path, self.download().await?).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SupabaseStorage;
    use crate::{Error, SupabaseClient};
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    fn server(status: &'static str, body: &'static str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("local address");
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0; 2048];
            let size = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..size]).to_ascii_lowercase();
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
            request
        });
        (format!("http://{address}"), handle)
    }

    #[tokio::test]
    async fn private_download_uses_client_auth_and_encodes_object_path() {
        let (url, server) = server("200 OK", "bytes");
        let client = SupabaseClient::builder(url, "test-key")
            .rest_prefix("")
            .build()
            .expect("client");
        let result = SupabaseStorage::new(&client, "private", "folder/a b.txt")
            .download()
            .await
            .expect("download");
        let request = server.join().expect("server");
        assert_eq!(result, b"bytes");
        assert!(request
            .starts_with("get /storage/v1/object/authenticated/private/folder/a%20b.txt http/1.1"));
        assert!(request.contains("apikey: test-key"));
        assert!(request.contains("authorization: bearer test-key"));
    }

    #[tokio::test]
    async fn public_download_uses_public_endpoint_without_auth_headers() {
        let (url, server) = server("200 OK", "bytes");
        let client = SupabaseClient::builder(url, "test-key")
            .rest_prefix("")
            .build()
            .expect("client");
        SupabaseStorage::new(&client, "public", "a.txt")
            .public()
            .download()
            .await
            .expect("download");
        let request = server.join().expect("server");
        assert!(request.starts_with("get /storage/v1/object/public/public/a.txt http/1.1"));
        assert!(!request.contains("authorization:"));
        assert!(!request.contains("apikey:"));
    }

    #[tokio::test]
    async fn download_returns_typed_error_for_unsuccessful_status() {
        let (url, server) = server("403 Forbidden", r#"{"message":"denied"}"#);
        let client = SupabaseClient::builder(url, "test-key")
            .rest_prefix("")
            .build()
            .expect("client");
        let error = SupabaseStorage::new(&client, "private", "secret.txt")
            .download()
            .await
            .expect_err("403 must fail");
        assert!(matches!(error, Error::Api(api) if api.status == 403));
        server.join().expect("server");
    }
}
