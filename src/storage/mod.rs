//! Optional Supabase Storage downloads using the configured client and credentials.

mod download;

use crate::{Error, Result, SupabaseClient};

/// A file in a Supabase Storage bucket.
#[derive(Clone)]
pub struct SupabaseStorage {
    client: SupabaseClient,
    bucket: String,
    path: String,
    public: bool,
}

impl SupabaseStorage {
    /// Creates a private object reference. Call [`public`](Self::public) for a public bucket.
    pub fn new(
        client: &SupabaseClient,
        bucket: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            client: client.clone(),
            bucket: bucket.into(),
            path: path.into(),
            public: false,
        }
    }

    /// Uses the unauthenticated public-object endpoint.
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }

    fn endpoint(&self) -> Result<String> {
        if self.bucket.is_empty() || self.bucket.contains('/') {
            return Err(Error::InvalidInput(
                "bucket must be one non-empty path segment".into(),
            ));
        }
        let path = self.path.split('/').collect::<Vec<_>>();
        if path.is_empty()
            || path
                .iter()
                .any(|part| part.is_empty() || *part == "." || *part == "..")
        {
            return Err(Error::InvalidInput(
                "object path must contain valid path segments".into(),
            ));
        }
        let mut segments = vec!["storage", "v1", "object"];
        segments.push(if self.public {
            "public"
        } else {
            "authenticated"
        });
        segments.push(&self.bucket);
        segments.extend(path);
        Ok(self.client.service_endpoint_with_segments(&segments))
    }
}
