//! Native network shell; authority remains with the caller's effect dispatcher.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use zeroize::Zeroize as _;

use super::HttpClient;
use super::HttpError;
use super::HttpLimit;
use super::HttpRequestPlan;
use super::HttpResponse;

const WORKER_NAME: &str = "lattice-http";
const MAX_METHOD_BYTES: usize = 16;
const CANCELLATION_POLL_MS: u64 = 10;
const HEADER_FRAMING_BYTES: usize = 4;
const INITIAL_BODY_CAPACITY: usize = 8_192;
const RESERVED_REQUEST_HEADER_COUNT: usize = 3;
const HOST_HEADER_NAME_BYTES: usize = 4;
const ACCEPT_HEADER_BYTES: usize = 13;
const CONTENT_LENGTH_HEADER_NAME_BYTES: usize = 14;
const FORBIDDEN_HEADERS: &[&str] = &[
    "host",
    "content-length",
    "transfer-encoding",
    "connection",
    "proxy-connection",
    "proxy-authorization",
    "te",
    "trailer",
    "upgrade",
    "keep-alive",
    "expect",
];

/// Real native HTTP/HTTPS with finite admission, no redirects, retries or proxies.
#[derive(Debug, Default)]
pub struct NativeHttpClient;

// r[impl lattice.native_http.authority]
// r[impl lattice.native_http.cancellation]
impl HttpClient for NativeHttpClient {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        self.execute_with_cancellation(request, None)
    }

    fn execute_with_cancellation(
        &self,
        request: &HttpRequestPlan,
        cancellation: Option<&AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        self.execute_bounded(request, &super::HttpBounds::default(), cancellation)
    }

    fn execute_bounded(
        &self,
        request: &HttpRequestPlan,
        bounds: &super::HttpBounds,
        cancellation: Option<&AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        check_cancellation(cancellation)?;
        validate_bounds(bounds)?;
        let prepared = prepare(request, bounds)?;
        std::thread::scope(|scope| {
            let worker = std::thread::Builder::new()
                .name(WORKER_NAME.to_owned())
                .spawn_scoped(scope, move || run(request, prepared, *bounds, cancellation))
                .map_err(|_| transport("worker unavailable"))?;
            worker.join().map_err(|_| transport("worker failed"))?
        })
    }
}

struct Prepared {
    url: url::Url,
    method: reqwest::Method,
    headers: reqwest::header::HeaderMap,
}

fn validate_bounds(bounds: &super::HttpBounds) -> Result<(), HttpError> {
    if bounds.response_body_bytes == 0
        || bounds.response_body_bytes > super::MAX_RESPONSE_BODY_BYTES
        || bounds.header_bytes == 0
        || bounds.header_bytes > super::MAX_HEADER_BYTES
        || bounds.connect_timeout_secs == 0
        || bounds.connect_timeout_secs > super::MAX_CONNECT_TIMEOUT_SECS
    {
        return Err(invalid("transport bounds rejected"));
    }
    Ok(())
}

// r[impl lattice.native_http.bounds]
fn prepare(request: &HttpRequestPlan, bounds: &super::HttpBounds) -> Result<Prepared, HttpError> {
    if request.method.len() > MAX_METHOD_BYTES {
        return Err(invalid("method rejected"));
    }
    if request.url.len() > super::MAX_URL_BYTES {
        return Err(HttpError::LimitExceeded(HttpLimit::Url));
    }
    if request.timeout_secs == 0 || request.timeout_secs > super::MAX_TIMEOUT_SECS {
        return Err(HttpError::LimitExceeded(HttpLimit::Deadline));
    }
    if request.body.as_ref().is_some_and(|body| body.len() > super::MAX_REQUEST_BODY_BYTES) {
        return Err(HttpError::LimitExceeded(HttpLimit::RequestBody));
    }
    if !super::validate_http_method(&request.method) || !super::validate_http_url(&request.url) {
        return Err(invalid("method or URL rejected"));
    }
    // Reject control/whitespace repair by URL parsers rather than silently changing a plan.
    if request.url.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace()) {
        return Err(invalid("URL contains forbidden bytes"));
    }
    let url = url::Url::parse(&request.url).map_err(|_| invalid("URL rejected"))?;
    let method = reqwest::Method::from_bytes(super::normalized_method(&request.method).as_bytes())
        .map_err(|_| invalid("method rejected"))?;
    // Reserve Host, Accept and Content-Length, even when a method omits one.
    if request.headers.len() > super::MAX_HEADER_COUNT - RESERVED_REQUEST_HEADER_COUNT {
        return Err(HttpError::LimitExceeded(HttpLimit::RequestHeaderCount));
    }
    let authority_bytes = url[url::Position::BeforeHost..url::Position::AfterPort].len();
    let mut bytes = count_header_bytes(
        0,
        HOST_HEADER_NAME_BYTES,
        authority_bytes,
        bounds.header_bytes,
        HttpLimit::RequestHeaderBytes,
    )?;
    bytes = bytes
        .checked_add(ACCEPT_HEADER_BYTES)
        .ok_or(HttpError::LimitExceeded(HttpLimit::RequestHeaderBytes))?;
    bytes = count_header_bytes(
        bytes,
        CONTENT_LENGTH_HEADER_NAME_BYTES,
        request.body.as_ref().map_or(0, String::len).to_string().len(),
        bounds.header_bytes,
        HttpLimit::RequestHeaderBytes,
    )?;
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in &request.headers {
        bytes = count_header_bytes(bytes, name.len(), value.len(), bounds.header_bytes, HttpLimit::RequestHeaderBytes)?;
        let name =
            reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| invalid("header name rejected"))?;
        if FORBIDDEN_HEADERS.contains(&name.as_str()) || headers.contains_key(&name) {
            return Err(invalid("duplicate or transport-owned header rejected"));
        }
        let mut value = reqwest::header::HeaderValue::from_str(value).map_err(|_| invalid("header value rejected"))?;
        // Never allow debug representations in the HTTP stack to print supplied values.
        value.set_sensitive(true);
        headers.insert(name, value);
    }
    Ok(Prepared { url, method, headers })
}

fn count_header_bytes(
    current: usize,
    name: usize,
    value: usize,
    maximum: usize,
    limit: HttpLimit,
) -> Result<usize, HttpError> {
    let bytes = current
        .checked_add(name)
        .and_then(|size| size.checked_add(value))
        .and_then(|size| size.checked_add(HEADER_FRAMING_BYTES))
        .ok_or(HttpError::LimitExceeded(limit))?;
    if bytes > maximum {
        return Err(HttpError::LimitExceeded(limit));
    }
    Ok(bytes)
}

fn run(
    request: &HttpRequestPlan,
    prepared: Prepared,
    bounds: super::HttpBounds,
    cancellation: Option<&AtomicBool>,
) -> Result<HttpResponse, HttpError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| transport("runtime unavailable"))?;
    let result = runtime.block_on(async {
        tokio::select! {
            biased;
            () = cancelled(cancellation) => Err(HttpError::Cancelled),
            result = tokio::time::timeout(Duration::from_secs(request.timeout_secs), exchange(request, prepared, bounds)) => {
                result.unwrap_or_else(|_| Err(timeout()))
            }
        }
    });
    // DNS uses OS blocking resolution. Do not make local cancellation wait for it.
    // Dropped HTTP futures and clients cannot continue sending HTTP after this return.
    runtime.shutdown_timeout(Duration::ZERO);
    result
}

// r[impl lattice.native_http.security]
async fn exchange(
    request: &HttpRequestPlan,
    prepared: Prepared,
    bounds: super::HttpBounds,
) -> Result<HttpResponse, HttpError> {
    let deadline = Duration::from_secs(request.timeout_secs);
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .referer(false)
        .no_proxy()
        .pool_max_idle_per_host(0)
        .connect_timeout(deadline.min(Duration::from_secs(bounds.connect_timeout_secs)))
        .timeout(deadline)
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .build()
        .map_err(|_| transport("client unavailable"))?;
    let mut outbound = client.request(prepared.method, prepared.url).headers(prepared.headers);
    if let Some(body) = &request.body {
        outbound = outbound.body(body.clone());
    }
    let mut response = outbound.send().await.map_err(classify)?;
    let status = response.status().as_u16();
    let mut headers = response_headers(response.headers(), bounds.header_bytes)?;
    if response.content_length().is_some_and(|length| length > bounds.response_body_bytes as u64) {
        return Err(HttpError::LimitExceeded(HttpLimit::ResponseBody));
    }
    let mut body = zeroize::Zeroizing::new(Vec::with_capacity(INITIAL_BODY_CAPACITY.min(bounds.response_body_bytes)));
    while let Some(chunk) = response.chunk().await.map_err(classify)? {
        if chunk.len() > bounds.response_body_bytes - body.len() {
            return Err(HttpError::LimitExceeded(HttpLimit::ResponseBody));
        }
        body.extend_from_slice(&chunk);
    }
    let body = String::from_utf8(std::mem::take(&mut *body)).map_err(|error| {
        error.into_bytes().zeroize();
        transport("response body is not UTF-8")
    })?;
    Ok(HttpResponse {
        status,
        headers: std::mem::take(&mut headers.0),
        body,
    })
}

struct DecodedHeaders(HashMap<String, String>);

impl Drop for DecodedHeaders {
    fn drop(&mut self) {
        for (mut name, mut value) in self.0.drain() {
            name.zeroize();
            value.zeroize();
        }
    }
}

fn response_headers(source: &reqwest::header::HeaderMap, maximum: usize) -> Result<DecodedHeaders, HttpError> {
    if source.len() > super::MAX_HEADER_COUNT {
        return Err(HttpError::LimitExceeded(HttpLimit::ResponseHeaderCount));
    }
    let mut bytes = 0;
    let mut headers = DecodedHeaders(HashMap::with_capacity(source.len()));
    for (name, value) in source {
        bytes = count_header_bytes(
            bytes,
            name.as_str().len(),
            value.as_bytes().len(),
            maximum,
            HttpLimit::ResponseHeaderBytes,
        )?;
        let value = value.to_str().map_err(|_| transport("response header encoding rejected"))?;
        if let Some(existing) = headers.0.get_mut(name.as_str()) {
            // A newline preserves repeated fields without conflating Set-Cookie commas.
            existing.push('\n');
            existing.push_str(value);
        } else {
            headers.0.insert(name.as_str().to_owned(), value.to_owned());
        }
    }
    Ok(headers)
}

async fn cancelled(cancellation: Option<&AtomicBool>) {
    let Some(flag) = cancellation else {
        return std::future::pending().await;
    };
    loop {
        if flag.load(Ordering::Acquire) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(CANCELLATION_POLL_MS)).await;
    }
}

fn check_cancellation(cancellation: Option<&AtomicBool>) -> Result<(), HttpError> {
    if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return Err(HttpError::Cancelled);
    }
    Ok(())
}

fn classify(error: reqwest::Error) -> HttpError {
    if error.is_timeout() {
        timeout()
    } else {
        transport("request failed")
    }
}

fn timeout() -> HttpError {
    HttpError::Timeout("deadline exceeded; remote outcome may be unknown".to_owned())
}

fn transport(message: &str) -> HttpError {
    HttpError::Transport(message.to_owned())
}

fn invalid(message: &str) -> HttpError {
    HttpError::InvalidRequest(message.to_owned())
}
