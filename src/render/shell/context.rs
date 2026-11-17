/// Called only by the in-process Executor preparation path with its allocated
/// run/node identity. A workflow author cannot choose this context in a plan.
pub(super) fn bind_host_execution(
    request: &crate::render::model::Request,
    execution: &super::HostExecution,
    method: &str,
) -> anyhow::Result<(std::path::PathBuf, molten_batch_core::Id)> {
    super::check_current(request)?;
    let journal = super::Journal::open(request)?;
    let owner = bind_owner(&journal, execution, method)?;
    let permit = super::DispatchPermit {
        binding: request.id()?,
        owner,
        observer: execution.clone(),
        method: method.into(),
    };
    let identity = molten_orchestrator_codec::identity("lattice.render-dispatch.v1", &permit)?;
    let file = journal.exact_file(&format!("permit-{identity}.json"), &serde_json::to_vec(&permit)?)?;
    Ok((file, identity))
}

pub(super) fn validate_permit(
    request: &crate::render::model::Request,
    permit: &super::DispatchPermit,
) -> anyhow::Result<()> {
    super::check_current(request)?;
    anyhow::ensure!(
        permit.binding == request.id()? && crate::render::operation::METHODS.contains(&permit.method.as_str()),
        "crossed dispatch permit"
    );
    let root = std::path::PathBuf::from(&request.journal_root).join(request.id()?.to_string());
    let owner: super::HostExecution =
        serde_json::from_slice(&super::read_bounded(&root.join("host-execution.json"), 4096)?)?;
    anyhow::ensure!(
        permit.owner == owner && (permit.method != "advance" || permit.observer == owner),
        "crossed actual Lattice execution"
    );
    let id = molten_orchestrator_codec::identity("lattice.render-dispatch.v1", permit)?;
    let retained: super::DispatchPermit =
        serde_json::from_slice(&super::read_bounded(&root.join(format!("permit-{id}.json")), 16 * 1024)?)?;
    anyhow::ensure!(retained == *permit, "unretained dispatch context");
    Ok(())
}

fn bind_owner(
    journal: &super::Journal,
    execution: &super::HostExecution,
    method: &str,
) -> anyhow::Result<super::HostExecution> {
    match journal.owner()? {
        Some(owner) => {
            anyhow::ensure!(
                method != "advance" || owner == *execution,
                "another Lattice run owns this render; use explicit observe without redispatch"
            );
            Ok(owner)
        }
        None => {
            anyhow::ensure!(
                method == "advance" && !journal.record.dispatch_possible,
                "no engine-owned computation context to observe"
            );
            journal.exact_file("host-execution.json", &serde_json::to_vec(execution)?)?;
            Ok(execution.clone())
        }
    }
}
