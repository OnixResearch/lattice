//! The agent-revision session seam of one attempt (Lattice #25).
//!
//! With `--revision-store`, a run admits the attempt's session, whose
//! identifier is the envelope identity, through `seam::admit` before anything
//! is written. A refused session starts nothing. The admission mode, and the
//! pin of a managed session, are recorded in the intent; `seam::effect` runs
//! with that mode immediately before the worker would spawn, and `seam::close`
//! (no checkpoints: the worker is single-shot) after the terminal result or
//! when the attempt ends before dispatch. Recovery, inspection and export read
//! the recorded mode back. Without a store the attempt is unmanaged and
//! behaves exactly as before. The store is opened only around each seam call,
//! so operator commands such as `revoke` are never locked out during a run.
//!
//! Every seam call keys on `seam::subject`: the workflow-agent definition and
//! revision the request names, else the worker's own. A request naming a
//! workflow-agent revision is never unmanaged, so it needs a store.

// r[impl lattice.agent_revisions.seam.scenario.attempt]
// r[impl lattice.implementation_attempt.consumer.scenario.admission_mode]

pub(in super::super) const BLOCK_FILE: &str = "revision-block.json";
const BLOCK_SCHEMA: &str = "lattice.implementation-attempt-revision-block.v1";
const MAX_BLOCK_BYTES: usize = 4_096;

type Admission = crate::agent_revision::seam::Admission;
type Recorded = super::intent::Recorded;
type Request = super::super::Request;

/// The session identifier of one attempt: its envelope identity.
pub(in super::super) fn identifier(request: &Request) -> String {
    request.envelope.identity()
}

/// The optional `--revision-store` of one command.
#[derive(Debug, Clone, Copy)]
pub(in super::super) struct Store<'a>(pub Option<&'a std::path::Path>);

impl Store<'_> {
    /// Open an existing store; a named store that does not exist is an error,
    /// never silently unmanaged.
    fn open(path: &std::path::Path) -> Result<crate::store::Store, String> {
        if !path.is_file() {
            return Err("agent revision store is unavailable".into());
        }
        crate::store::Store::new(path).map_err(|error| error.to_string())
    }

    /// The store a recorded managed session needs. A managed session is never
    /// reconciled, inspected or exported without its lifecycle.
    fn for_recorded(self, recorded: &Recorded) -> Result<Option<crate::store::Store>, String> {
        match (recorded.admission, self.0) {
            (Admission::Unmanaged, _) => Ok(None),
            (Admission::Managed, Some(path)) => Self::open(path).map(Some),
            (Admission::Managed, None) => {
                Err("attempt was admitted lifecycle-managed; this command requires its --revision-store".into())
            }
        }
    }

    /// Refuse to read a managed attempt without its store.
    pub(in super::super) fn require(self, recorded: &Recorded) -> Result<(), String> {
        if recorded.admission == Admission::Managed && self.0.is_none() {
            return Err("attempt was admitted lifecycle-managed; this command requires its --revision-store".into());
        }
        Ok(())
    }

    /// The worker definition's lifecycle for profile facts and the session
    /// subject's lifecycle for its pin, both `None` without a store. They
    /// are one read unless the request names a workflow-agent revision.
    pub(in super::super) fn lifecycles(self, request: &Request) -> Result<Lifecycles, String> {
        let Some(path) = self.0 else {
            return Ok(Lifecycles {
                profile: None,
                workflow: None,
            });
        };
        let store = Self::open(path)?;
        let load = |definition: &str| {
            store
                .load_lifecycle(definition)
                .map(|stored| stored.map(|(_, lifecycle)| lifecycle))
                .map_err(|error| error.to_string())
        };
        let worker = crate::agent_revision::bundle::resolve(request)?.definition.identity;
        let subject = crate::agent_revision::seam::subject(request)?.definition;
        let workflow = (subject != worker).then(|| load(&subject)).transpose()?;
        Ok(Lifecycles {
            profile: load(&worker)?,
            workflow,
        })
    }
}

type Lifecycle = lattice_core::agent_revision::state::Lifecycle;

/// The lifecycles one inspection reads.
pub(in super::super) struct Lifecycles {
    /// The worker definition's.
    profile: Option<Lifecycle>,
    /// The workflow-agent definition's, read only when the session subject
    /// is not the worker definition.
    workflow: Option<Option<Lifecycle>>,
}

impl Lifecycles {
    /// The worker definition's lifecycle, for the worker profile facts.
    pub(in super::super) fn profile(&self) -> Option<&Lifecycle> {
        self.profile.as_ref()
    }

    /// The session subject's lifecycle, for the attempt's pin.
    pub(in super::super) fn session(&self) -> Option<&Lifecycle> {
        match &self.workflow {
            Some(workflow) => workflow.as_ref(),
            None => self.profile.as_ref(),
        }
    }
}

/// One run's admission before its intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct Admitted {
    /// `None` without a store: the legacy intent, unmanaged by construction.
    pub recorded: Option<Recorded>,
    /// This run pinned the session; another run did not pin it before.
    is_fresh: bool,
}

/// Admit the attempt's session before anything is written. A refusal prints
/// the typed refusal and starts nothing.
pub(in super::super) fn admit(store: Store<'_>, request: &Request) -> Result<Admitted, String> {
    let Some(path) = store.0 else {
        if request.workflow_agent.is_some() {
            return Err("a workflow-agent attempt requires its --revision-store; nothing was dispatched".into());
        }
        return Ok(Admitted {
            recorded: None,
            is_fresh: false,
        });
    };
    let decision = crate::agent_revision::seam::admit(&Store::open(path)?, request, &identifier(request))?;
    let (pin, is_fresh) = match decision {
        crate::agent_revision::seam::Decision::Unmanaged => (None, false),
        crate::agent_revision::seam::Decision::Admitted { admitted } => match admitted {
            lattice_core::agent_revision::ops::sessions::Admitted::Pinned { pin, pin_identity } => {
                (Some(recorded_pin(pin, pin_identity)), true)
            }
            lattice_core::agent_revision::ops::sessions::Admitted::AlreadyPinned { pin, pin_identity } => {
                (Some(recorded_pin(pin, pin_identity)), false)
            }
        },
        crate::agent_revision::seam::Decision::Refused { refusal } => {
            return Err(super::super::refusal::refuse_revision(
                &request.envelope,
                super::super::refusal::Reason::RevisionNotAdmitted,
                &refusal,
            ));
        }
    };
    // Only a session this run pinned is fresh, and the lifecycle pins bare
    // profile revisions only.
    debug_assert!(!is_fresh || pin.is_some());
    debug_assert!(pin.as_ref().is_none_or(|pin| lattice_core::agent_revision::is_bare_digest(&pin.revision)));
    let admission = if pin.is_some() {
        Admission::Managed
    } else {
        Admission::Unmanaged
    };
    Ok(Admitted {
        recorded: Some(Recorded { admission, pin }),
        is_fresh,
    })
}

fn recorded_pin(pin: lattice_core::agent_revision::state::Pin, pin_identity: String) -> super::intent::Pin {
    super::intent::Pin {
        revision: pin.revision,
        qualification_ref: pin.qualification_ref,
        rollout_generation: pin.rollout_generation,
        pin_identity,
    }
}

/// Close a session this run pinned when the attempt ends before its intent,
/// so a refused fence or binding never leaves a live pin behind.
pub(in super::super) fn abandon(store: Store<'_>, request: &Request, admitted: &Admitted) -> Result<(), String> {
    match &admitted.recorded {
        Some(recorded) if admitted.is_fresh => close(store, request, recorded),
        Some(_) | None => Ok(()),
    }
}

/// `seam::effect` immediately before the worker spawns. A blocked managed
/// session is recorded once and refused before dispatch; the caller closes it.
pub(in super::super) fn permit(
    store: Store<'_>,
    request: &Request,
    recorded: &Recorded,
    directory: &std::path::Path,
) -> Result<(), String> {
    let Some(refusal) = recheck(store, request, recorded)? else {
        return Ok(());
    };
    // Only a managed session has a store to block it.
    debug_assert_eq!(recorded.admission, Admission::Managed);
    let block = serde_json::json!({
        "schema": BLOCK_SCHEMA,
        "session": identifier(request),
        "refusal": refusal,
    });
    super::super::artifacts::write_once(
        &directory.join(BLOCK_FILE),
        &serde_json::to_vec(&block).map_err(|error| error.to_string())?,
    )?;
    // Recovery finds the block before it can find any refusal.
    debug_assert!(directory.join(BLOCK_FILE).is_file());
    Err(super::super::refusal::refuse_revision(
        &request.envelope,
        super::super::refusal::Reason::RevisionBlocked,
        &refusal,
    ))
}

/// The `seam::effect` check of [`permit`] without writing anything: the
/// refusal of a blocked managed session, or `None` while it is permitted or
/// unmanaged. A repair step after the first uses it and retains the refusal
/// in its post-dispatch disposition rather than as a pre-dispatch block.
pub(in super::super) fn recheck(
    store: Store<'_>,
    request: &Request,
    recorded: &Recorded,
) -> Result<Option<lattice_core::agent_revision::Refusal>, String> {
    let Some(opened) = store.for_recorded(recorded)? else {
        return Ok(None);
    };
    let effect = crate::agent_revision::seam::effect(&opened, request, &identifier(request), recorded.admission)?;
    drop(opened);
    match effect {
        crate::agent_revision::seam::Effect::Unmanaged | crate::agent_revision::seam::Effect::Permitted { .. } => {
            Ok(None)
        }
        crate::agent_revision::seam::Effect::Blocked { refusal } => Ok(Some(refusal)),
    }
}

/// `seam::close` with no checkpoints after the attempt ends. A managed
/// session already closed and forgotten answers `unknown_session`, which is
/// settled; any other refusal is an error.
pub(in super::super) fn close(store: Store<'_>, request: &Request, recorded: &Recorded) -> Result<(), String> {
    let Some(opened) = store.for_recorded(recorded)? else {
        return Ok(());
    };
    match crate::agent_revision::seam::close(&opened, request, &identifier(request), recorded.admission, &[])? {
        crate::agent_revision::seam::Closure::Unmanaged
        | crate::agent_revision::seam::Closure::Closed { .. }
        | crate::agent_revision::seam::Closure::Refused {
            refusal: lattice_core::agent_revision::Refusal::UnknownSession,
        } => Ok(()),
        crate::agent_revision::seam::Closure::Refused { refusal } => {
            Err(format!("agent revision session close was refused: {}", serde_json::json!(refusal)))
        }
    }
}

/// Whether a closed attempt's retained candidate may still be exported.
pub(in super::super) fn exportable(store: Store<'_>, request: &Request, recorded: &Recorded) -> Result<(), String> {
    let Some(opened) = store.for_recorded(recorded)? else {
        return Ok(());
    };
    // Only a managed session has a store to check.
    debug_assert_eq!(recorded.admission, Admission::Managed);
    let Some(pin) = &recorded.pin else {
        return Err("a managed intent has no recorded pin".into());
    };
    // A recorded pin names a bare profile revision, as admission recorded it.
    debug_assert!(lattice_core::agent_revision::is_bare_digest(&pin.revision));
    let permit = lattice_core::agent_revision::ops::sessions::Permit {
        revision: pin.revision.clone(),
        qualification_ref: pin.qualification_ref.clone(),
        rollout_generation: pin.rollout_generation,
    };
    match crate::agent_revision::seam::retained(&opened, request, &identifier(request), recorded.admission, permit)? {
        crate::agent_revision::seam::Effect::Unmanaged | crate::agent_revision::seam::Effect::Permitted { .. } => {
            Ok(())
        }
        crate::agent_revision::seam::Effect::Blocked { refusal } => Err(super::super::refusal::refuse_revision(
            &request.envelope,
            super::super::refusal::Reason::ExportRevoked,
            &refusal,
        )),
    }
}

/// Whether recovery finds a retained revision block, read back verbatim.
pub(in super::super) fn retained_block(directory: &std::path::Path) -> Result<Option<serde_json::Value>, String> {
    let Some(bytes) = super::super::artifacts::read_optional(&directory.join(BLOCK_FILE), MAX_BLOCK_BYTES)? else {
        return Ok(None);
    };
    let block: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "retained revision block is malformed")?;
    if block.get("schema").and_then(serde_json::Value::as_str) != Some(BLOCK_SCHEMA) {
        return Err("retained revision block is malformed".into());
    }
    Ok(Some(block))
}
