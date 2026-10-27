//! Explicit Tile metadata observation and receipt publication only.
//! There is deliberately no executor, workflow runner, UI handle, or ambient socket.

use std::time::Duration;

use async_trait::async_trait;
use lattice_compose_binding::ActivationReceipt;
use lattice_compose_binding::AdmissionError;
use lattice_compose_binding::BindingRequest;
use lattice_compose_binding::ObservationSchema;
use lattice_compose_binding::PaneLifecycle;
use lattice_compose_binding::PaneObservation;
use lattice_compose_binding::SourceIdentity;
use lattice_compose_binding::TILE_REVISION;
use lattice_compose_binding::admit;
use lattice_compose_binding::plan;
use lattice_workflow_contract::Identity;
use tile_protocol::WorkspaceInspectCapability;
use tile_protocol::WorkspaceInspectPaneLifecycle;
use tile_protocol::WorkspaceInspectRequest;

#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    #[error("compose admission failed: {0}")]
    Admission(AdmissionError),
    #[error("Tile observation failed")]
    ObservationUnavailable,
    #[error("Tile observation deadline exceeded")]
    ObservationTimeout,
    #[error("Tile observation projection rejected")]
    InvalidProjection,
    #[error("Tile pane absent from inspected workspace")]
    MissingPane,
    #[error("compose receipt publication failed")]
    PublicationFailed,
    #[error("compose input exceeds its byte bound")]
    InputTooLarge,
    #[error("compose input is not a closed valid DTO")]
    InvalidInput,
}

impl From<AdmissionError> for ComposeError {
    fn from(error: AdmissionError) -> Self {
        Self::Admission(error)
    }
}

/// The host injects observation authority; a binding request never creates it.
// r[impl lattice.compose_surface.dependency]
#[async_trait]
pub trait TilePaneObservationPort: Send {
    fn producer_revision(&self) -> &str;
    async fn observe(&mut self, source: &SourceIdentity) -> Result<PaneObservation, ComposeError>;
}

/// One publication call per accepted activation; no automatic retry after uncertainty.
pub trait ReceiptPublisher {
    fn publish(&mut self, receipt: &ActivationReceipt) -> Result<(), ComposeError>;
}

/// Connection and daemon authority are explicit host inputs. Construction never
/// starts a daemon and never consults TILE_DAEMON_SOCKET, HOME, or UI state.
pub struct TileSdkPanePort {
    client: tile_sdk::TileClient,
    daemon_identity: Identity,
}

impl TileSdkPanePort {
    pub fn new(client: tile_sdk::TileClient, daemon_identity: Identity) -> Self {
        Self {
            client,
            daemon_identity,
        }
    }
}

const OBSERVATION_DEADLINE: Duration = Duration::from_secs(10);

#[async_trait]
impl TilePaneObservationPort for TileSdkPanePort {
    fn producer_revision(&self) -> &str {
        TILE_REVISION
    }

    async fn observe(&mut self, source: &SourceIdentity) -> Result<PaneObservation, ComposeError> {
        if source.daemon_identity != self.daemon_identity {
            return Err(AdmissionError::SourceMismatch.into());
        }
        let bounds = WorkspaceInspectRequest::bounded(
            tile_protocol::tile_core::SessionId(source.session_id),
            Some(tile_protocol::tile_core::WorkspaceRevision(source.workspace_revision)),
        );
        bounds.validate().map_err(|_| ComposeError::InvalidProjection)?;
        let inspection = tokio::time::timeout(OBSERVATION_DEADLINE, self.client.inspect_workspace(bounds))
            .await
            .map_err(|_| ComposeError::ObservationTimeout)?
            .map_err(|_| ComposeError::ObservationUnavailable)?;
        project_inspection(inspection, source, &bounds)
    }
}

fn project_inspection(
    inspection: tile_protocol::WorkspaceInspectResponse,
    source: &SourceIdentity,
    bounds: &WorkspaceInspectRequest,
) -> Result<PaneObservation, ComposeError> {
    // Validate the real producer's complete omission/identity/bound contract
    // before narrowing it. Do not reinterpret permission to inspect as run authority.
    inspection.validate(bounds).map_err(|_| ComposeError::InvalidProjection)?;
    if !inspection.capabilities.contains(&WorkspaceInspectCapability::Inspect) {
        return Err(ComposeError::InvalidProjection);
    }
    let pane = inspection
        .panes
        .iter()
        .find(|pane| pane.pane_id.0 == source.pane_id)
        .ok_or(ComposeError::MissingPane)?;
    let lifecycle = match pane.lifecycle {
        WorkspaceInspectPaneLifecycle::Starting => PaneLifecycle::Starting,
        WorkspaceInspectPaneLifecycle::Running => PaneLifecycle::Running,
        WorkspaceInspectPaneLifecycle::Exited => PaneLifecycle::Exited,
        WorkspaceInspectPaneLifecycle::NeedsRestart => PaneLifecycle::NeedsRestart,
    };
    Ok(PaneObservation {
        schema: ObservationSchema::V1,
        source: SourceIdentity {
            producer_revision: TILE_REVISION.to_owned(),
            daemon_identity: source.daemon_identity.clone(),
            session_id: inspection.session_id.0,
            pane_id: pane.pane_id.0,
            workspace_revision: inspection.workspace_revision.0,
        },
        lifecycle,
        focused: pane.focused,
    })
}

/// Bounded JSON admission belongs in the shell, ahead of allocating wire DTOs.
pub fn decode_request(bytes: &[u8]) -> Result<BindingRequest, ComposeError> {
    if bytes.len() > lattice_compose_binding::MAXIMUM_REQUEST_BYTES {
        return Err(ComposeError::InputTooLarge);
    }
    serde_json::from_slice(bytes).map_err(|_| ComposeError::InvalidInput)
}

// r[impl lattice.compose_surface.plan]
// r[impl lattice.compose_surface.receipt]
pub async fn activate(
    observations: &mut impl TilePaneObservationPort,
    receipts: &mut impl ReceiptPublisher,
    request: BindingRequest,
) -> Result<ActivationReceipt, ComposeError> {
    // Reject revision/identity/capability errors before contacting the producer.
    lattice_compose_binding::validate_request(&request)?;
    let revision = observations.producer_revision();
    if revision.is_empty() {
        return Err(AdmissionError::MissingSourceRevision.into());
    }
    if revision != TILE_REVISION {
        return Err(AdmissionError::UnknownSourceRevision.into());
    }
    let observation = observations.observe(&request.source).await?;
    let plan = plan(admit(observation, request)?)?;
    let receipt = plan.receipt()?;
    receipts.publish(&receipt)?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "closed fixtures fail immediately on setup errors")]
    use super::*;

    fn fixture() -> (tile_protocol::WorkspaceInspectResponse, SourceIdentity, WorkspaceInspectRequest) {
        let mut session = tile_protocol::tile_core::WorkspaceSession::new(
            tile_protocol::tile_core::SessionId(1),
            "private title",
            "/private/path",
        );
        let pane_id = session.active_tab().focused;
        session.panes.get_mut(&pane_id).unwrap().lifecycle = tile_protocol::tile_core::PaneLifecycle::Running;
        let bounds = WorkspaceInspectRequest::bounded(session.id, Some(session.revision));
        let inspection = tile_protocol::project_workspace_inspection(&session, &bounds).unwrap();
        let source = SourceIdentity {
            producer_revision: TILE_REVISION.to_owned(),
            daemon_identity: Identity::new(format!("b3:{}", "1".repeat(64))).unwrap(),
            session_id: session.id.0,
            pane_id: session.active_tab().focused.0,
            workspace_revision: session.revision.0,
        };
        (inspection, source, bounds)
    }

    // r[verify lattice.compose_surface.admission.scenario.valid]
    #[test]
    fn actual_tile_projection_produces_only_identity_lifecycle_and_focus() {
        let (inspection, source, bounds) = fixture();
        let observation = project_inspection(inspection, &source, &bounds).unwrap();
        assert_eq!(observation.source, source);
        assert_eq!(observation.lifecycle, PaneLifecycle::Running);
        assert!(observation.focused);
    }

    // r[verify lattice.compose_surface.admission.scenario.malformed]
    #[test]
    fn stale_unredacted_unknown_and_unprivileged_projections_reject() {
        let (inspection, source, bounds) = fixture();
        let mut stale = inspection.clone();
        stale.workspace_revision.0 += 1;
        assert!(matches!(project_inspection(stale, &source, &bounds), Err(ComposeError::InvalidProjection)));
        let mut exposed = inspection.clone();
        exposed.redaction.secrets_omitted = false;
        assert!(matches!(project_inspection(exposed, &source, &bounds), Err(ComposeError::InvalidProjection)));
        let mut denied = inspection.clone();
        denied.capabilities.remove(&WorkspaceInspectCapability::Inspect);
        assert!(matches!(project_inspection(denied, &source, &bounds), Err(ComposeError::InvalidProjection)));
        let mut missing = source;
        missing.pane_id = u64::MAX;
        assert!(matches!(project_inspection(inspection, &missing, &bounds), Err(ComposeError::MissingPane)));
    }

    #[test]
    fn request_decoder_bounds_bytes_before_dto_admission() {
        let bytes = include_bytes!("../crates/lattice-compose-binding/fixtures/request.json");
        let mut maximum = bytes.to_vec();
        maximum.resize(lattice_compose_binding::MAXIMUM_REQUEST_BYTES, b' ');
        assert_eq!(decode_request(&maximum).unwrap().source.pane_id, 1);
        maximum.push(b' ');
        assert!(matches!(decode_request(&maximum), Err(ComposeError::InputTooLarge)));
    }
}
