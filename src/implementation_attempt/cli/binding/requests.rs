//! Durable per-request serving evidence in the attempt directory.
//!
//! `model-requests/<hex>.intent.json` is written with fsync before upstream
//! contact and `model-requests/<hex>.record.json` after the gate refused or
//! the call ended. Both are write-once; a different second write is refused.
//! Recovery and usage read them back and never call the serving port.

// r[impl lattice.serving_binding.request_observation]
// r[impl lattice.serving_binding.usage.scenario.dedup]

const DIRECTORY: &str = "model-requests";
const INTENT_SUFFIX: &str = ".intent.json";
const RECORD_SUFFIX: &str = ".record.json";
const MAX_FILE_BYTES: usize = 16_384;
/// Every served route's request ceiling, with an intent and a record each.
const MAX_FILES: usize = 4_096;

type Intent = lattice_core::execution_binding::serving::record::DispatchIntent;
type Record = lattice_core::execution_binding::serving::record::Observation;
type Admitted = lattice_core::execution_binding::admission::bound::Admitted;

/// An OMP attempt may serve only its brokered `primary` route, with the
/// broker's exact model, and its caller identities must be producer tokens.
/// Anything else is refused before any binding, intent or worker effect.
// r[impl lattice.serving_binding.declaration.scenario.unsupported]
pub(in super::super) fn validate(request: &super::super::Request) -> Result<(), String> {
    let Some(serving) = &request.execution_binding.serving else {
        return Ok(());
    };
    lattice_core::execution_binding::serving::validate(serving, &request.execution_binding.model_routes)
        .map_err(|defect| format!("serving declaration is unsupported: {defect:?}"))?;
    let [route] = serving.routes.as_slice() else {
        return Err("OMP attempt workers serve exactly their primary route".into());
    };
    if route.name != super::PRIMARY_ROUTE || route.model_id != request.omp.model_broker.model {
        return Err("served route differs from the brokered primary route or its exact model".into());
    }
    let is_caller_token = [&request.envelope.attempt_id, &request.envelope.worker_profile]
        .into_iter()
        .all(|value| lattice_core::execution_binding::serving::is_producer_token(value));
    if !is_caller_token {
        return Err("attempt or worker profile identity is not a serving caller token".into());
    }
    debug_assert_eq!(route.name, super::PRIMARY_ROUTE);
    Ok(())
}

/// Writes request evidence of one attempt directory.
pub(in super::super) struct RequestFiles {
    directory: std::path::PathBuf,
}

impl RequestFiles {
    pub(in super::super) fn new(attempt_directory: &std::path::Path) -> Result<Self, String> {
        let directory = attempt_directory.join(DIRECTORY);
        super::super::artifacts::private_directory(&directory)?;
        Ok(Self { directory })
    }

    fn write(&self, request_id: &str, is_intent: bool, bytes: &[u8]) -> Result<(), String> {
        let name = file_name(request_id, is_intent)?;
        super::super::artifacts::write_once(&self.directory.join(name), bytes)
            .map(|_| ())
            .map_err(|_| "model request evidence already exists with different content".to_string())
    }
}

impl crate::execution_binding::serving::Sink for RequestFiles {
    fn write_intent(&mut self, intent: &Intent) -> Result<(), String> {
        let bytes = serde_json::to_vec(intent).map_err(|error| error.to_string())?;
        self.write(&intent.request_id, true, &bytes)
    }

    fn write_record(&mut self, record: &Record) -> Result<(), String> {
        let bytes = serde_json::to_vec(record).map_err(|error| error.to_string())?;
        self.write(&record.request_id, false, &bytes)
    }
}

fn file_name(request_id: &str, is_intent: bool) -> Result<String, String> {
    let digest = request_id
        .strip_prefix("blake3:")
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("model request identity is not canonical")?;
    let suffix = if is_intent { INTENT_SUFFIX } else { RECORD_SUFFIX };
    Ok(format!("{digest}{suffix}"))
}

/// Read retained intents and records; a missing directory is no evidence.
pub(in super::super) fn load(attempt_directory: &std::path::Path) -> Result<(Vec<Intent>, Vec<Record>), String> {
    let directory = attempt_directory.join(DIRECTORY);
    let entries = match std::fs::read_dir(&directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), Vec::new())),
        Err(_) => return Err("retained model request evidence is unreadable".into()),
        Ok(entries) => entries,
    };
    let mut names = Vec::with_capacity(MAX_FILES);
    for entry in entries.take(MAX_FILES.saturating_add(1)) {
        let name = entry.map_err(|_| "retained model request evidence is unreadable")?.file_name();
        names.push(name.into_string().map_err(|_| "retained model request name is not UTF-8")?);
    }
    if names.len() > MAX_FILES {
        return Err("retained model request evidence exceeds its bound".into());
    }
    names.sort_unstable();
    let mut intents = Vec::with_capacity(names.len());
    let mut records = Vec::with_capacity(names.len());
    for name in &names {
        let bytes = super::super::artifacts::read_bounded(&directory.join(name), MAX_FILE_BYTES)?;
        if name.ends_with(INTENT_SUFFIX) {
            intents.push(serde_json::from_slice(&bytes).map_err(|_| "retained model request intent is malformed")?);
        } else if name.ends_with(RECORD_SUFFIX) {
            records.push(serde_json::from_slice(&bytes).map_err(|_| "retained model request record is malformed")?);
        } else {
            return Err("retained model request evidence has an unexpected file".into());
        }
    }
    debug_assert!(intents.len().saturating_add(records.len()) == names.len());
    Ok((intents, records))
}

/// Summarize the retained evidence for #13; conflicts fail closed.
pub(in super::super) fn coverage(
    admitted: &Admitted,
    attempt_directory: &std::path::Path,
) -> Result<lattice_core::execution_binding::serving::usage::ModelRequestCoverage, String> {
    let (intents, records) = load(attempt_directory)?;
    lattice_core::execution_binding::serving::usage::summarize(admitted, &intents, &records)
        .map_err(|conflict| format!("retained model request evidence conflicts: {conflict:?}"))
}
