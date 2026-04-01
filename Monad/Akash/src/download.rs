use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedDownload {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
}

impl PresignedDownload {
    pub fn new(url: String, method: impl Into<String>, headers: BTreeMap<String, String>) -> Self {
        Self {
            url,
            method: method.into(),
            headers,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }
}
