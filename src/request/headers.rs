use crate::request::Headers;
use reqwest::header::HeaderName;
use std::collections::HashMap;

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            headers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_owned(), value.to_owned());
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn with_defaults(api_key: &str, auth_token: &str) -> Self {
        let mut headers = Headers::new();
        headers.insert(HeadersTypes::ClientInfo.as_str(), &crate::client_info());
        headers.insert(HeadersTypes::ContentType.as_str(), "application/json");
        headers.insert(HeadersTypes::ApiKey.as_str(), api_key);
        headers.insert(
            HeadersTypes::Authorization.as_str(),
            &format!("Bearer {auth_token}"),
        );
        headers
    }
}

pub enum HeadersTypes {
    ApiKey,
    Authorization,
    ContentType,
    Prefer,
    ClientInfo,
    Range,
    AcceptProfile,
    ContentProfile,
}

impl HeadersTypes {
    pub fn as_str(&self) -> &str {
        match self {
            HeadersTypes::ApiKey => "apikey",
            HeadersTypes::Authorization => "Authorization",
            HeadersTypes::ContentType => "Content-Type",
            HeadersTypes::Prefer => "prefer",
            HeadersTypes::ClientInfo => "x_client_info",
            HeadersTypes::Range => "Range",
            HeadersTypes::AcceptProfile => "Accept-Profile",
            HeadersTypes::ContentProfile => "Content-Profile",
        }
    }
}

impl From<HeadersTypes> for HeaderName {
    fn from(value: HeadersTypes) -> Self {
        HeaderName::from_bytes(value.as_str().as_bytes()).expect("Invalid header name")
    }
}
