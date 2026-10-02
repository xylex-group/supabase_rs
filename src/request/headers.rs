use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::errors::{Error, Result};

pub(crate) fn default_headers(api_key: &str, auth_token: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    insert_header(
        &mut headers,
        HeadersTypes::ClientInfo.into(),
        &crate::client_info(),
    )?;
    insert_header(
        &mut headers,
        HeadersTypes::ContentType.into(),
        "application/json",
    )?;
    insert_header(&mut headers, HeadersTypes::ApiKey.into(), api_key)?;
    insert_header(
        &mut headers,
        HeadersTypes::Authorization.into(),
        &format!("Bearer {auth_token}"),
    )?;
    Ok(headers)
}

pub(crate) fn insert_header(headers: &mut HeaderMap, name: HeaderName, value: &str) -> Result<()> {
    let value = HeaderValue::from_str(value)
        .map_err(|error| Error::InvalidInput(format!("invalid HTTP header value: {error}")))?;
    headers.insert(name, value);
    Ok(())
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "apikey",
            Self::Authorization => "authorization",
            Self::ContentType => "content-type",
            Self::Prefer => "prefer",
            Self::ClientInfo => "x_client_info",
            Self::Range => "range",
            Self::AcceptProfile => "accept-profile",
            Self::ContentProfile => "content-profile",
        }
    }
}

impl From<HeadersTypes> for HeaderName {
    fn from(value: HeadersTypes) -> Self {
        HeaderName::from_static(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::default_headers;
    use crate::errors::Error;

    #[test]
    fn rejects_line_breaks_in_authentication_headers() {
        let error = default_headers("key\r\nX-Injected: yes", "key")
            .expect_err("header injection must fail");
        assert!(matches!(error, Error::InvalidInput(_)));
    }
}
