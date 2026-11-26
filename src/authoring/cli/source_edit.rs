//! Closed machine wire for source inspection, exact viewing and draft-only saves.
use std::io::Read as _;
use std::io::Write as _;

use super::super::AssociationState;
use super::super::ContentIdentity;
use super::super::SourceDraft;
use super::super::source_edit::ExpectedDraft;
use super::super::source_edit::NativeReceipt;
use super::super::source_edit::Rejection;
use super::super::source_edit::draft_revision;
use crate::store::Store;

const SCHEMA: &str = "lattice.source-edit.v1";
const MAX_MEMBER: usize = crate::workflow_template::MAX_FILE_BYTES;
// Four JSON bytes per member byte (e.g. `255,`), plus metadata/framing.
const MAX_REQUEST: u64 = 4_210_688;

#[derive(serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Inspect {
        schema: String,
        workflow: String,
    },
    View {
        schema: String,
        workflow: String,
        expected_head: ContentIdentity,
        draft_id: String,
        member: String,
    },
    Save {
        schema: String,
        workflow: String,
        expected_head: ContentIdentity,
        draft_id: String,
        member: String,
        #[serde(deserialize_with = "ExpectedDraft::required")]
        expected_draft: ExpectedDraft,
        bytes: Vec<u8>,
    },
}

#[derive(serde::Serialize)]
struct Envelope {
    schema: &'static str,
    #[serde(flatten)]
    outcome: Outcome,
}

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Outcome {
    Inspected {
        workflow: String,
        head: ContentIdentity,
        source: Option<SourceSummary>,
    },
    Viewed {
        workflow: String,
        head: ContentIdentity,
        draft_id: String,
        draft_revision: Option<ContentIdentity>,
        source_revision: ContentIdentity,
        member: String,
        bytes: Vec<u8>,
    },
    Saved {
        workflow: String,
        head: ContentIdentity,
        draft_id: String,
        draft_revision: ContentIdentity,
        source_revision: ContentIdentity,
        member: String,
        receipt: Box<NativeReceipt>,
    },
    Rejected {
        code: Rejection,
    },
}

#[derive(serde::Serialize)]
struct SourceSummary {
    source_revision: ContentIdentity,
    state: AssociationState,
    entry: String,
    members: Vec<String>,
}

pub(super) fn run(path: &std::path::Path) -> Result<(), String> {
    let mut stdout = std::io::stdout().lock();
    let reply = read_request(std::io::stdin().lock()).and_then(|request| dispatch(path, request));
    let rejected = reply.is_err();
    let envelope = Envelope {
        schema: SCHEMA,
        outcome: reply.unwrap_or_else(|code| Outcome::Rejected { code }),
    };
    serde_json::to_writer(&mut stdout, &envelope).map_err(|_| "source edit reply write failed")?;
    stdout
        .write_all(b"\n")
        .and_then(|()| stdout.flush())
        .map_err(|_| "source edit reply write failed")?;
    if rejected {
        return Err("source edit rejected; see JSON status".into());
    }
    Ok(())
}

fn read_request(reader: impl std::io::Read) -> Result<Request, Rejection> {
    let mut bytes = Vec::new();
    reader.take(MAX_REQUEST + 1).read_to_end(&mut bytes).map_err(|_| Rejection::InvalidRequest)?;
    if u64::try_from(bytes.len()).map_err(|_| Rejection::RequestTooLarge)? > MAX_REQUEST {
        return Err(Rejection::RequestTooLarge);
    }
    serde_json::from_slice(&bytes).map_err(|_| Rejection::InvalidRequest)
}

fn dispatch(path: &std::path::Path, request: Request) -> Result<Outcome, Rejection> {
    let schema = match &request {
        Request::Inspect { schema, .. } | Request::View { schema, .. } | Request::Save { schema, .. } => schema,
    };
    if schema != SCHEMA {
        return Err(Rejection::UnsupportedSchema);
    }
    match request {
        Request::Inspect { workflow, .. } => inspect(path, workflow),
        Request::View {
            workflow,
            expected_head,
            draft_id,
            member,
            ..
        } => {
            let (draft, draft_revision) = open_edit(&workflow, &expected_head, &draft_id, &member, || {
                Store::read_source_edit_snapshot(path, &workflow, &expected_head, &draft_id)
            })?;
            let source_revision = draft.program.identity().map_err(|_| Rejection::StoreError)?;
            let mut files = draft.program.files;
            let bytes = files.remove(&member).ok_or(Rejection::MemberNotFound)?;
            Ok(Outcome::Viewed {
                workflow,
                head: expected_head,
                draft_id,
                draft_revision,
                source_revision,
                member,
                bytes,
            })
        }
        Request::Save {
            workflow,
            expected_head,
            draft_id,
            member,
            expected_draft,
            bytes,
            ..
        } => {
            if bytes.len() > MAX_MEMBER {
                return Err(Rejection::RequestTooLarge);
            }
            let store = Store::open_source_edit(path).map_err(|_| Rejection::StoreError)?;
            let (mut draft, observed) = open_edit(&workflow, &expected_head, &draft_id, &member, || {
                store.source_edit_snapshot(&workflow, &expected_head, &draft_id)
            })?;
            if observed != expected_draft.0 {
                return Err(Rejection::DraftConflict);
            }
            *draft.program.files.get_mut(&member).ok_or(Rejection::MemberNotFound)? = bytes;
            let (draft_revision, source_revision, receipt) = store.save_source_edit(draft, &expected_draft)?;
            Ok(Outcome::Saved {
                workflow,
                head: expected_head,
                draft_id,
                draft_revision,
                source_revision,
                member,
                receipt: Box::new(receipt),
            })
        }
    }
}

// r[impl lattice.source_edit.view]
fn inspect(path: &std::path::Path, workflow: String) -> Result<Outcome, Rejection> {
    if !SourceDraft::valid_workflow_name(&workflow) {
        return Err(Rejection::InvalidRequest);
    }
    let stored = Store::load_workflow_read_only(path, &workflow)
        .map_err(|_| Rejection::StoreError)?
        .ok_or(Rejection::WorkflowNotFound)?;
    let head = stored.catalog_revision.ok_or(Rejection::StoreError)?;
    let source = stored.authoring.map(|source| SourceSummary {
        source_revision: source.source_revision,
        state: source.state,
        entry: source.program.entry.clone(),
        members: source.program.files.keys().cloned().collect(),
    });
    Ok(Outcome::Inspected { workflow, head, source })
}

fn open_edit(
    workflow: &str,
    head: &ContentIdentity,
    draft_id: &str,
    member: &str,
    snapshot: impl FnOnce() -> Result<(crate::engine::Workflow, Option<SourceDraft>), Rejection>,
) -> Result<(SourceDraft, Option<ContentIdentity>), Rejection> {
    if !SourceDraft::valid_workflow_name(workflow)
        || !SourceDraft::valid_id(draft_id)
        || !super::super::safe_path(member)
    {
        return Err(Rejection::InvalidRequest);
    }
    let (stored, current) = snapshot()?;
    let revision = current.as_ref().map(draft_revision).transpose().map_err(|_| Rejection::StoreError)?;
    let draft = match current {
        Some(draft) => draft,
        None => {
            let source = stored.authoring.ok_or(Rejection::SourceUnavailable)?;
            if source.program.identity().map_err(|_| Rejection::StoreError)? != source.source_revision {
                return Err(Rejection::StoreError);
            }
            SourceDraft {
                id: draft_id.into(),
                workflow_name: workflow.into(),
                expected_revision: Some(head.clone()),
                program: std::sync::Arc::unwrap_or_clone(source.program),
                diagnostic: None,
                preview: None,
            }
        }
    };
    draft.validate().map_err(|_| Rejection::StoreError)?;
    if !draft.program.files.contains_key(member) {
        return Err(Rejection::MemberNotFound);
    }
    Ok((draft, revision))
}
