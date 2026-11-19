//! Serving readiness for inventory: the injected port's latest snapshot of
//! each served route, screened for freshness by the producer, or the actual
//! reason none was read. Only `observe_snapshot` is called; never admission or
//! recheck, and nothing here is a reservation or a dispatch decision.

// r[impl lattice.serving_binding.readiness]
// r[impl lattice.serving_binding.readiness.scenario.no_port]
// r[impl lattice.serving_binding.readiness.scenario.injected]

type Declaration = lattice_core::execution_binding::serving::Declaration;
type State = lattice_core::execution_binding::projection::ServingReadiness;
type Reading = lattice_core::execution_binding::projection::SnapshotReading;

/// Readiness of every route `declaration` serves, read through `port`.
pub fn observe(
    declaration: Option<&Declaration>,
    attempt_id: &str,
    port: &mut dyn super::OwnerPort,
) -> Result<State, String> {
    if port.provenance() == super::PortProvenance::Unavailable {
        return Ok(State::NoServingPort {});
    }
    let Some(declaration) = declaration else {
        return Ok(State::NoServedRoute {});
    };
    let profile = super::draft::profile(&declaration.profile)?;
    let privacy = super::draft::privacy(declaration.privacy);
    let mut routes = Vec::with_capacity(declaration.routes.len());
    for route in &declaration.routes {
        let query = super::SnapshotQuery {
            attempt_id: attempt_id.to_owned(),
            route_name: route.name.clone(),
            service_id: route.service_id.clone(),
        };
        let snapshot = match port.observe_snapshot(&query) {
            super::SnapshotReply::Observed { snapshot, now_tick } => {
                reading(&profile, privacy, route, (&snapshot, now_tick))?
            }
            super::SnapshotReply::Unavailable => Reading::Unavailable {},
            super::SnapshotReply::PortIssue => Reading::PortIssue {},
        };
        routes.push(lattice_core::execution_binding::projection::RouteReadiness {
            route_name: route.name.clone(),
            service_id: route.service_id.clone(),
            snapshot,
        });
    }
    debug_assert_eq!(routes.len(), declaration.routes.len());
    debug_assert!(!routes.is_empty(), "a validated declaration serves a route");
    Ok(State::Observed { routes })
}

/// The service's readiness and the snapshot's freshness under the profile.
fn reading(
    profile: &animus_adapters::serving::Profile,
    privacy: animus_adapters::router::PrivacyBand,
    route: &lattice_core::execution_binding::serving::Route,
    (snapshot, now_tick): (&animus_adapters::serving::AvailabilitySnapshot, Option<u64>),
) -> Result<Reading, String> {
    let candidate = super::draft::route_candidate(route, None)?;
    let screening = animus_adapters::serving::screen_routes(&animus_adapters::serving::ScreenRequest {
        profile,
        snapshot,
        freshness: animus_adapters::serving::Freshness {
            now_tick,
            newest_accepted: None,
        },
        privacy,
        candidates: std::slice::from_ref(&candidate),
    })
    .map_err(|issue| format!("serving screen contract issue: {issue:?}"))?;
    let readiness = snapshot
        .services()
        .iter()
        .find(|service| service.deployment().service_id() == route.service_id)
        .map_or("not-listed", |service| service.readiness().as_str());
    debug_assert_eq!(screening.snapshot_identity(), snapshot.identity());
    Ok(Reading::Observed {
        snapshot_identity: snapshot.identity().to_owned(),
        source_generation: snapshot.source_generation(),
        sequence: snapshot.sequence(),
        readiness: readiness.to_owned(),
        freshness: screening.status().as_str().to_owned(),
    })
}
