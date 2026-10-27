//! Pure generation-scoped ownership for reusable response connections.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::path_segment_repetition,
        reason = "response ownership is the stable public domain term for this focused module"
    )
)]

const MAX_LABEL_BYTES: usize = 128;

/// Reusable response-connection state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseConnectionOwnership {
    /// The connection can accept one new generation owner.
    Vacant,
    /// One exact generation owns the connection.
    Owned {
        /// Safe connection identity.
        connection_id: String,
        /// Safe generation identity.
        generation_id: String,
    },
    /// The connection must be discarded by its host.
    Poisoned,
}

/// Response ownership observation or request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseOwnershipEvent<'a> {
    /// Claim one vacant connection.
    Claim {
        /// Safe connection identity.
        connection_id: &'a str,
        /// Safe generation identity.
        generation_id: &'a str,
    },
    /// Accept one frame for the named generation.
    Frame {
        /// Safe generation identity.
        generation_id: &'a str,
    },
    /// Observe the admitted terminal frame.
    Terminal {
        /// Safe generation identity.
        generation_id: &'a str,
    },
    /// Observe interruption before terminal completion.
    Interrupted,
    /// Observe malformed framing.
    Malformed,
    /// Observe return before the terminal frame.
    EarlyReturn,
}

/// Stable response ownership failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseOwnershipIssue {
    /// A label is empty, unsafe, or overbound.
    InvalidIdentity,
    /// Another generation already owns the connection.
    ConcurrentOwner,
    /// A frame or terminal observation names another generation.
    GenerationMismatch,
    /// A poisoned connection cannot be reused.
    PoisonedConnection,
    /// The transition is invalid for the current state.
    InvalidTransition,
}

/// Safe response ownership failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseOwnershipError {
    issue: ResponseOwnershipIssue,
}

impl ResponseOwnershipError {
    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(self) -> ResponseOwnershipIssue {
        self.issue
    }
}

impl core::fmt::Display for ResponseOwnershipError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "response ownership rejected: {:?}", self.issue)
    }
}

impl core::error::Error for ResponseOwnershipError {}

/// Applies one generation-scoped ownership transition.
///
/// The host must close or discard every returned `Poisoned` connection.
///
/// # Errors
///
/// Returns [`ResponseOwnershipError`] for unsafe identities, concurrent claims,
/// stale generations, poisoned reuse, or an invalid transition.
///
/// # Panics
///
/// Panics if a successful claim does not preserve both validated identities.
// r[impl animus.ownership.scope]
// r[impl animus.ownership.poison]
// r[impl animus.ownership.release]
pub fn reduce_response_ownership(
    state: &ResponseConnectionOwnership,
    event: ResponseOwnershipEvent<'_>,
) -> Result<ResponseConnectionOwnership, ResponseOwnershipError> {
    match (state, event) {
        (
            ResponseConnectionOwnership::Vacant,
            ResponseOwnershipEvent::Claim {
                connection_id,
                generation_id,
            },
        ) => {
            validate_label(connection_id)?;
            validate_label(generation_id)?;
            let next = ResponseConnectionOwnership::Owned {
                connection_id: connection_id.to_string(),
                generation_id: generation_id.to_string(),
            };
            assert!(matches!(next, ResponseConnectionOwnership::Owned { .. }));
            assert_ne!(connection_id, "");
            Ok(next)
        }
        (ResponseConnectionOwnership::Owned { .. }, ResponseOwnershipEvent::Claim { .. }) => {
            Err(error(ResponseOwnershipIssue::ConcurrentOwner))
        }
        (
            ResponseConnectionOwnership::Owned { generation_id, .. },
            ResponseOwnershipEvent::Frame {
                generation_id: observed,
            },
        ) if generation_id == observed => Ok(state.clone()),
        (
            ResponseConnectionOwnership::Owned { generation_id, .. },
            ResponseOwnershipEvent::Terminal {
                generation_id: observed,
            },
        ) if generation_id == observed => Ok(ResponseConnectionOwnership::Vacant),
        (
            ResponseConnectionOwnership::Owned { .. },
            ResponseOwnershipEvent::Frame { .. } | ResponseOwnershipEvent::Terminal { .. },
        ) => Err(error(ResponseOwnershipIssue::GenerationMismatch)),
        (
            ResponseConnectionOwnership::Owned { .. },
            ResponseOwnershipEvent::Interrupted
            | ResponseOwnershipEvent::Malformed
            | ResponseOwnershipEvent::EarlyReturn,
        ) => Ok(ResponseConnectionOwnership::Poisoned),
        (ResponseConnectionOwnership::Poisoned, _) => {
            Err(error(ResponseOwnershipIssue::PoisonedConnection))
        }
        (ResponseConnectionOwnership::Vacant, _) => {
            Err(error(ResponseOwnershipIssue::InvalidTransition))
        }
    }
}

fn validate_label(value: &str) -> Result<(), ResponseOwnershipError> {
    let is_safe = !value.is_empty()
        && value.len() <= MAX_LABEL_BYTES
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
        });
    if is_safe {
        Ok(())
    } else {
        Err(error(ResponseOwnershipIssue::InvalidIdentity))
    }
}

const fn error(issue: ResponseOwnershipIssue) -> ResponseOwnershipError {
    ResponseOwnershipError { issue }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_generation_claims_and_terminal_completion_releases() {
        // r[verify animus.ownership.scope.claim]
        // r[verify animus.ownership.release.terminal]
        // r[verify animus.ownership.verification]
        // r[verify animus.ownership.verification.matrix]
        let owned = reduce_response_ownership(
            &ResponseConnectionOwnership::Vacant,
            ResponseOwnershipEvent::Claim {
                connection_id: "connection-1",
                generation_id: "generation-1",
            },
        )
        .expect("claim");
        let framed = reduce_response_ownership(
            &owned,
            ResponseOwnershipEvent::Frame {
                generation_id: "generation-1",
            },
        )
        .expect("owned frame");
        let released = reduce_response_ownership(
            &framed,
            ResponseOwnershipEvent::Terminal {
                generation_id: "generation-1",
            },
        )
        .expect("terminal release");

        assert!(matches!(owned, ResponseConnectionOwnership::Owned { .. }));
        assert_eq!(framed, owned);
        assert_eq!(released, ResponseConnectionOwnership::Vacant);
    }

    #[test]
    fn concurrent_stale_interrupted_and_early_reuse_fail_closed() {
        // r[verify animus.ownership.scope.concurrent]
        // r[verify animus.ownership.poison.interrupted]
        // r[verify animus.ownership.poison.stale_frame]
        // r[verify animus.ownership.release.early]
        let owned = reduce_response_ownership(
            &ResponseConnectionOwnership::Vacant,
            ResponseOwnershipEvent::Claim {
                connection_id: "connection-1",
                generation_id: "generation-1",
            },
        )
        .expect("claim");
        let concurrent = reduce_response_ownership(
            &owned,
            ResponseOwnershipEvent::Claim {
                connection_id: "connection-1",
                generation_id: "generation-2",
            },
        )
        .expect_err("concurrent owner");
        let stale = reduce_response_ownership(
            &owned,
            ResponseOwnershipEvent::Frame {
                generation_id: "generation-old",
            },
        )
        .expect_err("stale frame");
        let poisoned = reduce_response_ownership(&owned, ResponseOwnershipEvent::Interrupted)
            .expect("interruption poisons");
        let reuse = reduce_response_ownership(
            &poisoned,
            ResponseOwnershipEvent::Claim {
                connection_id: "connection-1",
                generation_id: "generation-2",
            },
        )
        .expect_err("poisoned reuse");
        let early = reduce_response_ownership(&owned, ResponseOwnershipEvent::EarlyReturn)
            .expect("early return poisons");

        assert_eq!(concurrent.issue(), ResponseOwnershipIssue::ConcurrentOwner);
        assert_eq!(stale.issue(), ResponseOwnershipIssue::GenerationMismatch);
        assert_eq!(poisoned, ResponseConnectionOwnership::Poisoned);
        assert_eq!(reuse.issue(), ResponseOwnershipIssue::PoisonedConnection);
        assert_eq!(early, ResponseConnectionOwnership::Poisoned);
    }
}
