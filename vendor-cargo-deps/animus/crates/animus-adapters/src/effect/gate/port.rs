/// Replaceable host-authority provider over implementation-neutral Animus contracts.
pub trait HostAuthorityService {
    /// Evaluate one exact validated request without dispatching its host effect.
    ///
    /// # Errors
    ///
    /// Returns a stable issue when the provider is missing, stale, unsupported,
    /// denied, or unable to produce a safe exact decision.
    fn authorize(
        &mut self,
        request: &super::input::HostAuthorityRequest,
    ) -> Result<super::evaluate::HostAuthorityDecision, super::evaluate::HostAuthorityIssue>;
}
