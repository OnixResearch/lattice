/// Actor label used in provenance and Toolpath projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceActor {
    /// Actor taxonomy label, such as `human`, `agent`, `tool`, or `ci`.
    pub kind: Text,
    /// Safe actor identity label.
    pub label: Text,
}

impl ProvenanceActor {
    /// Creates a safe actor label.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] if either label is empty, overlong, or unsafe.
    pub fn new(kind: impl Into<Text>, label: impl Into<Text>) -> Result<Self, ProvenanceError> {
        let kind = validate_safe_text("actor_kind", kind.into())?;
        let label = validate_safe_text("actor_label", label.into())?;
        Ok(Self { kind, label })
    }

    /// Returns the Toolpath-compatible `kind:label` actor label.
    #[must_use]
    pub fn toolpath_label(&self) -> Text {
        format!("{}:{}", self.kind, self.label)
    }
}

/// Model-visible replay role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceReplayRole {
    /// User-authored model-visible message.
    User,
    /// Assistant-authored model-visible message.
    Assistant,
    /// Tool result visible to a model.
    Tool,
    /// Bounded summary visible to a model.
    Summary,
}

impl ProvenanceReplayRole {
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::Summary => "summary",
        }
    }
}

/// Model-visible message rebuilt by pure replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceReplayMessage {
    /// Role of the replayed message.
    pub role: ProvenanceReplayRole,
    /// Message body supplied from the already-loaded in-memory ledger.
    pub content: Text,
}

/// Safe replay metadata counts.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProvenanceReplayMetadata {
    /// Number of user messages observed.
    pub user_messages: u32,
    /// Number of assistant messages observed.
    pub assistant_messages: u32,
    /// Number of tool messages observed.
    pub tool_messages: u32,
    /// Number of summaries observed.
    pub summaries: u32,
    /// Number of usage observations observed.
    pub usage_observations: u32,
    /// Number of safe receipts observed.
    pub receipts: u32,
}

/// Result of deterministic ledger replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceReplay {
    /// Model-visible context reconstructed in input order.
    pub model_visible: Collection<ProvenanceReplayMessage>,
    /// Safe metadata counts for receipts and audit.
    pub metadata: ProvenanceReplayMetadata,
}

/// In-memory ledger message entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceMessageEntry {
    /// Stable entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
    /// Message role.
    pub role: ProvenanceReplayRole,
    /// Safe actor label.
    pub actor: ProvenanceActor,
    /// Already-loaded model-visible content.
    pub content: Text,
    /// BLAKE3 hash of the content or source envelope.
    pub content_hash: Text,
}

/// Bounded summary entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceSummaryEntry {
    /// Stable entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
    /// Summary body visible to a model.
    pub summary: Text,
    /// BLAKE3 hash of the summary body.
    pub summary_hash: Text,
}

/// Usage observation entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceUsageEntry {
    /// Stable entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
    /// Safe model label.
    pub model_label: Text,
    /// Input token count reported by the host.
    pub input_tokens: u64,
    /// Output token count reported by the host.
    pub output_tokens: u64,
}

/// Unsupported adapter-owned entry represented safely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceUnsupportedEntry {
    /// Stable entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
    /// Safe adapter label.
    pub adapter_label: Text,
    /// Hash of the omitted raw payload.
    pub payload_hash: Text,
}

/// Neutral provenance ledger entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceLedgerEntry {
    /// Model-visible message.
    Message(ProvenanceMessageEntry),
    /// Bounded summary.
    Summary(ProvenanceSummaryEntry),
    /// Usage observation.
    Usage(ProvenanceUsageEntry),
    /// Safe receipt.
    Receipt(ProvenanceReceipt),
    /// Adapter-owned entry that replay must fail closed on.
    Unsupported(ProvenanceUnsupportedEntry),
}

impl ProvenanceLedgerEntry {
    /// Returns this entry identity.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Message(entry) => entry.id.as_str(),
            Self::Summary(entry) => entry.id.as_str(),
            Self::Usage(entry) => entry.id.as_str(),
            Self::Receipt(entry) => entry.id.as_str(),
            Self::Unsupported(entry) => entry.id.as_str(),
        }
    }

    /// Returns this entry parent identities.
    #[must_use]
    pub fn parents(&self) -> &[Text] {
        match self {
            Self::Message(entry) => entry.parents.as_slice(),
            Self::Summary(entry) => entry.parents.as_slice(),
            Self::Usage(entry) => entry.parents.as_slice(),
            Self::Receipt(entry) => entry.parents.as_slice(),
            Self::Unsupported(entry) => entry.parents.as_slice(),
        }
    }

    fn graph_entry(&self) -> ProvenanceGraphEntry {
        ProvenanceGraphEntry {
            id: self.id().to_owned(),
            parents: self.parents().to_vec(),
        }
    }
}

/// Replays already-loaded neutral ledger entries into model-visible context.
///
/// # Errors
///
/// Returns [`ProvenanceError`] when an unsupported adapter-owned entry is
/// encountered. The error contains only safe labels and hashes.
pub fn replay_provenance_ledger(
    entries: &[ProvenanceLedgerEntry],
) -> Result<ProvenanceReplay, ProvenanceError> {
    let mut model_visible = Collection::new();
    let mut metadata = ProvenanceReplayMetadata::default();

    for entry in entries {
        match entry {
            ProvenanceLedgerEntry::Message(message) => {
                match message.role {
                    ProvenanceReplayRole::User => metadata.user_messages += 1,
                    ProvenanceReplayRole::Assistant => metadata.assistant_messages += 1,
                    ProvenanceReplayRole::Tool => metadata.tool_messages += 1,
                    ProvenanceReplayRole::Summary => metadata.summaries += 1,
                }
                model_visible.push(ProvenanceReplayMessage {
                    role: message.role,
                    content: message.content.clone(),
                });
            }
            ProvenanceLedgerEntry::Summary(summary) => {
                metadata.summaries += 1;
                model_visible.push(ProvenanceReplayMessage {
                    role: ProvenanceReplayRole::Summary,
                    content: summary.summary.clone(),
                });
            }
            ProvenanceLedgerEntry::Usage(_) => {
                metadata.usage_observations += 1;
            }
            ProvenanceLedgerEntry::Receipt(_) => {
                metadata.receipts += 1;
            }
            ProvenanceLedgerEntry::Unsupported(unsupported) => {
                return Err(ProvenanceError::new(
                    ProvenanceErrorKind::UnsupportedEntry,
                    "ledger_entry",
                    unsupported.id.as_str(),
                    format!(
                        "unsupported adapter {} payload {}",
                        unsupported.adapter_label, unsupported.payload_hash
                    ),
                ));
            }
        }
    }

    Ok(ProvenanceReplay {
        model_visible,
        metadata,
    })
}
