//! Promotion, rollback and revocation commands.

type Kind = lattice_core::agent_revision::state::SelectionKind;
type Refusal = lattice_core::agent_revision::Refusal;
type Value = serde_json::Value;

fn now_ms() -> Result<u64, String> {
    let observed_ms = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    u64::try_from(observed_ms).map_err(|error| error.to_string())
}

pub(super) fn select(
    args: &super::SelectArgs,
    kind: Kind,
    port: &mut dyn super::QualificationPort,
) -> Result<Value, String> {
    let command = match kind {
        Kind::Promotion => "promote",
        Kind::Rollback => "rollback",
    };
    let required_class = serde_json::from_value(Value::String(args.required_class.clone()))
        .map_err(|_| "required class is not an evidence class".to_owned())?;
    let selection = lattice_core::agent_revision::ops::selection::Command {
        kind,
        request_id: args.request_id.clone(),
        revision: args.revision.clone(),
        qualification_ref: args.qualification_ref.clone(),
        policy: lattice_core::agent_revision::state::Policy {
            task_family: args.task_family.clone(),
            required_class,
        },
        expected_rollout_generation: args.expected_generation,
    };
    let store = super::commands::open(&args.store)?;
    let result = select_with(&store, &args.definition, &selection, port, now_ms()?)?;
    debug_assert_eq!(selection.kind, kind);
    debug_assert_eq!(selection.request_id, args.request_id);
    let header = super::commands::Header {
        command,
        definition: &args.definition,
    };
    Ok(super::commands::document(header, result))
}

/// Ask the port about `command`'s exact subject, then apply it.
pub(super) fn select_with(
    store: &crate::store::Store,
    definition: &str,
    command: &lattice_core::agent_revision::ops::selection::Command,
    port: &mut dyn super::QualificationPort,
    now_ms: u64,
) -> Result<Result<lattice_core::agent_revision::ops::selection::Selected, Refusal>, String> {
    let (_, lifecycle) = store
        .load_lifecycle(definition)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("definition {definition} is not lifecycle-managed"))?;
    let query = match lattice_core::agent_revision::ops::selection::query(&lifecycle, command, now_ms) {
        Ok(query) => query,
        Err(refusal) => return Ok(Err(refusal)),
    };
    let verdict = port.answer(&query);
    super::commands::transact(store, definition, super::commands::Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::selection::select(lifecycle, command, &verdict)
    })
}

pub(super) fn revoke(args: &super::RevokeArgs) -> Result<Value, String> {
    let target = match (&args.revision, &args.qualification_ref, &args.session) {
        (Some(revision), None, None) => lattice_core::agent_revision::ops::sessions::Target::Revision(revision.clone()),
        (None, Some(reference), None) => {
            lattice_core::agent_revision::ops::sessions::Target::Qualification(reference.clone())
        }
        (None, None, Some(session)) => lattice_core::agent_revision::ops::sessions::Target::Session(session.clone()),
        _ => return Err("name exactly one revocation target".to_owned()),
    };
    let store = super::commands::open(&args.store)?;
    let result = super::commands::transact(&store, &args.definition, super::commands::Seed::Existing, |lifecycle| {
        lattice_core::agent_revision::ops::sessions::revoke(lifecycle, &target)
    })?;
    let header = super::commands::Header {
        command: "revoke",
        definition: &args.definition,
    };
    Ok(super::commands::document(header, result))
}
