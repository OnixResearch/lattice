//! Bounded GitHub Actions identity acquisition and Check Runs transport.
//!
//! Callers must durably reserve `CheckRunPlan::reserved_binding` before execute.
//! An uncertain mutation is recovered by observation, never blindly repeated.

use std::collections::BTreeSet;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroize;
use zeroize::Zeroizing;

use super::core::CheckProjection;
use super::core::CheckStatus;
use super::core::Conclusion;
use super::core::Identity;
use super::core::{self};
use crate::credentials::CredentialDestination;
use crate::credentials::CredentialProvider;
use crate::credentials::CredentialResolutionContext;
use crate::http::HttpBounds;
use crate::http::HttpClient;
use crate::http::HttpError;
use crate::http::HttpRequestPlan;
use crate::http::HttpResponse;

pub const REQUEST_TIMEOUT_SECS: u64 = 20;
/// Includes DNS, TCP connection and TLS negotiation in one deadline.
pub const DNS_CONNECT_TLS_TIMEOUT_SECS: u64 = 5;
pub const MAX_HEADER_BYTES: usize = 8_192;
pub const MAX_HEADER_COUNT: usize = 32;
pub const MAX_OIDC_BODY_BYTES: usize = 16_384;
pub const MAX_TOKEN_BYTES: usize = 12_288;
pub const MAX_API_BODY_BYTES: usize = 1_048_576;
pub const MAX_LIST_PAGES: u32 = 10;
pub const CHECKS_PER_PAGE: usize = 100;
pub const MAX_ANNOTATIONS: usize = 50;
pub const MAX_TITLE_BYTES: usize = 255;
pub const MAX_SUMMARY_BYTES: usize = 32_768;
pub const MAX_TEXT_BYTES: usize = 32_768;
pub const MAX_ANNOTATION_BYTES: usize = 4_096;
pub const MAX_URL_BYTES: usize = 8_192;
pub const SECONDARY_LIMIT_MIN_DELAY_MS: u64 = 60_000;
pub const API_VERSION: &str = "2022-11-28";
pub const API_HOST: &str = "api.github.com";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    MissingIdentityPermission,
    UnsafeUrl,
    Bounds,
    Malformed,
    CredentialUnavailable,
    RevokedCredential,
    PermissionDenied,
    MissingRepository,
    RateLimited,
    AbuseLimited,
    Transient,
    Timeout,
    Redirect,
    Ambiguous,
    ReconciliationRequired,
    StateConflict,
    Rejected,
}

/// Public categories only. No response bodies, provider errors, URLs or tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("GitHub adapter: {kind:?}")]
pub struct Error {
    pub kind: ErrorKind,
    pub retry_after_ms: Option<u64>,
    pub rate_reset_ms: Option<u64>,
    /// True only for a definitive rejection, never a disconnect or server 5xx.
    pub remote_rejected: bool,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            retry_after_ms: None,
            rate_reset_ms: None,
            remote_rejected: false,
        }
    }

    pub fn retry_decision(
        &self,
        retries: u32,
        started_ms: u64,
        now_ms: u64,
        policy: &core::Policy,
    ) -> Result<core::RetryDecision, core::Error> {
        let observed = match self.kind {
            ErrorKind::RateLimited => core::ObservedError::RateLimited,
            ErrorKind::AbuseLimited => core::ObservedError::AbuseLimited,
            ErrorKind::Transient => core::ObservedError::Transient,
            ErrorKind::Timeout => core::ObservedError::Timeout,
            ErrorKind::RevokedCredential => core::ObservedError::Unauthorized,
            ErrorKind::PermissionDenied => core::ObservedError::PermissionDenied,
            ErrorKind::MissingRepository => core::ObservedError::MissingRepository,
            ErrorKind::ReconciliationRequired => core::ObservedError::DuplicateCreate,
            _ => core::ObservedError::Rejected,
        };
        let reset_delay = self.rate_reset_ms.map(|reset| reset.saturating_sub(now_ms));
        let delay = match (self.retry_after_ms, reset_delay) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        core::retry_decision(observed, retries, started_ms, now_ms, delay, policy)
    }
}

/// Documented job request credential; intentionally no Debug/Clone/Serialize.
pub struct ActionsRequestToken(Zeroizing<String>);

impl ActionsRequestToken {
    pub fn from_documented_value(value: String) -> Result<Self, Error> {
        let token = Self(Zeroizing::new(value));
        if !token_valid(&token.0) {
            return Err(Error::new(ErrorKind::MissingIdentityPermission));
        }
        Ok(token)
    }
}

/// Intentionally neither Debug, Clone nor Serialize. Borrow only for immediate
/// Basalt exchange. It must not enter workflow data or durable observations.
pub struct OidcToken(Zeroizing<String>);

impl OidcToken {
    pub fn expose_for_exchange(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for OidcToken {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| Self(Zeroizing::new(value)))
    }
}

// The generic HTTP contract is serializable/debuggable, but secret-bearing
// instances never escape these non-Debug guards. All caller-owned copies clear.
struct TransientRequest(HttpRequestPlan);
impl Drop for TransientRequest {
    fn drop(&mut self) {
        for value in self.0.headers.values_mut() {
            value.zeroize();
        }
        if let Some(body) = &mut self.0.body {
            body.zeroize();
        }
        self.0.url.zeroize();
    }
}
struct TransientResponse(HttpResponse);
impl Drop for TransientResponse {
    fn drop(&mut self) {
        self.0.body.zeroize();
        for value in self.0.headers.values_mut() {
            value.zeroize();
        }
    }
}

fn transport_error(error: HttpError) -> Error {
    let kind = match error {
        HttpError::Timeout(_) => ErrorKind::Timeout,
        HttpError::LimitExceeded(_) => ErrorKind::Bounds,
        HttpError::InvalidRequest(_) => ErrorKind::Rejected,
        HttpError::Transport(_) | HttpError::Cancelled => ErrorKind::Transient,
    };
    Error::new(kind)
}

fn header<'a>(response: &'a HttpResponse, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn classify(response: &HttpResponse) -> Error {
    let remaining_zero = header(response, "x-ratelimit-remaining") == Some("0");
    let retry = header(response, "retry-after");
    let secondary = [b"secondary rate limit".as_slice(), b"abuse".as_slice()]
        .iter()
        .any(|needle| response.body.as_bytes().windows(needle.len()).any(|part| part.eq_ignore_ascii_case(needle)));
    let kind = match response.status {
        300..=399 => ErrorKind::Redirect,
        401 => ErrorKind::RevokedCredential,
        403 if remaining_zero => ErrorKind::RateLimited,
        403 if retry.is_some() || secondary => ErrorKind::AbuseLimited,
        403 => ErrorKind::PermissionDenied,
        404 => ErrorKind::MissingRepository,
        429 => ErrorKind::RateLimited,
        409 | 422 => ErrorKind::ReconciliationRequired,
        408 | 500..=599 => ErrorKind::Transient,
        _ => ErrorKind::Rejected,
    };
    // Retry-After seconds only. HTTP dates/overflow/malformed values fail closed
    // rather than accidentally authorizing an immediate retry.
    let retry_after_ms = retry
        .map(|value| value.parse::<u64>().ok().and_then(|s| s.checked_mul(1000)).unwrap_or(u64::MAX))
        .or_else(|| (kind == ErrorKind::AbuseLimited).then_some(SECONDARY_LIMIT_MIN_DELAY_MS));
    let rate_reset_ms = if remaining_zero {
        Some(
            header(response, "x-ratelimit-reset")
                .and_then(|value| value.parse::<u64>().ok())
                .and_then(|s| s.checked_mul(1000))
                .unwrap_or(u64::MAX),
        )
    } else {
        None
    };
    Error {
        kind,
        retry_after_ms,
        rate_reset_ms,
        remote_rejected: matches!(response.status, 400..=407 | 410..=421 | 423..=499),
    }
}

fn token_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TOKEN_BYTES && value.bytes().all(|c| c.is_ascii_graphic())
}

fn send(http: &dyn HttpClient, request: &TransientRequest, max_body: usize) -> Result<TransientResponse, Error> {
    if request.0.headers.len() > MAX_HEADER_COUNT
        || request.0.headers.iter().map(|(name, value)| name.len() + value.len() + 4).sum::<usize>() > MAX_HEADER_BYTES
    {
        return Err(Error::new(ErrorKind::Bounds));
    }
    let bounds = HttpBounds {
        response_body_bytes: max_body,
        header_bytes: MAX_HEADER_BYTES,
        connect_timeout_secs: DNS_CONNECT_TLS_TIMEOUT_SECS,
    };
    let response = TransientResponse(http.execute_bounded(&request.0, &bounds, None).map_err(transport_error)?);
    // Independently enforce the port contract for embedders as well as native I/O.
    if response.0.body.len() > max_body
        || response.0.headers.len() > MAX_HEADER_COUNT
        || response
            .0
            .headers
            .iter()
            .try_fold(0usize, |size, (name, value)| {
                size.checked_add(name.len())?.checked_add(value.len())?.checked_add(4)
            })
            .is_none_or(|size| size > MAX_HEADER_BYTES)
    {
        return Err(Error::new(ErrorKind::Bounds));
    }
    if !(200..300).contains(&response.0.status) {
        return Err(classify(&response.0));
    }
    Ok(response)
}

pub struct ActionsOidcClient<'a> {
    http: &'a dyn HttpClient,
    allowed_hosts: BTreeSet<String>,
}

impl<'a> ActionsOidcClient<'a> {
    /// Exact host names only, supplied by the selected deployment; no wildcards.
    pub fn new(http: &'a dyn HttpClient, allowed_hosts: BTreeSet<String>) -> Result<Self, Error> {
        if allowed_hosts.is_empty()
            || allowed_hosts.iter().any(|host| {
                host.is_empty()
                    || host.len() > 253
                    || host.parse::<std::net::IpAddr>().is_ok()
                    || !host.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.' || c == b'-')
            })
        {
            return Err(Error::new(ErrorKind::UnsafeUrl));
        }
        Ok(Self { http, allowed_hosts })
    }

    /// Reads ONLY GitHub's documented job identity endpoint and request token.
    /// No GITHUB_TOKEN, static Mantle credential or ambient fallback is consulted.
    pub fn acquire_from_environment(&self, basalt_audience: &str) -> Result<OidcToken, Error> {
        let url = Zeroizing::new(
            std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")
                .map_err(|_| Error::new(ErrorKind::MissingIdentityPermission))?,
        );
        let token = ActionsRequestToken::from_documented_value(
            std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
                .map_err(|_| Error::new(ErrorKind::MissingIdentityPermission))?,
        )?;
        self.acquire(&url, &token, basalt_audience)
    }

    /// Explicit documented values for hosts whose environment is already scoped.
    pub fn acquire(
        &self,
        request_url: &str,
        request_token: &ActionsRequestToken,
        basalt_audience: &str,
    ) -> Result<OidcToken, Error> {
        if basalt_audience.is_empty() || basalt_audience.len() > 2_048 || basalt_audience.chars().any(char::is_control)
        {
            return Err(Error::new(ErrorKind::Bounds));
        }
        let mut url = safe_https(request_url)?;
        if !url.host_str().is_some_and(|host| self.allowed_hosts.contains(host))
            || url.query_pairs().any(|(key, value)| key != "api-version" || value != "2.0")
            || url.query_pairs().count() > 1
        {
            return Err(Error::new(ErrorKind::UnsafeUrl));
        }
        url.query_pairs_mut().append_pair("audience", basalt_audience);
        if url.as_str().len() > MAX_URL_BYTES {
            return Err(Error::new(ErrorKind::Bounds));
        }
        let mut request = TransientRequest(HttpRequestPlan {
            method: "GET".into(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout_secs: REQUEST_TIMEOUT_SECS,
        });
        request.0.headers.insert("Authorization".into(), format!("Bearer {}", request_token.0.as_str()));
        request.0.headers.insert("Accept".into(), "application/json".into());
        let response = send(self.http, &request, MAX_OIDC_BODY_BYTES)?;
        #[derive(Deserialize)]
        struct IdentityResponse {
            value: OidcToken,
        }
        let parsed: IdentityResponse =
            serde_json::from_str(&response.0.body).map_err(|_| Error::new(ErrorKind::Malformed))?;
        if !token_valid(parsed.value.expose_for_exchange()) {
            return Err(Error::new(ErrorKind::Malformed));
        }
        Ok(parsed.value)
    }
}

fn safe_https(value: &str) -> Result<url::Url, Error> {
    if value.len() > MAX_URL_BYTES || value.chars().any(char::is_control) || value.contains('\\') {
        return Err(Error::new(ErrorKind::UnsafeUrl));
    }
    let url = url::Url::parse(value).map_err(|_| Error::new(ErrorKind::UnsafeUrl))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.port().is_some_and(|p| p != 443)
    {
        return Err(Error::new(ErrorKind::UnsafeUrl));
    }
    Ok(url)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRunTarget {
    pub owner: String,
    pub repository: String,
    pub head_sha: String,
    pub name: String,
    pub external_id: Identity,
}

impl CheckRunTarget {
    pub(super) fn validate(&self) -> Result<(), Error> {
        let segment = |s: &str| {
            !s.is_empty()
                && s.len() <= 100
                && s != "."
                && s != ".."
                && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        };
        if !segment(&self.owner)
            || !segment(&self.repository)
            || self.head_sha.len() != 40
            || !self.head_sha.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            || self.name.is_empty()
            || self.name.len() > MAX_TITLE_BYTES
            || self.name.chars().any(char::is_control)
        {
            return Err(Error::new(ErrorKind::Bounds));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedProjection {
    pub version: u64,
    pub update_id: Identity,
    pub terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRunIntent {
    pub version: u64,
    pub update_id: Identity,
    pub terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRunBinding {
    pub target: CheckRunTarget,
    pub check_id: Option<u64>,
    pub applied: Option<AppliedProjection>,
    pub pending: Option<CheckRunIntent>,
}

impl CheckRunBinding {
    pub fn new(target: CheckRunTarget) -> Result<Self, Error> {
        target.validate()?;
        Ok(Self {
            target,
            check_id: None,
            applied: None,
            pending: None,
        })
    }

    /// Only an observed rejection permits clearing a reservation for retry. A
    /// timeout, malformed success or lost response must remain recovery-only.
    pub fn after_rejection(&self, error: &Error) -> Result<Self, Error> {
        if !error.remote_rejected {
            return Err(Error::new(ErrorKind::ReconciliationRequired));
        }
        let mut next = self.clone();
        next.pending = None;
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Noop,
    Create,
    Update,
    Recover,
}

/// Deliberately not serializable: persist the public reservation, then execute
/// this one-use plan. Replanning a persisted pending intent always recovers.
pub struct CheckRunPlan {
    reserved_binding: CheckRunBinding,
    action: Action,
}

impl CheckRunPlan {
    pub fn reserved_binding(&self) -> &CheckRunBinding {
        &self.reserved_binding
    }
}

pub fn prepare_check(binding: &CheckRunBinding, projection: &CheckProjection) -> Result<CheckRunPlan, Error> {
    binding.target.validate()?;
    validate_projection(projection)?;
    if projection.external_id != binding.target.external_id
        || binding.check_id == Some(0)
        || projection.check_id.is_some_and(|id| Some(id) != binding.check_id)
        || (binding.applied.is_some() && binding.check_id.is_none())
        || binding.pending.as_ref().is_some_and(|pending| {
            binding
                .applied
                .as_ref()
                .is_some_and(|applied| pending.version <= applied.version || applied.terminal)
        })
    {
        return Err(Error::new(ErrorKind::StateConflict));
    }
    let mut next = binding.clone();
    if let Some(applied) = &binding.applied {
        if projection.state_version < applied.version
            || (projection.state_version == applied.version && projection.update_id == applied.update_id)
        {
            return Ok(CheckRunPlan {
                reserved_binding: next,
                action: Action::Noop,
            });
        }
        if applied.terminal || projection.state_version == applied.version {
            return Err(Error::new(ErrorKind::StateConflict));
        }
    }
    let intent = CheckRunIntent {
        version: projection.state_version,
        update_id: projection.update_id,
        terminal: projection.status == CheckStatus::Completed,
    };
    let action = if let Some(pending) = &binding.pending {
        if pending != &intent {
            return Err(Error::new(ErrorKind::ReconciliationRequired));
        }
        Action::Recover
    } else if binding.check_id.is_some() {
        Action::Update
    } else {
        Action::Create
    };
    next.pending = Some(intent);
    Ok(CheckRunPlan {
        reserved_binding: next,
        action,
    })
}

fn validate_projection(p: &CheckProjection) -> Result<(), Error> {
    if p.title.is_empty()
        || p.title.len() > MAX_TITLE_BYTES
        || p.summary.len() > MAX_SUMMARY_BYTES
        || p.text.len() > MAX_TEXT_BYTES
        || p.annotations.len() > MAX_ANNOTATIONS
        || p.links.len() > 16
        || (p.status == CheckStatus::Completed) != p.conclusion.is_some()
    {
        return Err(Error::new(ErrorKind::Bounds));
    }
    let rendered_text_bytes = p.links.iter().try_fold(p.text.len(), |size, link| {
        size.checked_add("\n\nEvidence: ".len())?.checked_add(link.as_str().len())
    });
    if rendered_text_bytes.is_none_or(|size| size > MAX_TEXT_BYTES) {
        return Err(Error::new(ErrorKind::Bounds));
    }
    for annotation in &p.annotations {
        if annotation.line == 0
            || annotation.path.is_empty()
            || annotation.path.len() > 1_024
            || annotation.path.starts_with('/')
            || annotation.path.contains('\\')
            || annotation.path.split('/').any(|segment| segment.is_empty() || segment == ".." || segment == ".")
            || annotation.path.chars().any(char::is_control)
            || annotation.message.len() > MAX_ANNOTATION_BYTES
        {
            return Err(Error::new(ErrorKind::Bounds));
        }
    }
    for link in &p.links {
        let url = safe_https(link.as_str())?;
        if url.query().is_some() {
            return Err(Error::new(ErrorKind::UnsafeUrl));
        }
    }
    Ok(())
}

pub struct GitHubChecksClient<'a> {
    http: &'a dyn HttpClient,
    credentials: &'a dyn CredentialProvider,
}

impl<'a> GitHubChecksClient<'a> {
    /// github.com only; enterprise endpoints require a separately reviewed policy.
    /// Host allowance is explicit rather than inferred from repository text.
    pub fn new(
        http: &'a dyn HttpClient,
        credentials: &'a dyn CredentialProvider,
        allowed_hosts: &BTreeSet<String>,
    ) -> Result<Self, Error> {
        if !allowed_hosts.contains(API_HOST) {
            return Err(Error::new(ErrorKind::UnsafeUrl));
        }
        Ok(Self { http, credentials })
    }

    /// Consumes the plan so a local caller cannot accidentally execute its POST
    /// twice. The durable reservation must be committed before this call.
    pub fn execute(
        &self,
        plan: CheckRunPlan,
        projection: &CheckProjection,
        context: &CredentialResolutionContext,
    ) -> Result<CheckRunBinding, Error> {
        let mut binding = plan.reserved_binding;
        if plan.action == Action::Noop {
            return Ok(binding);
        }
        validate_projection(projection)?;
        if binding.pending.as_ref()
            != Some(&CheckRunIntent {
                version: projection.state_version,
                update_id: projection.update_id,
                terminal: projection.status == CheckStatus::Completed,
            })
            || projection.external_id != binding.target.external_id
        {
            return Err(Error::new(ErrorKind::StateConflict));
        }
        if context.destination != CredentialDestination::HttpHeader || context.binding != "Authorization" {
            return Err(Error::new(ErrorKind::CredentialUnavailable));
        }
        let credential = self
            .credentials
            .resolve_in_context(&context.reference, context, None)
            .map_err(|_| Error::new(ErrorKind::CredentialUnavailable))?;
        let token = credential.expose_for_execution();
        if !token_valid(token) {
            return Err(Error::new(ErrorKind::CredentialUnavailable));
        }
        let remote = self.find_exact(&binding.target, token).map_err(|mut error| {
            // A rejection of this read says nothing about the earlier uncertain
            // mutation. It must never authorize clearing that reservation.
            if plan.action == Action::Recover {
                error.remote_rejected = false;
            }
            error
        })?;
        if let Some(remote) = remote {
            if binding.check_id.is_some_and(|id| id != remote.id) {
                return Err(Error::new(ErrorKind::Ambiguous));
            }
            binding.check_id = Some(remote.id);
            if let Some(observed) = remote.observed_projection() {
                if observed.version > projection.state_version {
                    binding.applied = Some(observed);
                    binding.pending = None;
                    return Ok(binding);
                }
                if observed.version == projection.state_version && observed.update_id != projection.update_id {
                    return Err(Error::new(ErrorKind::StateConflict));
                }
            }
            if remote.matches_projection(projection) {
                acknowledge(&mut binding, projection);
                return Ok(binding);
            }
            if remote.status == "completed" || plan.action == Action::Recover {
                return Err(Error::new(ErrorKind::ReconciliationRequired));
            }
            let path =
                format!("/repos/{}/{}/check-runs/{}", binding.target.owner, binding.target.repository, remote.id);
            let response = self.request("PATCH", &path, Some(payload(&binding.target, projection, false)?), token)?;
            let updated = parse_run(&response.0.body)?;
            verify_run(&updated, &binding.target)?;
            if updated.id != remote.id || !updated.matches_projection(projection) {
                return Err(Error::new(ErrorKind::Malformed));
            }
        } else {
            if plan.action != Action::Create || binding.check_id.is_some() {
                return Err(Error::new(ErrorKind::ReconciliationRequired));
            }
            let path = format!("/repos/{}/{}/check-runs", binding.target.owner, binding.target.repository);
            let response = self.request("POST", &path, Some(payload(&binding.target, projection, true)?), token)?;
            let created = parse_run(&response.0.body)?;
            verify_run(&created, &binding.target)?;
            if !created.matches_projection(projection) {
                return Err(Error::new(ErrorKind::Malformed));
            }
            binding.check_id = Some(created.id);
        }
        acknowledge(&mut binding, projection);
        Ok(binding)
    }

    fn request(&self, method: &str, path: &str, body: Option<String>, token: &str) -> Result<TransientResponse, Error> {
        if body.as_ref().is_some_and(|body| body.len() > MAX_API_BODY_BYTES) {
            return Err(Error::new(ErrorKind::Bounds));
        }
        let mut request = TransientRequest(HttpRequestPlan {
            method: method.into(),
            url: format!("https://{API_HOST}{path}"),
            headers: HashMap::new(),
            body,
            timeout_secs: REQUEST_TIMEOUT_SECS,
        });
        request.0.headers.insert("Authorization".into(), format!("Bearer {token}"));
        request.0.headers.insert("Accept".into(), "application/vnd.github+json".into());
        request.0.headers.insert("X-GitHub-Api-Version".into(), API_VERSION.into());
        request.0.headers.insert("User-Agent".into(), "lattice-async-checks".into());
        if request.0.body.is_some() {
            request.0.headers.insert("Content-Type".into(), "application/json".into());
        }
        send(self.http, &request, MAX_API_BODY_BYTES)
    }

    fn find_exact(&self, target: &CheckRunTarget, token: &str) -> Result<Option<RemoteRun>, Error> {
        let mut found = None;
        let mut seen = BTreeSet::new();
        let mut total = None;
        let mut count = 0usize;
        let external_id = target.external_id.hex();
        for page in 1..=MAX_LIST_PAGES {
            // Never follow an untrusted Link URL. All pagination is reconstructed
            // under this exact API origin, repo, commit and fixed page bound.
            let path = format!(
                "/repos/{}/{}/commits/{}/check-runs?filter=all&per_page={CHECKS_PER_PAGE}&page={page}",
                target.owner, target.repository, target.head_sha
            );
            let response = self.request("GET", &path, None, token)?;
            let list: RunList = serde_json::from_str(&response.0.body).map_err(|_| Error::new(ErrorKind::Malformed))?;
            if list.total_count > MAX_LIST_PAGES as usize * CHECKS_PER_PAGE
                || list.check_runs.len() > CHECKS_PER_PAGE
                || total.is_some_and(|prior| prior != list.total_count)
            {
                return Err(Error::new(ErrorKind::Bounds));
            }
            total = Some(list.total_count);
            let page_len = list.check_runs.len();
            for run in list.check_runs {
                if run.id == 0 || !seen.insert(run.id) {
                    return Err(Error::new(ErrorKind::Ambiguous));
                }
                count += 1;
                if run.external_id.as_deref() == Some(external_id.as_str()) {
                    verify_run(&run, target)?;
                    if found.is_some() {
                        return Err(Error::new(ErrorKind::Ambiguous));
                    }
                    found = Some(run);
                }
            }
            if count == list.total_count {
                return Ok(found);
            }
            if count > list.total_count || page_len < CHECKS_PER_PAGE {
                return Err(Error::new(ErrorKind::Malformed));
            }
        }
        Err(Error::new(ErrorKind::Bounds))
    }
}

fn marker(p: &CheckProjection) -> String {
    format!("\n\n<!-- lattice-check-v1:{}:{} -->", p.state_version, p.update_id.hex())
}

fn acknowledge(binding: &mut CheckRunBinding, p: &CheckProjection) {
    binding.applied = Some(AppliedProjection {
        version: p.state_version,
        update_id: p.update_id,
        terminal: p.status == CheckStatus::Completed,
    });
    binding.pending = None;
}

fn payload(target: &CheckRunTarget, p: &CheckProjection, create: bool) -> Result<String, Error> {
    let status = match p.status {
        CheckStatus::Queued => "queued",
        CheckStatus::InProgress => "in_progress",
        CheckStatus::Completed => "completed",
    };
    let conclusion = p.conclusion.map(|c| match c {
        Conclusion::Success => "success",
        Conclusion::Failure => "failure",
        Conclusion::Cancelled => "cancelled",
        Conclusion::ActionRequired => "action_required",
    });
    let text = if p.links.is_empty() {
        std::borrow::Cow::Borrowed(p.text.as_str())
    } else {
        let size =
            p.text.len() + p.links.iter().map(|link| "\n\nEvidence: ".len() + link.as_str().len()).sum::<usize>();
        if size > MAX_TEXT_BYTES {
            return Err(Error::new(ErrorKind::Bounds));
        }
        let mut text = String::with_capacity(size);
        text.push_str(&p.text);
        for link in &p.links {
            text.push_str("\n\nEvidence: ");
            text.push_str(link.as_str());
        }
        std::borrow::Cow::Owned(text)
    };
    #[derive(Serialize)]
    struct Output<'a> {
        title: &'a str,
        summary: String,
        text: std::borrow::Cow<'a, str>,
        #[serde(serialize_with = "serialize_annotations")]
        annotations: &'a [core::Annotation],
    }
    #[derive(Serialize)]
    struct Payload<'a> {
        name: &'a str,
        external_id: String,
        status: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        conclusion: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        head_sha: Option<&'a str>,
        output: Output<'a>,
    }
    let body = Payload {
        name: &target.name,
        external_id: target.external_id.hex(),
        status,
        conclusion,
        head_sha: create.then_some(target.head_sha.as_str()),
        output: Output {
            title: &p.title,
            summary: format!("{}\n\n<!-- lattice-check-v1:{}:{} -->", p.summary, p.state_version, p.update_id.hex()),
            text,
            annotations: &p.annotations,
        },
    };
    serde_json::to_string(&body).map_err(|_| Error::new(ErrorKind::Malformed))
}

fn serialize_annotations<S: serde::Serializer>(
    annotations: &[core::Annotation],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    #[derive(Serialize)]
    struct WireAnnotation<'a> {
        path: &'a str,
        start_line: u32,
        end_line: u32,
        annotation_level: &'a str,
        message: &'a str,
    }
    let mut sequence = serializer.serialize_seq(Some(annotations.len()))?;
    for annotation in annotations {
        sequence.serialize_element(&WireAnnotation {
            path: &annotation.path,
            start_line: annotation.line,
            end_line: annotation.line,
            annotation_level: "failure",
            message: &annotation.message,
        })?;
    }
    sequence.end()
}

#[derive(Deserialize)]
struct RunList {
    total_count: usize,
    check_runs: Vec<RemoteRun>,
}
#[derive(Deserialize)]
struct RemoteRun {
    id: u64,
    name: String,
    head_sha: String,
    external_id: Option<String>,
    status: String,
    conclusion: Option<String>,
    output: Option<RemoteOutput>,
}
#[derive(Deserialize)]
struct RemoteOutput {
    summary: Option<String>,
}
fn parse_run(body: &str) -> Result<RemoteRun, Error> {
    serde_json::from_str(body).map_err(|_| Error::new(ErrorKind::Malformed))
}
fn verify_run(run: &RemoteRun, target: &CheckRunTarget) -> Result<(), Error> {
    if run.id == 0
        || run.name != target.name
        || run.head_sha != target.head_sha
        || run.external_id.as_deref() != Some(&target.external_id.hex())
    {
        return Err(Error::new(ErrorKind::Ambiguous));
    }
    Ok(())
}

impl RemoteRun {
    fn observed_projection(&self) -> Option<AppliedProjection> {
        let summary = self.output.as_ref()?.summary.as_deref()?;
        let (_, suffix) = summary.rsplit_once("\n\n<!-- lattice-check-v1:")?;
        let (version, update) = suffix.strip_suffix(" -->")?.split_once(':')?;
        let version = version.parse().ok()?;
        let mut update_id = [0; 32];
        hex::decode_to_slice(update, &mut update_id).ok()?;
        Some(AppliedProjection {
            version,
            update_id: Identity(update_id),
            terminal: self.status == "completed",
        })
    }

    fn matches_projection(&self, p: &CheckProjection) -> bool {
        let status = match p.status {
            CheckStatus::Queued => "queued",
            CheckStatus::InProgress => "in_progress",
            CheckStatus::Completed => "completed",
        };
        let conclusion = p.conclusion.map(|c| match c {
            Conclusion::Success => "success",
            Conclusion::Failure => "failure",
            Conclusion::Cancelled => "cancelled",
            Conclusion::ActionRequired => "action_required",
        });
        self.status == status
            && self.conclusion.as_deref() == conclusion
            && self
                .output
                .as_ref()
                .and_then(|o| o.summary.as_deref())
                .is_some_and(|summary| summary.ends_with(&marker(p)))
    }
}
