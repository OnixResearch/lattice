//! The session-admission seam for lifecycle-managed definitions.
//!
//! An embedder (the implementation-attempt shell after integration) calls
//! [`admit`] before writing its intent and [`effect`] before each protected
//! effect. A definition with no stored lifecycle is `unmanaged`: the caller
//! proceeds exactly as it does today, and nothing is written. A managed
//! definition admits only the revision selected for new sessions, pins it,
//! and blocks effects after any relevant revocation.

// r[impl lattice.agent_revisions.seam]
// r[impl lattice.agent_revisions.seam.scenario.unmanaged]

type Refusal = lattice_core::agent_revision::Refusal;

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

/// Whether `session` of `request`'s definition may perform its next
/// protected effect.
pub fn effect(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
) -> Result<Effect, String> {
    let definition = super::bundle::resolve(request)?.definition.identity;
    let Some((_, lifecycle)) = store.load_lifecycle(&definition).map_err(|error| error.to_string())? else {
        return Ok(Effect::Unmanaged);
    };
    Ok(match lattice_core::agent_revision::ops::sessions::effect_permitted(&lifecycle, session) {
        Ok(permit) => Effect::Permitted { permit },
        Err(refusal) => Effect::Blocked { refusal },
    })
}

/// Close `session` of `request`'s definition, holding its checkpoints on the
/// pinned revision. An unmanaged definition is left untouched.
pub fn close(
    store: &crate::store::Store,
    request: &crate::implementation_attempt::cli::Request,
    session: &str,
    checkpoints: &[String],
) -> Result<Option<Result<lattice_core::agent_revision::ops::sessions::Closed, Refusal>>, String> {
    let definition = super::bundle::resolve(request)?.definition.identity;
    if store.load_lifecycle(&definition).map_err(|error| error.to_string())?.is_none() {
        return Ok(None);
    }
    let closed = super::commands::transact(store, &definition, super::commands::Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::sessions::close(lifecycle, session, checkpoints)
    })?;
    Ok(Some(closed))
}
