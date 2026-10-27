//! Bounded product adapters selected after exact authorization.

pub(crate) mod credential;
mod operation;

use std::io::Write as _;

const FILE_READ_ABILITY: &str = "file/read";
const FILE_WRITE_ABILITY: &str = "file/write";
const FILE_APPEND_ABILITY: &str = "file/append";
pub(super) const PROCESS_ABILITY: &str = "process/execute";
const NETWORK_ABILITY: &str = "network/request";
const FILE_PLAN_DOMAIN: &[u8] = b"lattice.effect.file-plan.v1";
pub(super) const PROCESS_PLAN_DOMAIN: &[u8] = b"lattice.effect.process-plan.v1";
const NETWORK_PLAN_DOMAIN: &[u8] = b"lattice.effect.network-plan.v1";
const DISPATCH_REQUEST_DOMAIN: &[u8] = b"lattice.effect.dispatch-request.v1";
const PLAN_FIELD_SEPARATOR: &[u8] = &[0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAdapterRequest {
    pub root_id: String,
    pub relative_path: String,
    pub content: Option<Vec<u8>>,
    pub overwrite: bool,
    pub read_max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAdapterOutput {
    Read(Vec<u8>),
    Written { bytes: usize },
}

pub struct Adapters<'a> {
    dispatch: &'a super::dispatch::Shell,
    http: &'a dyn crate::http::HttpClient,
    process: &'a crate::process::core::RuntimeConfig,
    cancellation: Option<&'a std::sync::atomic::AtomicBool>,
}

impl super::dispatch::AuthorizedAdapterRequest for FileAdapterRequest {
    type Output = FileAdapterOutput;

    fn execute(
        &self,
        runtime: &Adapters<'_>,
        authorized: &super::dispatch::AuthorizedAttempt,
    ) -> Result<Self::Output, super::dispatch::Error> {
        runtime.file(authorized, self)
    }
}

impl super::dispatch::AuthorizedAdapterRequest for crate::agent_node::ProcessRequest {
    type Output = crate::agent_node::ProcessOutput;

    fn execute(
        &self,
        runtime: &Adapters<'_>,
        authorized: &super::dispatch::AuthorizedAttempt,
    ) -> Result<Self::Output, super::dispatch::Error> {
        runtime.process(authorized, self, None)
    }
}

impl super::dispatch::AuthorizedAdapterRequest for crate::http::HttpRequestPlan {
    type Output = crate::http::HttpResponse;

    fn execute(
        &self,
        runtime: &Adapters<'_>,
        authorized: &super::dispatch::AuthorizedAttempt,
    ) -> Result<Self::Output, super::dispatch::Error> {
        runtime.network(authorized, self)
    }
}

impl<'a> Adapters<'a> {
    #[must_use]
    pub fn new(
        dispatch: &'a super::dispatch::Shell,
        http: &'a dyn crate::http::HttpClient,
        process: &'a crate::process::core::RuntimeConfig,
    ) -> Self {
        Self {
            dispatch,
            http,
            process,
            cancellation: None,
        }
    }

    #[must_use]
    pub fn new_with_cancellation(
        dispatch: &'a super::dispatch::Shell,
        http: &'a dyn crate::http::HttpClient,
        process: &'a crate::process::core::RuntimeConfig,
        cancellation: &'a std::sync::atomic::AtomicBool,
    ) -> Self {
        Self {
            dispatch,
            http,
            process,
            cancellation: Some(cancellation),
        }
    }

    // r[impl lattice.choregraph_effect_authority.files]
    // r[impl lattice.choregraph_effect_authority.root_authority]
    pub fn file(
        &self,
        authorized: &super::dispatch::AuthorizedAttempt,
        request: &FileAdapterRequest,
    ) -> Result<FileAdapterOutput, super::dispatch::Error> {
        let ability = authorized.attempt.binding.ability.as_str();
        let is_ability_supported = matches!(ability, FILE_READ_ABILITY | FILE_WRITE_ABILITY | FILE_APPEND_ABILITY);
        let is_payload_match =
            plan_identity(FILE_PLAN_DOMAIN, &file_plan_fields(request)) == authorized.attempt.binding.payload.as_str();
        if !is_ability_supported || !is_payload_match {
            return Err(super::dispatch::Error::AdapterRejected);
        }
        let relative = self.dispatch.admit_file_path(authorized, super::dispatch::FileTarget {
            root_id: &request.root_id,
            relative_path: &request.relative_path,
        })?;
        let mut output = None;
        let mut operation = |root: &cap_std::fs::Dir| {
            output = Some(match ability {
                FILE_READ_ABILITY => {
                    let file = crate::capability::open_relative_read(root, relative.as_path())
                        .map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    let bytes = crate::capability::read_bounded(file, request.read_max_bytes)
                        .map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    if bytes.len() > request.read_max_bytes {
                        return Err(super::dispatch::Error::AdapterRejected);
                    }
                    FileAdapterOutput::Read(bytes)
                }
                FILE_WRITE_ABILITY => {
                    let bytes = request.content.as_deref().unwrap_or_default();
                    let mut options = cap_std::fs::OpenOptions::new();
                    options.write(true);
                    if request.overwrite {
                        options.create(true).truncate(true);
                    } else {
                        options.create_new(true);
                    }
                    let mut file = root
                        .open_with(relative.as_path(), &options)
                        .map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    file.write_all(bytes).map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    FileAdapterOutput::Written { bytes: bytes.len() }
                }
                FILE_APPEND_ABILITY => {
                    let bytes = request.content.as_deref().unwrap_or_default();
                    let mut options = cap_std::fs::OpenOptions::new();
                    options.create(true).append(true);
                    let mut file = root
                        .open_with(relative.as_path(), &options)
                        .map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    file.write_all(bytes).map_err(|_| super::dispatch::Error::AdapterRejected)?;
                    FileAdapterOutput::Written { bytes: bytes.len() }
                }
                _ => return Err(super::dispatch::Error::AdapterRejected),
            });
            Ok(())
        };
        self.dispatch.with_file_root(authorized, &request.root_id, &mut operation)?;
        output.ok_or(super::dispatch::Error::AdapterRejected)
    }

    // r[impl lattice.choregraph_effect_authority.adapters]
    // r[impl lattice.choregraph_effect_authority.adapters.scenario.process]
    pub fn process(
        &self,
        authorized: &super::dispatch::AuthorizedAttempt,
        request: &crate::agent_node::ProcessRequest,
        cancellation: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<crate::agent_node::ProcessOutput, super::dispatch::Error> {
        let is_ability_match = authorized.attempt.binding.ability.as_str() == PROCESS_ABILITY;
        let is_payload_match = plan_identity(PROCESS_PLAN_DOMAIN, &process_plan_fields(request))
            == authorized.attempt.binding.payload.as_str();
        if !is_ability_match || !is_payload_match {
            return Err(super::dispatch::Error::AdapterRejected);
        }
        let cancellation = cancellation.or(self.cancellation);
        crate::process::shell::run_bounded(request, self.process, cancellation)
            .map_err(|_| super::dispatch::Error::AdapterRejected)
    }

    // r[impl lattice.choregraph_effect_authority.adapters]
    pub fn network(
        &self,
        authorized: &super::dispatch::AuthorizedAttempt,
        request: &crate::http::HttpRequestPlan,
    ) -> Result<crate::http::HttpResponse, super::dispatch::Error> {
        let is_ability_match = authorized.attempt.binding.ability.as_str() == NETWORK_ABILITY;
        let is_payload_match = plan_identity(NETWORK_PLAN_DOMAIN, &network_plan_fields(request))
            == authorized.attempt.binding.payload.as_str();
        let is_request_admitted =
            crate::http::validate_http_method(&request.method) && crate::http::validate_http_url(&request.url);
        if !is_ability_match || !is_payload_match || !is_request_admitted {
            return Err(super::dispatch::Error::AdapterRejected);
        }
        self.http
            .execute_with_cancellation(request, self.cancellation)
            .map_err(|_| super::dispatch::Error::AdapterRejected)
    }
}

#[must_use]
pub fn file_plan_identity(request: &FileAdapterRequest) -> String {
    plan_identity(FILE_PLAN_DOMAIN, &file_plan_fields(request))
}

#[must_use]
pub fn process_plan_identity(request: &crate::agent_node::ProcessRequest) -> String {
    plan_identity(PROCESS_PLAN_DOMAIN, &process_plan_fields(request))
}

#[must_use]
pub fn network_plan_identity(request: &crate::http::HttpRequestPlan) -> String {
    plan_identity(NETWORK_PLAN_DOMAIN, &network_plan_fields(request))
}

#[must_use]
pub fn dispatch_request_identity(payload_identity: &str) -> String {
    plan_identity(DISPATCH_REQUEST_DOMAIN, &[payload_identity.to_owned()])
}

fn file_plan_fields(request: &FileAdapterRequest) -> Vec<String> {
    vec![
        request.root_id.clone(),
        request.relative_path.clone(),
        blake3::hash(request.content.as_deref().unwrap_or_default()).to_hex().to_string(),
        request.overwrite.to_string(),
        request.read_max_bytes.to_string(),
    ]
}

pub(super) fn process_plan_fields(request: &crate::agent_node::ProcessRequest) -> Vec<String> {
    let mut fields = vec![
        request.program.clone(),
        request.cwd.clone().unwrap_or_default(),
        blake3::hash(&request.stdin).to_hex().to_string(),
        request.timeout_secs.to_string(),
        request.output_limit_bytes.to_string(),
    ];
    fields.extend(request.args.iter().cloned());
    fields.extend(request.env.iter().map(|(key, value)| format!("{key}={value}")));
    fields
}

fn network_plan_fields(request: &crate::http::HttpRequestPlan) -> Vec<String> {
    let mut fields = vec![
        crate::http::normalized_method(&request.method),
        request.url.clone(),
        request.body.clone().unwrap_or_default(),
        request.timeout_secs.to_string(),
    ];
    let mut headers = request.headers.iter().collect::<Vec<_>>();
    headers.sort_by(|left, right| left.0.cmp(right.0));
    fields.extend(headers.into_iter().map(|(key, value)| format!("{key}:{value}")));
    fields
}

pub(super) fn plan_identity(domain: &[u8], fields: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        hasher.update(PLAN_FIELD_SEPARATOR);
        hasher.update(field.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub struct ConfiguredRoots {
    roots: std::collections::BTreeMap<String, cap_std::fs::Dir>,
}

impl ConfiguredRoots {
    #[must_use]
    pub fn new(roots: std::collections::BTreeMap<String, cap_std::fs::Dir>) -> Self {
        Self { roots }
    }
}

impl super::dispatch::ConfiguredRootRegistry for ConfiguredRoots {
    fn contains_root(&self, root_id: &str) -> bool {
        self.roots.contains_key(root_id)
    }

    fn with_root(
        &self,
        root_id: &str,
        operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), super::dispatch::Error>,
    ) -> Result<(), super::dispatch::Error> {
        let root = self.roots.get(root_id).ok_or(super::dispatch::Error::UnknownRoot)?;
        operation(root)
    }
}
