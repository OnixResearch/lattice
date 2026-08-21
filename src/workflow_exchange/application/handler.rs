#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed transport dispatcher keeps each producer command and result variant explicit"
)]

use lattice_workflow_transport::ExchangeCommand;
use lattice_workflow_transport::ExchangeSuccess;
use lattice_workflow_transport::HandlerError;
use lattice_workflow_transport::WorkflowExchangeHandler;

use super::ApplicationHandler;

// r[impl lattice.workflow_interchange.application_lifecycle]
// r[impl lattice.workflow_interchange.application_recovery]
// r[impl lattice.workflow_interchange.application_logs]
impl WorkflowExchangeHandler for ApplicationHandler {
    fn handle(&mut self, command: &ExchangeCommand) -> Result<ExchangeSuccess, HandlerError> {
        self.refresh_workers();
        match command {
            ExchangeCommand::Dispatch { request, .. } => self.dispatch(request).map(ExchangeSuccess::Accepted),
            ExchangeCommand::RecoverDispatch { request, .. } => {
                self.recover_dispatch(request).map(ExchangeSuccess::Accepted)
            }
            ExchangeCommand::Observe {
                profile,
                request,
                accepted,
            }
            | ExchangeCommand::Reconcile {
                profile,
                request,
                accepted,
            } => self.observe(profile, request, accepted).map(ExchangeSuccess::Observation),
            ExchangeCommand::Control {
                profile,
                request,
                accepted,
                current_sequence,
                control,
            } => self
                .control(profile, request, accepted, *current_sequence, control)
                .map(ExchangeSuccess::Observation),
            ExchangeCommand::ReadLog {
                profile,
                request,
                accepted,
                log_read,
            } => self.read_log(profile, request, accepted, log_read),
        }
    }
}
