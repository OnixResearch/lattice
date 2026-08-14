#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SafeApprovalView {
    pub request_id: String,
    pub run_id: crate::engine::executor::RunId,
    pub workflow_name: String,
    pub workflow_revision: String,
    pub node_id: crate::engine::workflow::NodeId,
    pub gate_id: String,
    pub safe_summary: String,
    pub status: super::ApprovalRequestStatus,
    pub requested_epoch_millis: i64,
    pub decision_deadline_epoch_millis: i64,
    pub decision: Option<super::DecisionClass>,
    pub decision_actor: Option<String>,
    pub completed_node_count: usize,
    pub event_count: usize,
    pub event_head: String,
    pub choregraph_profile: String,
    pub suspension_class: String,
    pub version: u64,
}

pub fn inspect(
    store: &crate::store::Store,
    request_id: &super::ApprovalRequestId,
) -> Result<Option<super::ApprovalRequest>, Error> {
    store.load_workflow_approval(request_id).map_err(Into::into)
}

pub fn receipt(
    store: &crate::store::Store,
    request_id: &super::ApprovalRequestId,
) -> Result<Option<super::ApprovalReceipt>, Error> {
    let Some(request) = inspect(store, request_id)? else {
        return Ok(None);
    };
    let workflow = load_bound_revision(store, &request)?;
    let config = approval_config(&workflow, request.node_id)?;
    super::core::approval_receipt(&request, config).map(Some).map_err(Into::into)
}

pub fn inspect_safe(
    store: &crate::store::Store,
    request_id: &super::ApprovalRequestId,
) -> Result<Option<SafeApprovalView>, Error> {
    inspect(store, request_id).map(|request| request.map(|request| safe_view(&request)))
}

pub fn list(store: &crate::store::Store, count: u32) -> Result<Vec<super::ApprovalRequest>, Error> {
    store.list_workflow_approvals(count).map_err(Into::into)
}

pub fn list_safe(store: &crate::store::Store, count: u32) -> Result<Vec<SafeApprovalView>, Error> {
    list(store, count).map(|requests| requests.iter().map(safe_view).collect())
}

pub fn render_safe_text(view: &SafeApprovalView) -> String {
    format!(
        concat!(
            "request={} run={} status={:?} gate={} node={}\n",
            "workflow={} revision={} summary={}\n",
            "decision={:?} actor={} deadline={}\n",
            "checkpoint_nodes={} events={} event_head={} version={}\n",
            "choregraph_profile={} suspension_class={}\n"
        ),
        view.request_id,
        view.run_id,
        view.status,
        view.gate_id,
        view.node_id.0,
        view.workflow_name,
        view.workflow_revision,
        view.safe_summary,
        view.decision,
        view.decision_actor.as_deref().unwrap_or("none"),
        view.decision_deadline_epoch_millis,
        view.completed_node_count,
        view.event_count,
        view.event_head,
        view.version,
        view.choregraph_profile,
        view.suspension_class,
    )
}

fn safe_view(request: &super::ApprovalRequest) -> SafeApprovalView {
    SafeApprovalView {
        request_id: request.id.to_string(),
        run_id: request.run_id,
        workflow_name: request.workflow_name.clone(),
        workflow_revision: request.workflow_revision.to_string(),
        node_id: request.node_id,
        gate_id: request.gate_id.to_string(),
        safe_summary: request.safe_summary.clone(),
        status: request.status,
        requested_epoch_millis: request.requested_epoch_millis,
        decision_deadline_epoch_millis: request.decision_deadline_epoch_millis,
        decision: request.decision.as_ref().map(|decision| decision.decision),
        decision_actor: request.decision.as_ref().map(|decision| decision.actor.to_string()),
        completed_node_count: request.completed_nodes.len(),
        event_count: request.events.len(),
        event_head: request.events.last().map(|event| event.id.to_string()).unwrap_or_else(|| "none".to_string()),
        choregraph_profile: request.choregraph_profile.clone(),
        suspension_class: request.suspension_class.clone(),
        version: request.version,
    }
}
