//! A first run's repair gates before its fence record: the lineage and the
//! repair ports.

// r[impl lattice.implementation_attempt.lineage.scenario.exhausted]
// r[impl lattice.implementation_attempt.operator_repair.scenario.authority]
// r[impl lattice.implementation_attempt.operator_repair.scenario.absent]

type Stop = super::super::refusal::deferred::Stop;
type Pending = super::super::refusal::deferred::Pending;

/// The repair ports a first run admitted before its fence record.
pub(in super::super) enum Admitted<'a> {
    /// No repair policy: the single-shot step.
    SingleShot,
    /// Embedder-supplied ports.
    Supplied(super::Ports<'a>),
    /// The standalone CLI's operator ports, granted by its profile.
    Operator(Box<super::operator::Granted<'a>>),
}

/// Before the fence record: the lineage must admit the first step, and a
/// repair policy needs embedder ports or the standalone operator's ports
/// with an admitted catalog and a granting profile. Each refusal is typed;
/// the caller settles it under the per-work lock, which closes the
/// attempt-fence.
pub(in super::super) fn admit<'a>(
    request: &super::super::Request,
    ports: Option<super::Ports<'a>>,
    operator: Option<super::operator::Standalone<'a>>,
) -> Result<Admitted<'a>, Stop> {
    debug_assert!(request.validate_binding().is_ok(), "admission follows request validation");
    if let Some(lineage) = &request.lineage {
        let next = super::first_reservation(request)?;
        debug_assert!(next.model_requests > 0, "a step reserves at least one model request");
        let drawn = crate::implementation_attempt::lineage::Draws::default();
        if let Err(refusal) = lineage.gate(&drawn, &next) {
            let reason = match refusal {
                crate::implementation_attempt::lineage::Refusal::Exhausted { .. } => {
                    super::super::refusal::Reason::LineageExhausted
                }
                _ => super::super::refusal::Reason::LineageBlocked,
            };
            return Err(Stop::Refused(Pending::lineage(reason, &refusal)));
        }
    }
    match (&request.repair, ports, operator) {
        (None, _, _) => Ok(Admitted::SingleShot),
        (Some(_), Some(ports), _) => Ok(Admitted::Supplied(ports)),
        (Some(_), None, Some(operator)) => match super::operator::admit(request, operator) {
            Ok(granted) => Ok(Admitted::Operator(Box::new(granted))),
            Err(super::operator::Refusal::Ports(error)) => Err(Stop::Refused(Pending::catalog(error))),
            Err(super::operator::Refusal::Authority(refusal)) => Err(Stop::Refused(Pending::authority(refusal))),
        },
        (Some(_), None, None) => {
            Err(Stop::Refused(Pending::new(super::super::refusal::Reason::RepairPortsUnavailable)))
        }
    }
}
