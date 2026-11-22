//! The operator's check catalog of the standalone CLI's repair checks.
//!
//! The catalog is operator input passed as `run --repair-checks`, never read
//! from a candidate or the worker's workspace. Each entry pins one immutable
//! Nix store program by path and BLAKE3, its arguments and the failure class
//! a non-zero exit reports. An entry's canonical identity is the
//! `configuration` a repair policy declares for its check, and the executor
//! profile's identity is the policy's `check_executor`. Each check then runs
//! as one bounded process through the candidate worker's generic bubblewrap
//! route on a read-only copy of the candidate, with a cleared environment, a
//! private network, no model broker and bounded output.
//!
//! This module is pure: the shell reads the bytes and verifies the pins.

// r[impl lattice.implementation_attempt.operator_repair.scenario.checks]

/// Wire schema of a check catalog.
pub const SCHEMA: &str = "lattice.implementation-attempt-check-catalog.v1";
/// Upper bound on the catalog's bytes.
pub const MAX_CATALOG_BYTES: usize = 65_536;
/// Upper bound on one check's arguments.
pub const MAX_ARGS: usize = 32;
/// Upper bound on one argument's bytes.
pub const MAX_ARG_BYTES: usize = 4_096;
/// Each of a check's stdout and stderr is bounded to this many bytes.
pub const OUTPUT_LIMIT_BYTES: usize = 1_048_576;
const STORE_PREFIX: &str = "/nix/store/";
const CONFIGURATION_DOMAIN: &str = "lattice.implementation-attempt-check-catalog.v1 configuration";
const EXECUTOR_DOMAIN: &str = "lattice.implementation-attempt-check-catalog.v1 executor";
/// The launch the executor identity frames: the generic bubblewrap route
/// with a cleared environment, a private network and no broker.
const EXECUTOR_ROUTE: &str = "candidate-worker-bubblewrap;clearenv;unshare-net;no-broker;read-only-copy";

/// One declared check's pinned process.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub name: String,
    pub program: crate::candidate_worker::broker::Executable,
    pub args: Vec<String>,
    /// The failure class a non-zero exit reports.
    pub failure_class: String,
}

/// The operator's check catalog.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pins {
    pub schema: String,
    /// The reviewed bubblewrap executable every check runs under.
    pub sandbox: crate::candidate_worker::broker::Executable,
    /// Strictly sorted by name.
    pub checks: Vec<Entry>,
}

/// Why a catalog is not admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("check catalog is unavailable")]
    Unavailable,
    #[error("check catalog lies inside the attempt workspace")]
    Workspace,
    #[error("check catalog exceeds its byte bound")]
    Oversized,
    #[error("check catalog is malformed")]
    Malformed,
    #[error("check catalog entries are empty, too many, unsorted, duplicated or malformed")]
    Entries,
    #[error("check catalog arguments exceed their bounds")]
    Arguments,
    #[error("check catalog program or sandbox is not a pinned Nix store executable")]
    Unpinned,
    #[error("check catalog program or sandbox bytes differ from their pins")]
    Mismatched,
    #[error("check catalog does not match the repair policy's executor and checks")]
    Unmatched,
}

/// Decode and validate catalog `bytes`.
pub fn parse(bytes: &[u8]) -> Result<Pins, Error> {
    if bytes.len() > MAX_CATALOG_BYTES {
        return Err(Error::Oversized);
    }
    let catalog: Pins = serde_json::from_slice(bytes).map_err(|_| Error::Malformed)?;
    catalog.validate()?;
    Ok(catalog)
}

impl Pins {
    fn validate(&self) -> Result<(), Error> {
        if self.schema != SCHEMA {
            return Err(Error::Malformed);
        }
        let names = self.checks.iter().map(|entry| entry.name.as_str());
        let is_sorted = super::super::is_strictly_sorted(names);
        let is_well_named = self
            .checks
            .iter()
            .all(|entry| super::super::is_name(&entry.name) && super::super::is_name(&entry.failure_class));
        if self.checks.len() > super::super::MAX_CHECKS || !is_sorted || !is_well_named {
            return Err(Error::Entries);
        }
        let has_bounded_args = self.checks.iter().all(|entry| {
            entry.args.len() <= MAX_ARGS
                && entry.args.iter().all(|arg| arg.len() <= MAX_ARG_BYTES && !arg.contains('\0'))
        });
        if !has_bounded_args {
            return Err(Error::Arguments);
        }
        if !is_pinned(&self.sandbox) || !self.checks.iter().all(|entry| is_pinned(&entry.program)) {
            return Err(Error::Unpinned);
        }
        Ok(())
    }

    /// The executor profile's identity, the policy's `check_executor`.
    #[must_use]
    pub fn executor(&self) -> String {
        let mut hasher = blake3::Hasher::new_derive_key(EXECUTOR_DOMAIN);
        for part in [SCHEMA, EXECUTOR_ROUTE, &self.sandbox.blake3] {
            super::super::super::lineage::frame(&mut hasher, part.as_bytes());
        }
        super::super::super::lineage::frame(&mut hasher, self.sandbox.path.as_os_str().as_encoded_bytes());
        super::super::super::lineage::frame(&mut hasher, OUTPUT_LIMIT_BYTES.to_string().as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// The entry of check `name`.
    #[must_use]
    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.checks.iter().find(|entry| entry.name == name)
    }

    /// Admit the catalog for `policy`: its executor must be the policy's
    /// `check_executor`, and it must hold exactly the policy's checks, each
    /// with the declared configuration.
    pub fn admit(&self, policy: &super::super::Policy) -> Result<(), Error> {
        let is_matched = self.executor() == policy.check_executor
            && self.checks.len() == policy.checks.len()
            && policy.checks.iter().all(|declared| {
                self.entry(&declared.name).is_some_and(|entry| entry.configuration() == declared.configuration)
            });
        if !is_matched {
            return Err(Error::Unmatched);
        }
        Ok(())
    }
}

impl Entry {
    /// The entry's canonical identity, the policy check's `configuration`.
    #[must_use]
    pub fn configuration(&self) -> String {
        let mut hasher = blake3::Hasher::new_derive_key(CONFIGURATION_DOMAIN);
        super::super::super::lineage::frame(&mut hasher, self.name.as_bytes());
        super::super::super::lineage::frame(&mut hasher, self.program.path.as_os_str().as_encoded_bytes());
        super::super::super::lineage::frame(&mut hasher, self.program.blake3.as_bytes());
        super::super::super::lineage::frame_count(&mut hasher, self.args.len());
        for arg in &self.args {
            super::super::super::lineage::frame(&mut hasher, arg.as_bytes());
        }
        super::super::super::lineage::frame(&mut hasher, self.failure_class.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

/// A pin names an absolute, normal path inside the Nix store and a BLAKE3.
fn is_pinned(executable: &crate::candidate_worker::broker::Executable) -> bool {
    let path = &executable.path;
    let is_normal = path.components().skip(1).all(|part| matches!(part, std::path::Component::Normal(_)));
    path.to_str().is_some_and(|text| text.starts_with(STORE_PREFIX))
        && is_normal
        && super::super::super::is_blake3(&executable.blake3)
}

/// The check outcome of one launcher observation. Only an exit observed on
/// an unchanged copy decides: zero passes and non-zero fails with the
/// entry's class. A copy the check changed, a stop at the deadline or output
/// limit, a stop without an exit status, cancellation and anything the
/// launcher could not observe are unknown.
#[must_use]
pub fn outcome(observed: &crate::candidate_worker::Outcome, failure_class: &str) -> super::Outcome {
    let stop = crate::candidate_worker::omp::stop::of(observed);
    let outcome = match (observed, stop) {
        (crate::candidate_worker::Outcome::Unchanged { .. }, _) => super::Outcome::Passed,
        (_, Some(crate::candidate_worker::omp::stop::Termination::Exited { status_code: Some(_) })) => {
            super::Outcome::Failed {
                class: failure_class.to_owned(),
            }
        }
        _ => super::Outcome::Unknown,
    };
    debug_assert!(super::super::is_name(failure_class), "an admitted entry's failure class");
    debug_assert!(
        outcome != super::Outcome::Passed || matches!(observed, crate::candidate_worker::Outcome::Unchanged { .. }),
        "only an unchanged copy passes"
    );
    outcome
}

#[cfg(test)]
mod tests;
