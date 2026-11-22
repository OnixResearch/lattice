//! Retained repair records of one attempt directory.
//!
//! Under `<attempt>/repair/`, each step writes `step-N.intent` once before
//! its worker effect and `step-N.json` once after its checks; the run ends
//! with one `disposition.json`. Records are write-once: a replay with other
//! content is refused, and recovery reads only the retained disposition.
//! Each check's transient copy of the candidate, `check-N-<check>/`, is
//! removed after the check.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]

const DIRECTORY: &str = "repair";
const DISPOSITION_FILE: &str = "disposition.json";
pub(super) const OBSERVATION_SCHEMA: &str = "lattice.implementation-attempt-repair-observation.v1";
pub(super) const STEP_SCHEMA: &str = "lattice.implementation-attempt-repair-step.v1";
const MAX_RECORD_BYTES: usize = 1_048_576;

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
            root: directory.join(DIRECTORY),
            cancel_request: directory.join("cancel.request"),
            identity: identity.to_owned(),
        }
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

    /// The retained disposition of this exact request; a missing one is an
    /// unknown outcome, never a reason to run again.
    pub(super) fn disposition(&self, request: &super::super::Request) -> Result<serde_json::Value, String> {
        self.retained(request).map_err(|unretained| unretained.message().to_owned())
    }

    /// [`Self::disposition`] with a typed reason when none is read.
    pub(super) fn retained(&self, request: &super::super::Request) -> Result<serde_json::Value, Unretained> {
        let path = self.root.join(DISPOSITION_FILE);
        let bytes = match super::super::artifacts::read_optional(&path, MAX_RECORD_BYTES) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Err(Unretained::Missing),
            Err(_) => return Err(Unretained::Malformed),
        };
        let observation: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| Unretained::Malformed)?;
        let text = |field: &str| observation.get(field).and_then(serde_json::Value::as_str);
        if text("schema") != Some(OBSERVATION_SCHEMA) {
            return Err(Unretained::Malformed);
        }
        let is_same_request = text("worker_request") == Some(self.identity.as_str())
            && text("request_identity") == Some(request.admitted_envelope.as_str());
        if !is_same_request {
            return Err(Unretained::Crossed);
        }
        Ok(observation)
    }

    /// The retained record of step `iteration`; `None` when it is missing,
    /// unreadable or not JSON.
    pub(super) fn step(&self, iteration: u32) -> Option<serde_json::Value> {
        let path = self.root.join(format!("step-{iteration}.json"));
        let bytes = super::super::artifacts::read_optional(&path, MAX_RECORD_BYTES).ok()??;
        serde_json::from_slice(&bytes).ok()
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

/// Why no retained disposition of the exact request was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum Unretained {
    /// None is retained: the outcome is unknown, never a reason to run again.
    Missing,
    /// Unreadable, not JSON or not a repair observation.
    Malformed,
    /// The observation of other request bytes or another admitted envelope.
    Crossed,
}

impl Unretained {
    pub(in super::super) const fn message(self) -> &'static str {
        match self {
            Self::Missing => {
                "repair attempt has no retained disposition; outcome unknown; retain the workspace and reconcile, do \
                 not rerun"
            }
            Self::Malformed => "retained repair disposition is malformed",
            Self::Crossed => "retained repair disposition does not describe this request",
        }
    }
}
