//! The session-admission seam for lifecycle-managed definitions.
//!
//! An embedder (the implementation-attempt shell after integration) calls
//! [`admit`] once before writing its intent, records the session's
//! [`Admission`] durably with the session, and passes that recorded mode to
//! [`effect`] before each protected effect and to [`close`] after the
//! terminal result.
//!
//! - A definition with no stored lifecycle admits `unmanaged`: nothing is written and the caller
//!   proceeds exactly as it does today.
//! - A managed definition admits only the revision selected for new sessions, pins it, and answers
//!   `admitted` (record [`Admission::Managed`]) or `refused` (start nothing).
//! - An [`Admission::Unmanaged`] session stays unmanaged for its whole life, even if its definition
//!   becomes managed later: [`effect`] and [`close`] answer `unmanaged` without reading the store.
//! - An [`Admission::Managed`] session fails closed: an effect is blocked unless the lifecycle
//!   still holds its live, unrevoked pin.

// r[impl lattice.agent_revisions.seam]
// r[impl lattice.agent_revisions.seam.scenario.unmanaged]
// r[impl lattice.agent_revisions.seam.scenario.admission_mode]

type Refusal = lattice_core::agent_revision::Refusal;

/// How a session was admitted. The caller records it durably with the
/// session at admission and passes it back for every later seam call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Admission {
    /// Admitted through [`admit`] with a pin in the definition's lifecycle.
    Managed,
    /// Admitted while the definition had no lifecycle, or without the seam.
    Unmanaged,
}

/// The seam's answer for one session admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    /// No lifecycle exists for the definition; behave exactly as before.
    Unmanaged,
    Admitted {
        admitted: lattice_core::agent_revision::ops::sessions::Admitted,
    },
    /// Refuse the attempt before any binding, intent or worker effect.
    Refused { refusal: Refusal },
}

impl Decision {
    /// The admission mode to record with the session, or `None` when the
    /// session was refused and must not start.
    #[must_use]
    pub const fn admission(&self) -> Option<Admission> {
        match self {
            Self::Unmanaged => Some(Admission::Unmanaged),
            Self::Admitted { .. } => Some(Admission::Managed),
            Self::Refused { .. } => None,
        }
    }
}

/// The seam's answer before one protected effect.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Effect {
    Unmanaged,
    Permitted {
        permit: lattice_core::agent_revision::ops::sessions::Permit,
    },
    Blocked {
        refusal: Refusal,
    },
}

/// The seam's answer for closing one session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Closure {
    Unmanaged,
    Closed {
        closed: lattice_core::agent_revision::ops::sessions::Closed,
    },
    Refused {
        refusal: Refusal,
    },
}

/// Pin `session` to `request`'s revision when its definition is managed.
pub fn admit(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
) -> Result<Decision, String> {
    let resolved = super::bundle::resolve(request)?;
    let definition = resolved.definition.identity.as_str();
    if store.load_lifecycle(definition).map_err(|error| error.to_string())?.is_none() {
        return Ok(Decision::Unmanaged);
    }
    let entry = lattice_core::agent_revision::ops::sessions::Entry {
        session,
        revision: &resolved.revision,
    };
    let result = super::commands::transact(store, definition, super::commands::Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::sessions::admit(lifecycle, entry)
    })?;
    Ok(match result {
        Ok(admitted) => Decision::Admitted { admitted },
        Err(refusal) => Decision::Refused { refusal },
    })
}

/// Whether `session`, admitted as `admission`, may perform its next
/// protected effect. A managed session unknown to the lifecycle is blocked.
pub fn effect(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
    admission: Admission,
) -> Result<Effect, String> {
    if admission == Admission::Unmanaged {
        return Ok(Effect::Unmanaged);
    }
    let definition = super::bundle::resolve(request)?.definition.identity;
    let Some((_, lifecycle)) = store.load_lifecycle(&definition).map_err(|error| error.to_string())? else {
        return Ok(Effect::Blocked {
            refusal: Refusal::UnknownSession,
        });
    };
    Ok(match lattice_core::agent_revision::ops::sessions::effect_permitted(&lifecycle, session) {
        Ok(permit) => Effect::Permitted { permit },
        Err(refusal) => Effect::Blocked { refusal },
    })
}

/// Whether the retained output of `session`, admitted as `admission` under
/// `permit`, may still be published after the session closed. Closing forgets
/// the pin, so the caller passes the one it recorded at admission. An
/// unmanaged session answers `unmanaged` without reading the store; a managed
/// one is blocked once its session, revision or qualification is revoked.
pub fn retained(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
    admission: Admission,
    permit: lattice_core::agent_revision::ops::sessions::Permit,
) -> Result<Effect, String> {
    if admission == Admission::Unmanaged {
        return Ok(Effect::Unmanaged);
    }
    let definition = super::bundle::resolve(request)?.definition.identity;
    let Some((_, lifecycle)) = store.load_lifecycle(&definition).map_err(|error| error.to_string())? else {
        return Ok(Effect::Blocked {
            refusal: Refusal::UnknownSession,
        });
    };
    Ok(
        match lattice_core::agent_revision::ops::sessions::retained_permitted(&lifecycle, session, &permit) {
            Ok(()) => Effect::Permitted { permit },
            Err(refusal) => Effect::Blocked { refusal },
        },
    )
}

/// Close `session`, admitted as `admission`, holding its checkpoints on the
/// pinned revision. An unmanaged session is left untouched.
pub fn close(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
    admission: Admission,
    checkpoints: &[String],
) -> Result<Closure, String> {
    if admission == Admission::Unmanaged {
        return Ok(Closure::Unmanaged);
    }
    let definition = super::bundle::resolve(request)?.definition.identity;
    if store.load_lifecycle(&definition).map_err(|error| error.to_string())?.is_none() {
        return Ok(Closure::Refused {
            refusal: Refusal::UnknownSession,
        });
    }
    let closed = super::commands::transact(store, &definition, super::commands::Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::sessions::close(lifecycle, session, checkpoints)
    })?;
    Ok(match closed {
        Ok(closed) => Closure::Closed { closed },
        Err(refusal) => Closure::Refused { refusal },
    })
}
