use alloc::string::String;

const MAX_PROFILE_CHARS: u32 = 128;
const MAX_CONTEXT_CHARS: u32 = 32_768;
const MAX_LINEAGE_LABEL_CHARS: u32 = 256;

/// Bounded `rlm(...)` child-spawn request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSpawnRequest {
    profile: String,
    context_slice: String,
    depth: u32,
    fan_out: u32,
    token_budget: u64,
    wall_time_ms: u64,
    result_char_budget: u32,
    ambient_discovery: bool,
    cross_session: bool,
}

impl ChildSpawnRequest {
    /// Builds a bounded child-spawn request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when text fields are malformed.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile: &str,
        context_slice: &str,
        depth: u32,
        fan_out: u32,
        token_budget: u64,
        wall_time_ms: u64,
        result_char_budget: u32,
        ambient_discovery: bool,
        cross_session: bool,
    ) -> Result<Self, crate::Error> {
        Ok(Self {
            profile: crate::required_text("child_profile", profile, MAX_PROFILE_CHARS)?,
            context_slice: crate::required_text("child_context_slice", context_slice, MAX_CONTEXT_CHARS)?,
            depth,
            fan_out,
            token_budget,
            wall_time_ms,
            result_char_budget,
            ambient_discovery,
            cross_session,
        })
    }

    /// Returns the requested child profile.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }

    /// Returns the explicit child context slice.
    #[must_use]
    pub fn context_slice(&self) -> &str {
        self.context_slice.as_str()
    }

    /// Returns the requested recursion depth.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the requested fan-out.
    #[must_use]
    pub const fn fan_out(&self) -> u32 {
        self.fan_out
    }

    /// Returns the requested token budget.
    #[must_use]
    pub const fn token_budget(&self) -> u64 {
        self.token_budget
    }

    /// Returns the requested wall-clock budget in milliseconds.
    #[must_use]
    pub const fn wall_time_ms(&self) -> u64 {
        self.wall_time_ms
    }

    /// Returns the requested result character budget.
    #[must_use]
    pub const fn result_char_budget(&self) -> u32 {
        self.result_char_budget
    }

    /// Returns whether ambient child discovery was requested.
    #[must_use]
    pub const fn ambient_discovery(&self) -> bool {
        self.ambient_discovery
    }

    /// Returns whether cross-session registration was requested.
    #[must_use]
    pub const fn cross_session(&self) -> bool {
        self.cross_session
    }

    /// Returns the bounded request character count.
    #[must_use]
    pub fn character_count(&self) -> usize {
        self.profile.chars().count().saturating_add(self.context_slice.chars().count())
    }
}

/// Safe parent-child lineage metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lineage {
    parent_id_hash: String,
    child_id_hash: String,
    profile: String,
}

impl Lineage {
    /// Builds safe lineage metadata from BLAKE3 identities and a profile label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when an identity or label is malformed.
    pub fn new(parent_id_hash: &str, child_id_hash: &str, profile: &str) -> Result<Self, crate::Error> {
        validate_hash("parent_id_hash", parent_id_hash)?;
        validate_hash("child_id_hash", child_id_hash)?;
        Ok(Self {
            parent_id_hash: String::from(parent_id_hash),
            child_id_hash: String::from(child_id_hash),
            profile: crate::required_text("child_profile", profile, MAX_LINEAGE_LABEL_CHARS)?,
        })
    }

    /// Returns the parent identity hash.
    #[must_use]
    pub fn parent_id_hash(&self) -> &str {
        self.parent_id_hash.as_str()
    }

    /// Returns the child identity hash.
    #[must_use]
    pub fn child_id_hash(&self) -> &str {
        self.child_id_hash.as_str()
    }

    /// Returns the child profile label.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }
}

/// Typed bounded child result returned to the parent kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildResult {
    lineage: Lineage,
    value: String,
    value_hash: String,
}

impl ChildResult {
    /// Builds a child result bounded by the admitted spawn request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the result is empty, contains NUL, or
    /// exceeds the request result budget.
    pub fn new(request: &ChildSpawnRequest, lineage: Lineage, value: &str) -> Result<Self, crate::Error> {
        let value = crate::required_text("child_result", value, request.result_char_budget)?;
        let value_hash = crate::hash_parts("animus.rlm-child-result.v1", &[value.as_str()]);
        Ok(Self {
            lineage,
            value,
            value_hash,
        })
    }

    /// Returns the safe lineage metadata.
    #[must_use]
    pub const fn lineage(&self) -> &Lineage {
        &self.lineage
    }

    /// Returns the bounded model-visible result value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the domain-separated BLAKE3 result identity.
    #[must_use]
    pub fn value_hash(&self) -> &str {
        self.value_hash.as_str()
    }
}

/// Revalidates a child-spawn request before existing subagent owners run.
///
/// # Errors
///
/// Returns [`crate::Error`] for an unadmitted profile, ambient spawn, or any
/// depth, fan-out, token, wall-clock, or result-size budget violation.
pub fn admit_child_spawn(policy: &crate::RlmKernelPolicy, request: &ChildSpawnRequest) -> Result<(), crate::Error> {
    if request.ambient_discovery || request.cross_session {
        return Err(crate::Error::AmbientChildSpawnDenied);
    }
    if !policy.admits_child_profile(request.profile()) {
        return Err(crate::Error::ChildProfileNotAdmitted);
    }
    let bounds = policy.bounds();
    crate::ensure_at_most("child_depth", u64::from(request.depth), u64::from(bounds.max_child_depth))?;
    crate::ensure_at_most("child_fan_out", u64::from(request.fan_out), u64::from(bounds.max_child_fan_out))?;
    crate::ensure_at_most("child_tokens", request.token_budget, bounds.max_child_tokens)?;
    crate::ensure_at_most("child_wall_time_ms", request.wall_time_ms, bounds.max_child_wall_time_ms)?;
    crate::ensure_at_most(
        "child_result_chars",
        u64::from(request.result_char_budget),
        u64::from(bounds.max_child_result_chars),
    )
}

fn validate_hash(field: &'static str, value: &str) -> Result<(), crate::Error> {
    if crate::is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(crate::Error::InvalidBlake3Identity { field })
    }
}
