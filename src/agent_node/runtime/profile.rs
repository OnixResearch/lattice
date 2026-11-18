//! Authority-free compatibility facts for the routes the current host configures.

// r[impl lattice.agent_runtime.configured_bindings]
// r[impl lattice.agent_runtime.configured_bindings.scenario.compatibility]
// r[impl lattice.agent_runtime.configured_bindings.scenario.boundary]

pub const CONFIGURED_RUNTIME_SCHEMA: &str = "lattice.configured-agent-runtime.v1";

/// Configuration identity, not executable authenticity, worker ownership or capacity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredBinding {
    pub schema: String,
    pub target: super::super::Target,
    pub revision: String,
}

impl ConfiguredBinding {
    pub fn validate(&self) -> bool {
        self.schema == CONFIGURED_RUNTIME_SCHEMA
            && self.revision.strip_prefix("blake3:").is_some_and(|digest| {
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilitySupport {
    Unsupported,
    Unknown,
}

/// Bounded, material-free projection of an existing route. It grants nothing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConfiguredProfile {
    pub binding: ConfiguredBinding,
    pub adapter: &'static str,
    pub protocol: &'static str,
    pub output_modes: Vec<super::super::OutputMode>,
    pub sessions: CompatibilitySupport,
    pub image_inputs: CompatibilitySupport,
    pub checkpoint: CompatibilitySupport,
    pub model_service_bindings: CompatibilitySupport,
    pub compute_reservations: CompatibilitySupport,
    pub qualified_worker_telemetry: CompatibilitySupport,
    pub deadline_secs: u64,
    pub max_output_bytes: usize,
}

impl super::super::RuntimeOptions {
    /// List only known configured targets; never discover executables or instances.
    /// Ambiguous or invalid routes fail closed rather than advertising support.
    pub fn configured_profiles(&self) -> Result<Vec<ConfiguredProfile>, super::super::Error> {
        [
            super::super::Target::Pi,
            super::super::Target::Clankers,
            super::super::Target::Animus,
        ]
        .into_iter()
        .filter(|target| {
            let key = super::super::target_label(*target);
            self.runtimes.contains_key(key) || self.openai_compatible.contains_key(key)
        })
        .map(|target| self.configured_profile(target))
        .collect()
    }

    pub fn configured_profile(&self, target: super::super::Target) -> Result<ConfiguredProfile, super::super::Error> {
        let request = super::super::Request {
            target,
            prompt: "compatibility projection".to_string(),
            output_mode: super::super::OutputMode::Text,
            model_hint: None,
            tool_policy: String::new(),
            context_refs: Vec::new(),
            cwd: None,
            session: None,
            text_inputs: Vec::new(),
            image_inputs: Vec::new(),
            json_inputs: Vec::new(),
        };
        let (adapter, protocol, output_modes, sessions, image_inputs, deadline_secs) =
            match super::route_for_target(self, target)? {
                super::Route::Command => {
                    let plan = super::plan_command(self, &request)?;
                    (
                        "command",
                        "lattice.command-stdout.v1",
                        vec![
                            super::super::OutputMode::Text,
                            super::super::OutputMode::Json,
                            super::super::OutputMode::Image,
                            super::super::OutputMode::Artifact,
                        ],
                        CompatibilitySupport::Unknown,
                        // Command prompts carry image descriptions, not a qualified image protocol.
                        CompatibilitySupport::Unknown,
                        plan.timeout_secs,
                    )
                }
                super::Route::OpenAi(runtime) | super::Route::Animus(runtime) => {
                    crate::openai::plan_chat(runtime, &request, super::global_output_max_bytes(self)?)?;
                    (
                        if target == super::super::Target::Animus {
                            "animus"
                        } else {
                            "openai-compatible"
                        },
                        "openai.chat-completions.non-streaming",
                        vec![super::super::OutputMode::Text, super::super::OutputMode::Json],
                        CompatibilitySupport::Unsupported,
                        CompatibilitySupport::Unsupported,
                        runtime.timeout_secs,
                    )
                }
            };
        Ok(ConfiguredProfile {
            binding: super::configured_binding(self, target)?,
            adapter,
            protocol,
            output_modes,
            sessions,
            image_inputs,
            checkpoint: CompatibilitySupport::Unknown,
            model_service_bindings: CompatibilitySupport::Unsupported,
            compute_reservations: CompatibilitySupport::Unsupported,
            qualified_worker_telemetry: CompatibilitySupport::Unsupported,
            deadline_secs,
            max_output_bytes: self.max_output_bytes,
        })
    }
}

impl super::super::RuntimeOptions {
    /// `lattice.worker-profile-facts.v1` for the configured routes. Compute
    /// reservations and model-service bindings stay unavailable, executable
    /// bytes stay unknown, and appearing in configuration never qualifies a
    /// route.
    // r[impl lattice.execution_binding.profile_facts.scenario.compatibility]
    pub fn worker_profile_facts(
        &self,
    ) -> Result<Vec<lattice_core::execution_binding::projection::inventory::WorkerProfileFacts>, super::super::Error>
    {
        self.configured_profiles()?.iter().map(configured_facts).collect()
    }
}

fn configured_facts(
    profile: &ConfiguredProfile,
) -> Result<lattice_core::execution_binding::projection::inventory::WorkerProfileFacts, super::super::Error> {
    let outputs = profile.output_modes.iter().map(super::super::OutputMode::as_str).collect::<Vec<_>>().join(",");
    let max_output_bytes = u64::try_from(profile.max_output_bytes)
        .map_err(|_| super::super::Error::PreflightRejected("configured output bound is unaddressable".into()))?;
    debug_assert_eq!(profile.compute_reservations, CompatibilitySupport::Unsupported);
    debug_assert_eq!(profile.model_service_bindings, CompatibilitySupport::Unsupported);
    let unknown = lattice_core::execution_binding::projection::Offered::Unknown;
    let inputs = match profile.image_inputs {
        CompatibilitySupport::Unsupported => {
            lattice_core::execution_binding::projection::Offered::Declared("text-prompt-without-images")
        }
        CompatibilitySupport::Unknown => unknown,
    };
    lattice_core::execution_binding::projection::inventory::project_profile(
        &lattice_core::execution_binding::projection::inventory::ProfileInput {
            owner: lattice_core::execution_binding::projection::inventory::ProfileOwner::ConfiguredRuntime,
            revision: &profile.binding.revision,
            qualification_ref: None,
            adapter: lattice_core::execution_binding::projection::Offered::Declared(profile.adapter),
            protocol_revision: lattice_core::execution_binding::projection::Offered::Declared(profile.protocol),
            executable_identity: unknown,
            inputs,
            outputs: lattice_core::execution_binding::projection::Offered::Declared(&outputs),
            sessions: limit(profile.sessions),
            checkpoint: limit(profile.checkpoint),
            workspace: unknown,
            network: unknown,
            isolation: unknown,
            deadline_secs: lattice_core::execution_binding::projection::Offered::Declared(profile.deadline_secs),
            max_output_bytes: lattice_core::execution_binding::projection::Offered::Declared(max_output_bytes),
            model_routes: lattice_core::execution_binding::projection::Offered::Unavailable,
            compute: lattice_core::execution_binding::projection::Offered::Unavailable,
        },
    )
    .map_err(|defect| super::super::Error::PreflightRejected(format!("configured profile facts: {defect:?}")))
}

/// `Unsupported` stays unavailable and `Unknown` stays unknown.
fn limit(value: CompatibilitySupport) -> lattice_core::execution_binding::projection::Offered<&'static str> {
    match value {
        CompatibilitySupport::Unsupported => lattice_core::execution_binding::projection::Offered::Unavailable,
        CompatibilitySupport::Unknown => lattice_core::execution_binding::projection::Offered::Unknown,
    }
}
