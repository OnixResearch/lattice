//! Animus frozen-policy leaf behind Lattice's typed policy operation (Animus #15, Lattice #23).
//!
//! This shell is the only inference site of the typed policy operation. The host loads artifact
//! bytes and hands them over; admission pins the profile's exact artifact identity, and the
//! decision never carries a fallback, so Lattice owns fallback resolution. Sessions and recurrent
//! state stay unsupported, as for the Animus leaf agent.

/// One host-loaded frozen-policy artifact, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafAdapter {
    artifact: Option<Vec<u8>>,
}

impl LeafAdapter {
    /// An adapter over host-loaded artifact bytes; a file name is never identity.
    #[must_use]
    pub const fn new(artifact: Vec<u8>) -> Self {
        Self {
            artifact: Some(artifact),
        }
    }

    /// An adapter without an artifact; every decision reports `unavailable`.
    #[must_use]
    pub const fn without_artifact() -> Self {
        Self { artifact: None }
    }
}

// r[impl lattice.typed_policy.invocation]
impl crate::workflow::decision::Adapter for LeafAdapter {
    fn invoke(
        &self,
        invocation: &crate::workflow::decision::binding::Invocation,
    ) -> crate::workflow::decision::Response {
        let admitted =
            self.artifact.as_deref().map_or(Err(animus_frozen_policy::AdmissionError::Unavailable), |bytes| {
                animus_frozen_policy::admit_policy(bytes, &invocation.binding.artifact)
            });
        let request = animus_frozen_policy::DecisionRequest {
            binding: invocation.binding.clone(),
            features: invocation.features.clone(),
            eligible: invocation.candidates.clone(),
            fallback: None,
            max_operations: invocation.max_operation_count,
        };
        let decision = animus_frozen_policy::decide(admitted.as_ref(), &request);
        crate::workflow::decision::Response::Receipt(decision.receipt.encode().into_bytes())
    }
}
