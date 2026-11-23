//! Durable per-request model evidence in the attempt directory.
//!
//! Every attempt broker creates `model-requests/` before its worker starts.
//! `model-requests/<hex>.intent.json` is written with fsync before upstream
//! contact and `model-requests/<hex>.record.json` after the gate refused or
//! the call ended: serving-gate files for a binding with a serving
//! declaration, brokered files for one without. Both are write-once; a
//! different second write is refused. Recovery and usage read them back and
//! never call the serving port.

// r[impl lattice.serving_binding.request_observation]
// r[impl lattice.serving_binding.usage.scenario.dedup]
// r[impl lattice.serving_binding.brokered_evidence]

const DIRECTORY: &str = "model-requests";
const INTENT_SUFFIX: &str = ".intent.json";
const RECORD_SUFFIX: &str = ".record.json";
const MAX_FILE_BYTES: usize = 16_384;
/// An intent and a record for each of an attempt's brokered requests, which
/// also covers every served route's request ceiling.
const MAX_FILES: usize = lattice_core::execution_binding::serving::brokered::MAX_REQUESTS.saturating_mul(2);

type Intent = lattice_core::execution_binding::serving::record::DispatchIntent;
type Record = lattice_core::execution_binding::serving::record::Observation;
type BrokeredIntent = lattice_core::execution_binding::serving::brokered::Intent;
type BrokeredRecord = lattice_core::execution_binding::serving::brokered::Record;
type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
/// Retained intents and records, or `None` when no broker created the directory.
type Retained<I, R> = Option<(Vec<I>, Vec<R>)>;

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
    /// Create the evidence directory before any broker runs and sync it into
    /// the attempt directory, so a retained empty directory proves that no
    /// request reached upstream.
    pub(in super::super) fn new(attempt_directory: &std::path::Path) -> Result<Self, String> {
        let directory = attempt_directory.join(DIRECTORY);
        super::super::artifacts::private_directory(&directory)?;
        std::fs::File::open(attempt_directory)
            .and_then(|parent| parent.sync_all())
            .map_err(|_| "model request evidence directory is not durable")?;
        Ok(Self { directory })
    }

    fn write(&self, request_id: &str, is_intent: bool, bytes: &[u8]) -> Result<(), String> {
        let name = file_name(request_id, is_intent)?;
        // A record whose sync fails is removed: the broker stops, and the
        // surviving intent then reads as an unconfirmed request, never as an
        // exact charge from a file that was never durable.
        super::super::artifacts::write_once_durable(&self.directory.join(name), bytes)
            .map(|_| ())
            .map_err(|_| "model request evidence already exists with different content".to_string())
    }

    /// Create-only: an existing intent, identical or not, means the request
    /// was already admitted once, so it is refused before any upstream contact.
    fn create(&self, request_id: &str, bytes: &[u8]) -> Result<(), String> {
        let name = file_name(request_id, true)?;
        if super::super::artifacts::create_new(&self.directory.join(name), bytes)? {
            return Ok(());
        }
        Err("model request was already admitted".to_owned())
    }
}

impl crate::execution_binding::serving::Sink for RequestFiles {
    fn create_intent(&mut self, intent: &Intent) -> Result<(), String> {
        let bytes = serde_json::to_vec(intent).map_err(|error| error.to_string())?;
        self.create(&intent.request_id, &bytes)
    }

    fn write_record(&mut self, record: &Record) -> Result<(), String> {
        let bytes = serde_json::to_vec(record).map_err(|error| error.to_string())?;
        self.write(&record.request_id, false, &bytes)
    }

    /// Read back from the retained files, so a restarted session continues
    /// after every ordinal an earlier session used.
    fn highest_ordinal(&self, route_name: &str) -> Result<u64, String> {
        let attempt_directory = self.directory.parent().ok_or("model request evidence has no attempt directory")?;
        let (intents, records) = load::<Intent, Record>(attempt_directory)?.unwrap_or_default();
        let intents = intents.iter().filter(|intent| intent.route_name == route_name).map(|intent| intent.ordinal);
        let records = records.iter().filter(|record| record.route_name == route_name).map(|record| record.ordinal);
        Ok(intents.chain(records).max().unwrap_or(0))
    }
}

impl crate::candidate_worker::broker::RequestEvidence for RequestFiles {
    fn create_intent(&mut self, intent: &BrokeredIntent) -> Result<(), String> {
        let bytes = serde_json::to_vec(intent).map_err(|error| error.to_string())?;
        self.create(&intent.request_id, &bytes)
    }

    fn write_record(&mut self, record: &BrokeredRecord) -> Result<(), String> {
        let bytes = serde_json::to_vec(record).map_err(|error| error.to_string())?;
        self.write(&record.request_id, false, &bytes)
    }
}

/// The recorded route of a binding without a serving declaration: the OMP
/// broker's `primary` route, retained in `files`.
pub(in super::super) fn recorded<'a>(
    admitted: &Admitted,
    files: &'a mut RequestFiles,
) -> crate::candidate_worker::broker::RecordedRoute<'a> {
    debug_assert!(admitted.serving.is_none(), "a served binding records through the serving gate");
    crate::candidate_worker::broker::RecordedRoute::new(
        lattice_core::execution_binding::serving::brokered::Context::new(admitted, super::PRIMARY_ROUTE),
        files,
    )
}

fn file_name(request_id: &str, is_intent: bool) -> Result<String, String> {
    let digest = request_id
        .strip_prefix("blake3:")
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("model request identity is not canonical")?;
    let suffix = if is_intent { INTENT_SUFFIX } else { RECORD_SUFFIX };
    Ok(format!("{digest}{suffix}"))
}

/// Retained evidence names the request it records, so its file name can be
/// checked against that identity and not only its suffix.
pub(in super::super) trait NamesRequest {
    fn request_id(&self) -> &str;
}

impl NamesRequest for Intent {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl NamesRequest for Record {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl NamesRequest for BrokeredIntent {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl NamesRequest for BrokeredRecord {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

/// The canonical file name of `request_id` must equal the file read: a stray
/// rename never turns retained evidence into a different request's charge.
const MISNAMED: &str = "retained model request evidence does not name its request";

/// Read retained intents and records; `None` when the directory is missing,
/// which is no evidence at all.
pub(in super::super) fn load<
    I: serde::de::DeserializeOwned + NamesRequest,
    R: serde::de::DeserializeOwned + NamesRequest,
>(
    attempt_directory: &std::path::Path,
) -> Result<Retained<I, R>, String> {
    let directory = attempt_directory.join(DIRECTORY);
    let entries = match std::fs::read_dir(&directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
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
            let intent: I = serde_json::from_slice(&bytes).map_err(|_| "retained model request intent is malformed")?;
            if file_name(intent.request_id(), true)? != name.as_str() {
                return Err(MISNAMED.into());
            }
            intents.push(intent);
        } else if name.ends_with(RECORD_SUFFIX) {
            let record: R = serde_json::from_slice(&bytes).map_err(|_| "retained model request record is malformed")?;
            if file_name(record.request_id(), false)? != name.as_str() {
                return Err(MISNAMED.into());
            }
            records.push(record);
        } else {
            return Err("retained model request evidence has an unexpected file".into());
        }
    }
    debug_assert!(intents.len().saturating_add(records.len()) == names.len());
    Ok(Some((intents, records)))
}

/// Summarize the retained evidence for #13; conflicts fail closed. Only the
/// directory the broker pre-creates can prove a count, zero included: without
/// it the coverage is `unknown`, served or not (`no_model` stays `no_model`),
/// so lost evidence is never charged as zero requests.
// r[impl lattice.serving_binding.brokered_evidence.scenario.charged]
pub(in super::super) fn coverage(
    admitted: &Admitted,
    attempt_directory: &std::path::Path,
) -> Result<lattice_core::execution_binding::serving::usage::ModelRequestCoverage, String> {
    let summary = if admitted.serving.is_some() {
        match load::<Intent, Record>(attempt_directory)? {
            Some((intents, records)) => {
                lattice_core::execution_binding::serving::usage::summarize(admitted, &intents, &records)
            }
            None => Ok(lattice_core::execution_binding::serving::usage::ModelRequestCoverage::Unknown),
        }
    } else {
        match load::<BrokeredIntent, BrokeredRecord>(attempt_directory)? {
            Some((intents, records)) => {
                lattice_core::execution_binding::serving::brokered::summarize(admitted, &intents, &records)
            }
            // No evidence: `unknown`, or `no_model` for a no-model binding.
            None => lattice_core::execution_binding::serving::usage::summarize(admitted, &[], &[]),
        }
    };
    summary.map_err(|conflict| format!("retained model request evidence conflicts: {conflict:?}"))
}
