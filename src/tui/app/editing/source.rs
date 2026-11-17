//! Source edits use durable drafts and an explicit second admission action.
use super::*;

pub(in crate::tui) fn fields(workflow: &crate::engine::Workflow) -> Vec<crate::tui::authoring::Field> {
    let Some(source) = &workflow.authoring else {
        return Vec::new();
    };
    let mut fields = Vec::with_capacity(source.program.files.len().saturating_add(3));
    fields.extend(member_fields(&source.program));
    fields.extend(parameter_field(&source.program));
    fields.extend(control_fields());
    fields
}

fn member_fields(
    program: &crate::authoring::CapturedProgram,
) -> impl Iterator<Item = crate::tui::authoring::Field> + '_ {
    program.files.iter().filter_map(|(path, bytes)| {
        let text = std::str::from_utf8(bytes).ok()?;
        Some(crate::tui::authoring::field(&format!("source.file/{path}"), &format!("Captured source: {path}"),
            "Edits a separate durable draft; evaluates a preview only. Inspect before explicit source admission. Module import resolution is not captured.", text))
    })
}

fn parameter_field(program: &crate::authoring::CapturedProgram) -> Option<crate::tui::authoring::Field> {
    if program.kind != crate::authoring::ProgramKind::Parameterized {
        return None;
    }
    Some(crate::tui::authoring::field(
        "source.parameters",
        "Source parameter overrides (JSON)",
        "Only source-declared names are admitted. Preview is separate from the last admitted workflow.",
        serde_json::to_string(&program.parameters).unwrap_or_else(|_| "{}".into()),
    ))
}

fn control_fields() -> [crate::tui::authoring::Field; 2] {
    [
        crate::tui::authoring::field(
            "source.admit",
            "Admit exact source preview",
            "Type admit after inspecting the source preview. Durable expected-revision CAS rejects stale drafts.",
            "",
        ),
        crate::tui::authoring::field("source.origins", "Source attribution", crate::authoring::ORIGIN_UNAVAILABLE, ""),
    ]
}

impl State {
    pub(in crate::tui::app) fn refresh_source_fields(&self, state: &mut crate::tui::authoring::State) {
        let Some(store) = &self.store else {
            return;
        };
        let id = format!("tui-{}", blake3::hash(self.workflow.meta.name.as_bytes()).to_hex());
        let Ok(Some(draft)) = store.load_source_draft(&id) else {
            return;
        };
        for field in &mut state.fields {
            if let Some(path) = field.id.strip_prefix("source.file/") {
                if let Some(text) = draft.program.files.get(path).and_then(|bytes| std::str::from_utf8(bytes).ok()) {
                    field.value = text.into();
                }
            } else if field.id == "source.parameters"
                && let Ok(parameters) = serde_json::to_string(&draft.program.parameters)
            {
                field.value = parameters;
            }
        }
    }

    pub(in crate::tui::app) fn apply_source_edit(&mut self, edit: &crate::tui::authoring::Draft) -> Result<(), String> {
        let id = edit.field.id.as_str();
        let value = edit.text.as_str();
        if id == "source.origins" {
            self.output.push_line(crate::authoring::ORIGIN_UNAVAILABLE);
            return Ok(());
        }
        if self.dirty {
            return Err("Save or discard materialized edits before source editing; source compilation never inverses those edits".into());
        }
        let store = self.store.clone().ok_or("No database available for durable source draft")?;
        let draft_id = format!("tui-{}", blake3::hash(self.workflow.meta.name.as_bytes()).to_hex());
        let mut draft = match store.load_source_draft(&draft_id).map_err(|error| error.to_string())? {
            Some(draft) => draft,
            None => crate::authoring::SourceDraft {
                id: draft_id,
                workflow_name: self.workflow.meta.name.clone(),
                expected_revision: self.workflow.catalog_revision.clone(),
                program: self
                    .workflow
                    .authoring
                    .as_ref()
                    .ok_or("Workflow has no captured source")?
                    .program
                    .as_ref()
                    .clone(),
                diagnostic: None,
                preview: None,
            },
        };
        if draft.expected_revision != self.workflow.catalog_revision {
            return Err(
                "Source draft targets an older admitted revision; use a new CLI draft or reopen the matching revision"
                    .into(),
            );
        }
        if id == "source.admit" {
            if value.trim() != "admit" {
                return Err("Type admit to admit the exact successful preview".into());
            }
            self.workflow = store.admit_source_draft(&draft).map_err(|error| error.to_string())?;
            self.workflow_template_draft = None;
            self.clear_workflow_dirty();
            self.sync_graph_state();
            self.output.push_line("Source revision admitted; no workloads started or credentials delivered");
            return Ok(());
        }
        if id == "source.parameters" {
            draft.program.parameters =
                serde_json::from_str(value).map_err(|_| "Parameter overrides must be a JSON object")?;
        } else if let Some(path) = id.strip_prefix("source.file/") {
            let bytes = draft.program.files.get_mut(path).ok_or("Captured member is unavailable")?;
            *bytes = value.as_bytes().to_vec();
        } else {
            return Err("Unsupported source edit".into());
        }
        store.preview_source_draft(&mut draft).map_err(|error| error.to_string())?;
        let candidate = &draft.preview.as_ref().ok_or("Source preview unavailable")?.candidate;
        self.output.push_line(&crate::authoring::shell::preview_summary(Some(&self.workflow), candidate)?);
        self.output.push_line("Last admitted graph is unchanged. CLI authoring preview --expanded-output inspects exact candidate configuration; use Source: admit only after review.");
        Ok(())
    }
}
