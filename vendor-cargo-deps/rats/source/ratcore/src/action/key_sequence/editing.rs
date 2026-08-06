use super::ActionBindingId;
use super::BindingLayerId;
use super::KeySequence;
use super::LogicalKeystroke;
use super::catalog::ActionBinding;
use super::catalog::ActiveBindingLayers;
use super::catalog::BindingCatalogDiagnostic;
use super::catalog::BindingCatalogDiagnosticCode;
use super::catalog::BindingCatalogLimits;
use super::catalog::BindingCatalogSnapshot;

/// Complete-sequence capture state owned by a renderer/application editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceCapture {
    observed_catalog_generation: u64,
    observed_context_generation: u64,
    strokes: Vec<LogicalKeystroke>,
}

impl SequenceCapture {
    /// Starts an empty capture against observed accepted generations.
    #[must_use]
    pub const fn new(observed_catalog_generation: u64, observed_context_generation: u64) -> Self {
        Self {
            observed_catalog_generation,
            observed_context_generation,
            strokes: Vec::new(),
        }
    }

    /// Returns captured strokes without completing an edit.
    #[must_use]
    pub fn strokes(&self) -> &[LogicalKeystroke] {
        &self.strokes
    }

    /// Appends one validated logical stroke, preserving prior state on failure.
    #[must_use]
    pub fn push(&self, stroke: LogicalKeystroke, limits: &BindingCatalogLimits) -> SequenceCaptureOutcome {
        let mut next = self.clone();
        next.strokes.push(stroke);
        match KeySequence::try_new(next.strokes.clone(), limits) {
            Ok(_) => SequenceCaptureOutcome::Capturing(next),
            Err(issues) => SequenceCaptureOutcome::Rejected {
                state: self.clone(),
                diagnostic: SequenceCaptureDiagnostic {
                    message: commands::format_sequence_issues(&issues),
                    issues,
                },
            },
        }
    }

    /// Finishes the complete sequence if capture generations are current.
    #[must_use]
    pub fn finish(&self, catalog: &BindingCatalogSnapshot, active: &ActiveBindingLayers) -> SequenceCaptureOutcome {
        let current = self.observed_catalog_generation == catalog.generation()
            && self.observed_context_generation == active.context_generation();
        if !current {
            return SequenceCaptureOutcome::Rejected {
                state: self.clone(),
                diagnostic: SequenceCaptureDiagnostic {
                    issues: Vec::new(),
                    message: "sequence capture observes stale catalog or context generation".to_string(),
                },
            };
        }
        match KeySequence::try_new(self.strokes.clone(), &catalog.limits()) {
            Ok(sequence) => SequenceCaptureOutcome::Complete(sequence),
            Err(issues) => SequenceCaptureOutcome::Rejected {
                state: self.clone(),
                diagnostic: SequenceCaptureDiagnostic {
                    message: commands::format_sequence_issues(&issues),
                    issues,
                },
            },
        }
    }
}

/// Capture rejection evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceCaptureDiagnostic {
    /// Logical sequence diagnostics, empty for stale generation rejection.
    pub issues: Vec<super::KeySequenceDiagnostic>,
    /// Human-readable recovery explanation.
    pub message: String,
}

/// Result of one capture operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceCaptureOutcome {
    /// Capture remains open with accepted strokes.
    Capturing(SequenceCapture),
    /// Complete validated sequence.
    Complete(KeySequence),
    /// Capture failed and preserves the prior state.
    Rejected {
        /// Preserved prior state.
        state: SequenceCapture,
        /// Rejection evidence.
        diagnostic: SequenceCaptureDiagnostic,
    },
}

/// Atomic sequence-aware binding edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingEditCommand {
    /// Add another binding for an action.
    Add {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Complete candidate binding.
        binding: ActionBinding,
    },
    /// Replace a binding while preserving replacement provenance.
    Replace {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Existing binding to replace.
        binding_id: ActionBindingId,
        /// Complete replacement binding with a distinct identity.
        replacement: ActionBinding,
    },
    /// Clear one binding without deleting its provenance.
    Clear {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Binding to clear.
        binding_id: ActionBindingId,
    },
    /// Clear all bindings for one action, optionally in one layer.
    ClearAction {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Action to clear.
        action_id: super::super::ActionId,
        /// Optional layer restriction.
        layer_id: Option<BindingLayerId>,
    },
    /// Remove user overrides and restore non-user bindings for an action.
    ResetAction {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Action to reset.
        action_id: super::super::ActionId,
    },
    /// Renderer reports a capture it cannot normalize.
    UnsupportedCapture {
        /// Catalog generation observed by the editor.
        observed_catalog_generation: u64,
        /// Renderer-local key label.
        key: String,
    },
}

impl BindingEditCommand {
    const fn observed_generation(&self) -> u64 {
        match self {
            Self::Add {
                observed_catalog_generation,
                ..
            }
            | Self::Replace {
                observed_catalog_generation,
                ..
            }
            | Self::Clear {
                observed_catalog_generation,
                ..
            }
            | Self::ClearAction {
                observed_catalog_generation,
                ..
            }
            | Self::ResetAction {
                observed_catalog_generation,
                ..
            }
            | Self::UnsupportedCapture {
                observed_catalog_generation,
                ..
            } => *observed_catalog_generation,
        }
    }
}

/// Binding edit result with atomic accepted/preserved snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingEditOutcome {
    /// Accepted new snapshot or exact previous snapshot on rejection.
    pub snapshot: BindingCatalogSnapshot,
    /// Deterministic validation/edit diagnostics.
    pub diagnostics: Vec<BindingCatalogDiagnostic>,
    /// Whether the complete edit was accepted.
    pub accepted: bool,
}

/// Applies a complete sequence edit atomically through catalog validation.
#[must_use]
pub fn apply_binding_edit(
    snapshot: &BindingCatalogSnapshot,
    registry: &super::super::ActionRegistrySnapshot,
    command: &BindingEditCommand,
    next_generation: u64,
) -> BindingEditOutcome {
    if command.observed_generation() != snapshot.generation() {
        return commands::rejected(
            snapshot,
            BindingCatalogDiagnostic::new(
                BindingCatalogDiagnosticCode::StaleEditGeneration,
                "edit.observed_catalog_generation",
                "binding edit observes a stale catalog generation",
            ),
        );
    }
    if let BindingEditCommand::UnsupportedCapture { key, .. } = command {
        return commands::rejected(
            snapshot,
            BindingCatalogDiagnostic::new(
                BindingCatalogDiagnosticCode::UnsupportedCapture,
                "edit.capture",
                format!("renderer cannot normalize captured key `{key}`"),
            ),
        );
    }
    let mut candidate = snapshot.candidate_with_generation(next_generation);
    if let Some(diagnostic) = commands::apply_candidate_edit(&mut candidate.bindings, command) {
        return commands::rejected(snapshot, diagnostic);
    }
    let build = super::accept_binding_catalog(candidate, registry, Some(snapshot));
    BindingEditOutcome {
        snapshot: build.snapshot.unwrap_or_else(|| snapshot.clone()),
        accepted: build.candidate_accepted,
        diagnostics: build.diagnostics,
    }
}

#[path = "editing/commands.rs"]
mod commands;
