//! Thin injected skill-body loading shell.

/// Bounded injected skill-body service failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillBodyServiceError {
    /// Service is not configured or available.
    Unavailable,
    /// Current policy denied body access.
    Denied,
    /// Service returned malformed bounded data.
    Malformed,
    /// Operation reached its explicit deadline.
    Timeout,
    /// Current request was cancelled.
    Cancelled,
}

/// Host-injected exact skill-body service.
///
/// Implementations retain filesystem, package, process, credential, and cache authority.
pub trait SkillBodyService {
    /// Loads one exact current body.
    ///
    /// # Errors
    ///
    /// Returns a bounded service class without raw paths, body text, or credentials.
    fn load_body(
        &mut self,
        request: &super::data::SkillBodyLoadRequest,
    ) -> Result<super::data::LoadedSkillBody, SkillBodyServiceError>;
}

/// Loaded request-local body plus admitted pure lifecycle outputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedSkillTurn {
    /// Updated pure skill-turn state.
    pub turn: super::data::SkillTurn,
    /// One next-provider-response contribution.
    pub contribution: super::data::Contribution,
    /// Metadata-only selection result.
    pub result: super::data::SkillSelectionResult,
    /// Request-local body retained only by the immediate caller.
    pub body: super::data::LoadedSkillBody,
}

/// Loads one exact skill body through an injected service and revalidates it in the pure core.
///
/// # Errors
///
/// Returns a bounded service, stale-body, transition, metadata, or budget issue.
pub fn load_skill_turn<S: SkillBodyService>(
    service: &mut S,
    turn: &super::data::SkillTurn,
    policy: super::data::SkillContributionPolicy<'_>,
) -> Result<LoadedSkillTurn, super::data::ContributionIssue> {
    let request = super::data::SkillBodyLoadRequest {
        binding: turn.binding.clone(),
    };
    let loaded = service
        .load_body(&request)
        .map_err(|_| super::data::ContributionIssue::SkillBodyServiceFailed)?;
    let (next_turn, contribution, result) =
        super::core::admit_loaded_skill_body(turn, &loaded, policy)?;
    Ok(LoadedSkillTurn {
        turn: next_turn,
        contribution,
        result,
        body: loaded,
    })
}
