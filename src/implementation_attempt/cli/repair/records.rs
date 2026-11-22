//! Retained repair records of one attempt directory.
//!
//! Under `<attempt>/repair/`, each step writes `step-N.intent` once before
//! its worker effect and `step-N.json` once after its checks; the run ends
//! with one `disposition.json`. Records are write-once: a replay with other
//! content is refused, and recovery reads only the retained disposition.
//! Each check's transient copy of the candidate, `check-N-<check>/`, is
//! removed after the check. The standalone operator's OMP worker also
//! retains each launched step's raw events and launcher observation as
//! `omp-step-N.jsonl` and `omp-step-N.json` before its step record.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]

const DIRECTORY: &str = "repair";
pub(super) const DISPOSITION_FILE: &str = "disposition.json";
/// A disposition retains every revision's changed-path base digests.
const MAX_RECORD_BYTES: usize = 8 * 1_048_576;

/// The retained repair records of one attempt directory.
pub(super) struct Retained {
    root: std::path::PathBuf,
    cancel_request: std::path::PathBuf,
    identity: String,
}

impl Retained {
    /// Create, or reopen, the private records directory of a run.
    pub(super) fn open(directory: &std::path::Path, identity: &str) -> Result<Self, String> {
        let records = Self::at(directory, identity);
        super::super::artifacts::private_directory(&records.root)?;
        Ok(records)
    }

    /// The records of `directory` without creating anything.
    pub(super) fn at(directory: &std::path::Path, identity: &str) -> Self {
        Self {
            root: Self::root(directory),
            cancel_request: directory.join("cancel.request"),
            identity: identity.to_owned(),
        }
    }

    /// The records directory of `directory`.
    pub(super) fn root(directory: &std::path::Path) -> std::path::PathBuf {
        directory.join(DIRECTORY)
    }

    /// Write `value` once as `name`; an identical replay is accepted.
    pub(super) fn retain(&self, name: &str, value: &impl serde::Serialize) -> Result<(), String> {
        let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err("repair record exceeds its bound".into());
        }
        super::super::artifacts::write_once(&self.root.join(name), &bytes).map(|_| ())
    }

    /// Persisted before the step's worker effect.
    pub(super) fn intend(&self, step: &crate::implementation_attempt::repair::step::Request) -> Result<(), String> {
        self.retain(&format!("step-{}.intent", step.iteration), step)
    }

    pub(super) fn retain_disposition(&self, observation: &serde_json::Value) -> Result<(), String> {
        self.retain(DISPOSITION_FILE, observation)
    }

    /// Where one check of one step gets its own copy of the candidate; it
    /// is removed after the check.
    pub(super) fn check_copy(&self, iteration: u32, check: &str) -> std::path::PathBuf {
        self.root.join(format!("check-{iteration}-{check}"))
    }

    /// The bounded bytes of record `name`, `None` when it is missing.
    pub(super) fn read(&self, name: &str) -> Result<Option<Vec<u8>>, String> {
        super::super::artifacts::read_optional(&self.root.join(name), MAX_RECORD_BYTES)
    }

    /// The retained disposition of this exact request, of either schema, as
    /// recovery prints it; a missing one is an unknown outcome, never a reason
    /// to run again.
    pub(super) fn disposition(
        &self,
        request: &super::super::Request,
        binding_identity: &str,
    ) -> Result<serde_json::Value, String> {
        let bytes = match self.read(DISPOSITION_FILE) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Err(Unretained::Missing.message().into()),
            Err(_) => return Err(Unretained::Malformed.message().into()),
        };
        let observation: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| Unretained::Malformed.message())?;
        let text = |field: &str| observation.get(field).and_then(serde_json::Value::as_str);
        match text("schema") {
            Some(crate::implementation_attempt::repair::retained::OBSERVATION_SCHEMA) => {
                if !self.is_same_request(request, text("worker_request"), text("request_identity")) {
                    return Err(Unretained::Crossed.message().into());
                }
            }
            Some(crate::implementation_attempt::repair::retained::LEGACY_OBSERVATION_SCHEMA) => {
                if self.legacy(request, &observation).map_err(Unretained::message)? != binding_identity {
                    return Err(Unretained::Crossed.message().into());
                }
            }
            _ => return Err(Unretained::Malformed.message().into()),
        }
        Ok(observation)
    }

    /// The execution binding a first-schema disposition of this exact request
    /// names: `Malformed` without its required fields, `Crossed` when it names
    /// other request bytes, another envelope or policy, or claims acceptance.
    pub(super) fn legacy(
        &self,
        request: &super::super::Request,
        observation: &serde_json::Value,
    ) -> Result<String, Unretained> {
        let legacy: Legacy = serde::Deserialize::deserialize(observation).map_err(|_| Unretained::Malformed)?;
        debug_assert_eq!(legacy.schema, crate::implementation_attempt::repair::retained::LEGACY_OBSERVATION_SCHEMA);
        let is_bound = self.is_same_request(request, Some(&legacy.worker_request), Some(&legacy.request_identity))
            && legacy.envelope == request.envelope
            && request.repair.as_ref().is_some_and(|policy| policy.identity() == legacy.repair_policy)
            && !legacy.accepted
            && !legacy.integrated;
        if !is_bound {
            return Err(Unretained::Crossed);
        }
        let is_counted = match &legacy.disposition {
            crate::implementation_attempt::repair::state::Disposition::CorrectCandidate { .. } => legacy.steps == 1,
            crate::implementation_attempt::repair::state::Disposition::RepairedCandidate { repairs, .. } => {
                legacy.steps == repairs.saturating_add(1)
            }
            _ => true,
        };
        if !is_counted || legacy.steps > crate::implementation_attempt::repair::MAX_REPAIR_ITERATIONS.saturating_add(1)
        {
            return Err(Unretained::Malformed);
        }
        debug_assert!(is_bound, "only a record of this request reaches its binding");
        Ok(legacy.execution_binding)
    }

    /// Whether a record names this exact request.
    pub(super) fn is_same_request(
        &self,
        request: &super::super::Request,
        worker_request: Option<&str>,
        request_identity: Option<&str>,
    ) -> bool {
        worker_request == Some(self.identity.as_str()) && request_identity == Some(request.admitted_envelope.as_str())
    }

    /// A persisted cancellation request of this exact request. Any other
    /// content, or a failed observation, also cancels.
    pub(super) fn is_cancel_requested(&self) -> bool {
        match super::super::artifacts::read_optional(&self.cancel_request, self.identity.len()) {
            Ok(None) => false,
            Ok(Some(_)) | Err(_) => true,
        }
    }
}

/// The fields every first-schema disposition carries; its other fields are
/// not read.
#[derive(serde::Deserialize)]
struct Legacy {
    schema: String,
    envelope: crate::implementation_attempt::ImplementationAttemptEnvelope,
    request_identity: String,
    worker_request: String,
    execution_binding: String,
    repair_policy: String,
    disposition: crate::implementation_attempt::repair::state::Disposition,
    steps: u32,
    accepted: bool,
    integrated: bool,
}

/// Why no complete retained disposition of the exact request was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum Unretained {
    /// None is retained: the outcome is unknown, never a reason to run again.
    Missing,
    /// A first-schema record, which lacks the evidence export and inventory
    /// re-derive their answers from.
    Legacy,
    /// Unreadable, not JSON, not the complete typed record set, or
    /// contradicting itself.
    Malformed,
    /// The records of other request bytes, another envelope, execution
    /// binding, policy, lineage or dispatch plan.
    Crossed,
}

impl Unretained {
    pub(in super::super) const fn message(self) -> &'static str {
        match self {
            Self::Missing => {
                "repair attempt has no retained disposition; outcome unknown; retain the workspace and reconcile, do \
                 not rerun"
            }
            Self::Legacy => "retained repair disposition predates the retained check evidence",
            Self::Malformed => "retained repair disposition is malformed",
            Self::Crossed => "retained repair disposition does not describe this request",
        }
    }
}
