//! Pure admission of untrusted import-free component output.
//!
//! An admitted intent is still only a proposal. Existing operation planning,
//! credential, authority and durable effect owners must admit it separately.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

// r[impl lattice.operation_packs_wasm.world]
// r[impl lattice.operation_packs_wasm.effects]
// r[impl lattice.operation_packs_wasm.nonclaims]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectIntent {
    pub type_id: String,
    pub type_revision: String,
    pub instance_id: String,
    pub method: String,
    pub arguments: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Output {
    PureResult {
        schema: String,
        result: serde_json::Value,
    },
    EffectIntents {
        schema: String,
        intents: Vec<EffectIntent>,
        result: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedOutput {
    result: serde_json::Value,
    intents: Vec<EffectIntent>,
}

impl AdmittedOutput {
    pub fn result(&self) -> &serde_json::Value {
        &self.result
    }

    pub fn intents(&self) -> &[EffectIntent] {
        &self.intents
    }

    /// Consume the result only after separately observed effects succeeded.
    pub fn into_result(self) -> serde_json::Value {
        self.result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocker {
    Plan,
    Size,
    Encoding,
    Schema,
    ProhibitedContent,
    ResultContract,
    IntentCount,
    IntentTarget,
    IntentIdentity,
    IntentArguments,
}

/// Validate bounded output without resolving credentials or performing effects.
pub fn admit(
    bytes: &[u8],
    admitted: &super::model::AdmittedPack,
    descriptor: &super::wasm::Descriptor,
    plan: &crate::operation::InvocationPlan,
    prohibited_fragments: &[String],
) -> Result<AdmittedOutput, Blocker> {
    crate::operation::planning::validate_plan_identity(plan).map_err(|_| Blocker::Plan)?;
    super::admission::validate_lock(&admitted.candidate, &admitted.lock).map_err(|_| Blocker::Plan)?;
    let runtime = admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.kind == super::wasm::RUNTIME_KIND && runtime.wasm.as_ref() == Some(descriptor))
        .ok_or(Blocker::Plan)?;
    if !admitted.candidate.manifest.operations.iter().any(|operation| {
        operation.runtime_id == runtime.id
            && operation.type_id == plan.type_id
            && operation.type_revision == plan.type_revision
            && operation.methods.contains(&plan.method)
    }) || plan.capabilities.iter().any(|capability| !runtime.capabilities.contains(capability))
    {
        return Err(Blocker::Plan);
    }
    if bytes.is_empty()
        || bytes.len() > plan.bounds.output_bytes
        || u64::try_from(bytes.len()).map_err(|_| Blocker::Size)? > descriptor.resources.maximum_output_bytes
    {
        return Err(Blocker::Size);
    }
    if prohibited_fragments
        .iter()
        .any(|fragment| !fragment.is_empty() && bytes.windows(fragment.len()).any(|part| part == fragment.as_bytes()))
    {
        return Err(Blocker::ProhibitedContent);
    }
    let output: Output = serde_json::from_slice(bytes).map_err(|_| Blocker::Encoding)?;
    let (result, intents) = match output {
        Output::PureResult { schema, result } => {
            if schema != super::wasm::RESULT_SCHEMA {
                return Err(Blocker::Schema);
            }
            (result, Vec::new())
        }
        Output::EffectIntents {
            schema,
            intents,
            result,
        } => {
            if schema != super::wasm::EFFECT_INTENT_SCHEMA {
                return Err(Blocker::Schema);
            }
            if intents.is_empty() || intents.len() > descriptor.maximum_effect_intents {
                return Err(Blocker::IntentCount);
            }
            (result, intents)
        }
    };
    if crate::operation::runtime::output_contains_prohibited_field(&result) {
        return Err(Blocker::ProhibitedContent);
    }
    if !crate::operation::schema::validate_value(
        &plan.output_contract,
        &result,
        "component.result",
        crate::operation::DiagnosticCode::OutputRejected,
    )
    .is_empty()
    {
        return Err(Blocker::ResultContract);
    }
    for intent in &intents {
        admit_intent(intent, admitted, descriptor, plan)?;
    }
    Ok(AdmittedOutput { result, intents })
}

fn admit_intent(
    intent: &EffectIntent,
    admitted: &super::model::AdmittedPack,
    descriptor: &super::wasm::Descriptor,
    plan: &crate::operation::InvocationPlan,
) -> Result<(), Blocker> {
    if !crate::operation::identity::valid_identifier(&intent.instance_id)
        || !crate::operation::identity::valid_identifier(&intent.method)
        || !crate::operation::identity::validate(&intent.type_revision)
    {
        return Err(Blocker::IntentIdentity);
    }
    if !descriptor
        .effect_intent_targets
        .iter()
        .any(|target| target.type_id == intent.type_id && target.method == intent.method)
    {
        return Err(Blocker::IntentTarget);
    }
    let operation = admitted
        .candidate
        .manifest
        .operations
        .iter()
        .find(|operation| {
            operation.type_id == intent.type_id
                && operation.type_revision == intent.type_revision
                && operation.methods.contains(&intent.method)
        })
        .ok_or(Blocker::IntentTarget)?;
    let runtime = admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| {
            runtime.id == operation.runtime_id && runtime.kind == "bounded_process" && runtime.wasm.is_none()
        })
        .ok_or(Blocker::IntentTarget)?;
    if runtime.capabilities.iter().any(|capability| !plan.capabilities.contains(capability)) {
        return Err(Blocker::IntentTarget);
    }
    let arguments = serde_json::to_vec(&intent.arguments).map_err(|_| Blocker::IntentArguments)?;
    if arguments.len() > runtime.input_bytes
        || arguments.len() > plan.bounds.input_bytes
        || intent.arguments.iter().any(|(key, value)| {
            crate::operation::schema::key_is_secret_bearing(key)
                || crate::operation::runtime::output_contains_prohibited_field(value)
        })
    {
        return Err(Blocker::IntentArguments);
    }
    Ok(())
}
