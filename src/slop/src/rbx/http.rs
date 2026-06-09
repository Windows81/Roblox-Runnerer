//! Boundary for the engine HTTP stack (`util/Http.h`, `util/HttpAsync.h`).
//!
//! EXTERNAL: `RBX::Http`, `RBX::HttpAsync`, `RBX::HttpFuture`,
//! `RBX::HttpPostData`, `RBX::Http::SetUseCurl`, `RBX::Http::trustCheckBrowser`.

#![allow(dead_code, unused_variables)]

use std::io::Read;

/// `RBX::HttpFuture` — a boost shared future of the response body.
///
/// In the original this is `boost::shared_future<std::string>`; here it is a
/// small handle that yields the body (blocking) via [`HttpFuture::get`].
#[derive(Clone, Default)]
pub struct HttpFuture {
    inner: Option<std::sync::Arc<HttpFutureState>>,
}

#[derive(Default)]
struct HttpFutureState {
    // Engine-backed; stub.
}

impl HttpFuture {
    pub fn valid(&self) -> bool {
        self.inner.is_some()
    }
    /// `future.get()` — block for the body, propagating engine exceptions.
    pub fn get(&self) -> Result<String, EngineError> {
        Ok(String::new())
    }
    /// `future.wait()`.
    pub fn wait(&self) {}
}

/// `RBX::base_exception` family.
#[derive(Debug)]
pub struct EngineError(pub String);
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for EngineError {}

/// `RBX::HttpPostData`.
pub struct HttpPostData {
    pub body: String,
    pub content_type: String,
    pub gzip: bool,
}
impl HttpPostData {
    pub fn new(body: String, content_type: &str, gzip: bool) -> Self {
        Self { body, content_type: content_type.into(), gzip }
    }
}

pub const CONTENT_TYPE_APPLICATION_JSON: &str = "application/json";
pub const CONTENT_TYPE_DEFAULT_UNSPECIFIED: &str = "";

/// `RBX::HttpAsync::getWithRetries`.
pub fn get_with_retries(url: &str, retries: u32) -> HttpFuture {
    HttpFuture { inner: Some(Default::default()) }
}

/// `RBX::HttpAsync::post`.
pub fn post_async(url: &str, data: HttpPostData) -> HttpFuture {
    HttpFuture { inner: Some(Default::default()) }
}

/// `boost::make_shared_future(std::string())` — an already-ready empty future.
pub fn ready_empty_future() -> HttpFuture {
    HttpFuture { inner: Some(Default::default()) }
}

/// Synchronous `RBX::Http`.
pub struct Http {
    url: String,
    pub additional_headers: std::collections::HashMap<String, String>,
}
impl Http {
    pub fn new(url: &str) -> Self {
        Self { url: url.into(), additional_headers: Default::default() }
    }
    pub fn get(&self, out: &mut String) -> Result<(), EngineError> {
        Ok(())
    }
    pub fn post<R: Read>(
        &self,
        body: R,
        content_type: &str,
        gzip: bool,
        out: &mut String,
    ) -> Result<(), EngineError> {
        Ok(())
    }
    /// `RBX::Http::SetUseCurl`.
    pub fn set_use_curl(enable: bool) {}
    /// `RBX::Http::SetUseStatistics`.
    pub fn set_use_statistics(enable: bool) {}
    /// `RBX::Http::trustCheckBrowser`.
    pub fn trust_check_browser(url: &str) -> bool {
        true
    }
}
