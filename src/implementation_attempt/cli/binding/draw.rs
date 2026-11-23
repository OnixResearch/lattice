//! The single-shot attempt's lineage draw (#31).
//!
//! It charges exactly the model requests complete broker evidence proves,
//! with host-measured provenance, or the broker ceiling with `unknown`
//! provenance when any intent lacks a terminal record, any request ended
//! uncertain, the evidence conflicts or none was retained. The draw is
//! retained write-once before the attempt result is published; recovery
//! replays that retained draw and never derives a lower one. An attempt
//! without a retained draw reports the ceiling.

// r[impl lattice.implementation_attempt.lineage.scenario.usage]
// r[impl lattice.implementation_attempt.observed_charge]
// r[impl lattice.implementation_attempt.observed_charge.scenario.single_shot]
// r[impl lattice.implementation_attempt.observed_charge.scenario.incomplete]
// r[impl lattice.implementation_attempt.observed_charge.scenario.replay]

/// The retained draw in the attempt directory.
pub(in super::super) const FILE: &str = "lineage-draw.json";
const MAX_FILE_BYTES: usize = 16_384;

type Coverage = lattice_core::execution_binding::serving::usage::ModelRequestCoverage;
type Usage = crate::implementation_attempt::repair::step::Usage;

/// What a single-shot draw is charged against: the lineage, the broker
/// ceiling and the dimensions Lattice does not charge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct SingleShot {
    binding: String,
    reservation: crate::implementation_attempt::lineage::Draws,
    unaccounted: Vec<String>,
}

/// The retained draw, read back strictly.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Retained {
    binding: String,
    usage: Vec<Usage>,
    unaccounted_dimensions: Vec<String>,
}

impl SingleShot {
    /// The draw frame of a single-shot request with a lineage.
    pub(in super::super) fn of(request: &super::super::Request) -> Option<Self> {
        let lineage = request.lineage.as_ref()?;
        debug_assert!(request.repair.is_none(), "a repair attempt reports its own draws");
        let reservation = super::super::repair::first_reservation(request).ok()?;
        debug_assert_eq!(reservation.tool_invocations, 0, "a single shot reserves no tool invocations");
        let mut unaccounted = lineage.unaccounted();
        if lineage.remaining.contains_key(crate::implementation_attempt::lineage::TOOL_INVOCATIONS) {
            unaccounted.push(crate::implementation_attempt::lineage::TOOL_INVOCATIONS.to_owned());
        }
        Some(Self {
            binding: lineage.identity(),
            reservation,
            unaccounted,
        })
    }

    /// The draw `coverage` proves: exactly its observed requests, or the
    /// broker ceiling when the evidence is incomplete or absent.
    pub(in super::super) fn draw(&self, coverage: Option<&Coverage>) -> serde_json::Value {
        let observed = coverage.and_then(|coverage| coverage.observed_requests(None));
        let usage = Usage {
            phase: crate::implementation_attempt::repair::step::Phase::Edit,
            iteration: 1,
            model_requests: observed,
            tool_invocations: None,
            provenance: if observed.is_some() {
                crate::orchestration::UsageProvenance::HostMeasured
            } else {
                crate::orchestration::UsageProvenance::Unknown
            },
            charged: crate::implementation_attempt::lineage::Draws {
                model_requests: observed.unwrap_or(self.reservation.model_requests),
                tool_invocations: self.reservation.tool_invocations,
            },
        };
        serde_json::json!({
            "binding": self.binding,
            "usage": [usage],
            "unaccounted_dimensions": self.unaccounted,
        })
    }

    /// Retain the draw of the persisted `coverage` before the attempt result;
    /// an identical replay is idempotent and a different draw is refused.
    pub(in super::super) fn retain(&self, directory: &std::path::Path, coverage: &Coverage) -> Result<(), String> {
        let bytes = serde_json::to_vec(&self.draw(Some(coverage))).map_err(|error| error.to_string())?;
        debug_assert!(bytes.len() <= MAX_FILE_BYTES, "one usage entry fits the retained bound");
        super::super::artifacts::write_once(&directory.join(FILE), &bytes)
            .map(|_| ())
            .map_err(|_| "the retained lineage draw differs".to_owned())
    }

    /// The retained draw, or the broker ceiling with `unknown` provenance
    /// when none was retained. A malformed or crossed draw is refused.
    pub(in super::super) fn retained(&self, directory: &std::path::Path) -> Result<serde_json::Value, String> {
        let Some(bytes) = super::super::artifacts::read_optional(&directory.join(FILE), MAX_FILE_BYTES)? else {
            return Ok(self.draw(None));
        };
        let retained: Retained =
            serde_json::from_slice(&bytes).map_err(|_| "the retained lineage draw is malformed")?;
        if !self.is_consistent(&retained) {
            return Err("the retained lineage draw is crossed or inconsistent".into());
        }
        serde_json::from_slice(&bytes).map_err(|_| "the retained lineage draw is malformed".into())
    }

    /// One edit usage of this lineage: either an exact host-measured count
    /// charged as observed, or an unknown count charged at the ceiling.
    fn is_consistent(&self, retained: &Retained) -> bool {
        let [usage] = retained.usage.as_slice() else {
            return false;
        };
        let is_bound = retained.binding == self.binding && retained.unaccounted_dimensions == self.unaccounted;
        let is_edit = usage.phase == crate::implementation_attempt::repair::step::Phase::Edit
            && usage.iteration == 1
            && usage.tool_invocations.is_none()
            && usage.charged.tool_invocations == self.reservation.tool_invocations;
        let is_charged = match usage.provenance {
            crate::orchestration::UsageProvenance::HostMeasured => {
                usage.model_requests == Some(usage.charged.model_requests)
            }
            crate::orchestration::UsageProvenance::Unknown => {
                usage.model_requests.is_none() && usage.charged.model_requests == self.reservation.model_requests
            }
            crate::orchestration::UsageProvenance::ProviderReported
            | crate::orchestration::UsageProvenance::WorkerReported
            | crate::orchestration::UsageProvenance::Estimated => false,
        };
        is_bound && is_edit && is_charged
    }
}
