//! Command execution: load, apply one pure transition, commit with the
//! version read, and reapply on a version conflict.

type Lifecycle = lattice_core::agent_revision::state::Lifecycle;
type Refusal = lattice_core::agent_revision::Refusal;
type Applied<T> = lattice_core::agent_revision::ops::Applied<T>;
type Kind = lattice_core::agent_revision::state::SelectionKind;
type Value = serde_json::Value;

pub const OUTCOME_SCHEMA: &str = "lattice.agent-revision-outcome.v1";
/// Commits attempted before a contended lifecycle is reported.
const MAX_COMMIT_ATTEMPTS: u32 = 4;

/// Whether a missing lifecycle may be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seed {
    Create,
    Existing,
}

/// Which command printed a document, and for which definition.
#[derive(Debug, Clone, Copy)]
pub(super) struct Header<'a> {
    pub command: &'static str,
    pub definition: &'a str,
}

/// The definition and revision one preparation names.
#[derive(Debug, Clone, Copy)]
struct Subject<'a> {
    definition: &'a str,
    revision: &'a str,
}

/// Apply `apply` to `definition`'s stored lifecycle and commit any change
/// with the version read, reloading and reapplying the pure transition after
/// a concurrent commit, at most four times. A refusal or an unchanged
/// lifecycle writes nothing.
pub fn transact<T>(
    store: &crate::store::Store,
    definition: &str,
    seed: Seed,
    apply: impl Fn(&Lifecycle) -> Result<Applied<T>, Refusal>,
) -> Result<Result<T, Refusal>, String> {
    for _ in 0..MAX_COMMIT_ATTEMPTS {
        let (version, current) = match (store.load_lifecycle(definition).map_err(|error| error.to_string())?, seed) {
            (Some(stored), _) => stored,
            (None, Seed::Create) => (0, Lifecycle::new(definition)),
            (None, Seed::Existing) => return Err(format!("definition {definition} is not lifecycle-managed")),
        };
        debug_assert_eq!(current.definition, definition);
        let applied = match apply(&current) {
            Ok(applied) => applied,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if applied.lifecycle == current {
            return Ok(Ok(applied.outcome));
        }
        match store.commit_lifecycle(version, &applied.lifecycle) {
            Ok(committed) => {
                debug_assert!(committed > version);
                return Ok(Ok(applied.outcome));
            }
            Err(crate::store::StoreError::Conflict(_)) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("agent revision lifecycle stayed contended; retry the same request".to_owned())
}

/// One printed document: an outcome or a refusal, never both.
pub(super) fn document<T: serde::Serialize>(header: Header<'_>, result: Result<T, Refusal>) -> Value {
    let (key, value) = match result {
        Ok(outcome) => ("outcome", serde_json::to_value(outcome)),
        Err(refusal) => ("refusal", serde_json::to_value(refusal)),
    };
    let mut document = serde_json::json!({
        "schema": OUTCOME_SCHEMA,
        "command": header.command,
        "definition": header.definition,
    });
    document[key] = value.unwrap_or(Value::Null);
    document
}

pub(super) fn open(path: &std::path::Path) -> Result<crate::store::Store, String> {
    crate::store::Store::new(path).map_err(|error| error.to_string())
}

pub(super) fn execute(command: &super::Command, ports: super::Ports<'_>) -> Result<Value, String> {
    let document = match command {
        super::Command::Resolve { request } => resolve(request),
        super::Command::Desire { request, store } => desire(request, &open(store)?),
        super::Command::Prepare {
            store,
            definition,
            revision,
        } => prepare(&open(store)?, Subject { definition, revision }, ports.preparation),
        super::Command::Promote(args) => select(args, Kind::Promotion, ports.qualification),
        super::Command::Rollback(args) => select(args, Kind::Rollback, ports.qualification),
        super::Command::Revoke(args) => super::decisions::revoke(args),
        super::Command::AdmitSession {
            request,
            store,
            session,
        } => admit(request, &open(store)?, session),
        super::Command::Cleanup { store, definition } => cleanup(&open(store)?, definition, ports.preparation),
        super::Command::Inspect { store, definition } => inspect(store, definition),
    }?;
    debug_assert_eq!(document["schema"], OUTCOME_SCHEMA);
    debug_assert!(document.get("command").is_some());
    Ok(document)
}

/// Named attachment files answer through the attachment port; without them
/// the injected qualification port answers.
// r[impl lattice.agent_revisions.attachments.cli]
fn select(args: &super::SelectArgs, kind: Kind, port: &mut dyn super::QualificationPort) -> Result<Value, String> {
    if args.qualification_attachments.is_empty() {
        return super::decisions::select(args, kind, port);
    }
    let mut attached = super::AttachmentQualification::load(&args.qualification_attachments)?;
    super::decisions::select(args, kind, &mut attached)
}

fn resolve(request: &std::path::Path) -> Result<Value, String> {
    let resolved = super::bundle::resolve(&super::bundle::load(request)?)?;
    let definition = resolved.definition.identity.clone();
    Ok(document(
        Header {
            command: "resolve",
            definition: &definition,
        },
        Ok(resolved),
    ))
}

fn desire(request: &std::path::Path, store: &crate::store::Store) -> Result<Value, String> {
    let resolved = super::bundle::resolve(&super::bundle::load(request)?)?;
    let definition = resolved.definition.identity.as_str();
    let result = transact(store, definition, Seed::Create, |lifecycle| {
        lattice_core::agent_revision::ops::prepare::desire(lifecycle, &resolved.bundle)
    })?;
    Ok(document(
        Header {
            command: "desire",
            definition,
        },
        result,
    ))
}

fn prepare(
    store: &crate::store::Store,
    subject: Subject<'_>,
    port: &mut dyn super::PreparationPort,
) -> Result<Value, String> {
    let header = Header {
        command: "prepare",
        definition: subject.definition,
    };
    let begin = transact(store, subject.definition, Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::prepare::begin(lifecycle, subject.revision)
    })?;
    let started = match &begin {
        Ok(
            lattice_core::agent_revision::ops::prepare::Begin::Started { ticket }
            | lattice_core::agent_revision::ops::prepare::Begin::Reissued { ticket },
        ) => Some(ticket.clone()),
        Ok(lattice_core::agent_revision::ops::prepare::Begin::AlreadyPrepared { .. }) => {
            return Ok(document(header, begin));
        }
        Err(_) => None,
    };
    let (_, lifecycle) = store
        .load_lifecycle(subject.definition)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "agent revision lifecycle disappeared".to_owned())?;
    // A refused start may leave a pending or unknown attempt bound to an older
    // generation; reissue its ticket so the attempt settles instead of
    // blocking preparation for good.
    let reissued = || lattice_core::agent_revision::ops::prepare::reissue(&lifecycle, subject.revision).ok();
    let Some(ticket) = started.or_else(reissued) else {
        return Ok(document(header, begin));
    };
    let candidate = lifecycle
        .candidates
        .get(subject.revision)
        .ok_or_else(|| "prepared revision disappeared".to_owned())?;
    let observation = port.prepare(&ticket, &candidate.bundle);
    let recorded = transact(store, subject.definition, Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::prepare::record(lifecycle, &ticket, &observation)
    })?;
    let mut output = document(header, recorded);
    output["ticket"] = serde_json::to_value(&ticket).map_err(|error| error.to_string())?;
    output["observation"] = serde_json::to_value(&observation).map_err(|error| error.to_string())?;
    debug_assert_eq!(ticket.revision, subject.revision);
    debug_assert_eq!(ticket.definition, subject.definition);
    Ok(output)
}

fn admit(request: &std::path::Path, store: &crate::store::Store, session: &str) -> Result<Value, String> {
    let request = super::bundle::load(request)?;
    let decision = super::seam::admit(store, &request, session)?;
    let definition = super::bundle::resolve(&request)?.definition.identity;
    let result = match decision {
        super::seam::Decision::Refused { refusal } => Err(refusal),
        admitted @ (super::seam::Decision::Unmanaged | super::seam::Decision::Admitted { .. }) => Ok(admitted),
    };
    Ok(document(
        Header {
            command: "admit-session",
            definition: &definition,
        },
        result,
    ))
}

/// Release every unretained revision's artifacts through the port.
pub(super) fn cleanup(
    store: &crate::store::Store,
    definition: &str,
    port: &mut dyn super::PreparationPort,
) -> Result<Value, String> {
    let header = Header {
        command: "cleanup",
        definition,
    };
    let (_, lifecycle) = store
        .load_lifecycle(definition)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("definition {definition} is not lifecycle-managed"))?;
    let sweeps = lattice_core::agent_revision::ops::retention::plan(&lifecycle);
    debug_assert!(sweeps.iter().all(|sweep| sweep.definition == definition));
    let mut results = Vec::with_capacity(sweeps.len());
    for sweep in &sweeps {
        let observation = port.cleanup(sweep);
        let recorded = transact(store, definition, Seed::Existing, |lifecycle| {
            lattice_core::agent_revision::ops::retention::record(lifecycle, &sweep.revision, observation)
        })?;
        results.push(document(header, recorded));
    }
    debug_assert_eq!(results.len(), sweeps.len());
    Ok(document(header, Ok(results)))
}

fn inspect(path: &std::path::Path, definition: &str) -> Result<Value, String> {
    let header = Header {
        command: "inspect",
        definition,
    };
    let stored = if path.exists() {
        open(path)?.load_lifecycle(definition).map_err(|error| error.to_string())?
    } else {
        None
    };
    let Some((version, lifecycle)) = stored else {
        return Ok(document(header, Ok(serde_json::json!({ "managed": false }))));
    };
    debug_assert!(version > 0);
    debug_assert_eq!(lifecycle.definition, definition);
    let retention = lifecycle
        .candidates
        .keys()
        .map(|revision| {
            let reasons = lattice_core::agent_revision::ops::retention::retained_by(&lifecycle, revision);
            (revision.clone(), reasons)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let outcome = serde_json::json!({
        "managed": true,
        "version": version,
        "slots": lattice_core::agent_revision::projection::slots(&lifecycle),
        "retention": retention,
        "cleanup_plan": lattice_core::agent_revision::ops::retention::plan(&lifecycle),
        "lifecycle": lifecycle,
    });
    Ok(document(header, Ok(outcome)))
}
