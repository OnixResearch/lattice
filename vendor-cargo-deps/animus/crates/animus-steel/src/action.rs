const MODEL_TAG: &str = "model";
const TOOL_TAG: &str = "tool";
const RESPOND_TAG: &str = "respond";
const STOP_TAG: &str = "stop";
const MODEL_ARITY: usize = 2;
const TOOL_ARITY: usize = 9;
const RESPOND_ARITY: usize = 2;
const STOP_ARITY: usize = 1;

/// Next action selected by the Steel loop fabric.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    /// Send a request to a model adapter.
    CallModel {
        /// Validated model request.
        request: animus_adapters::ModelRequest,
    },
    /// Send a request to a tool adapter.
    CallTool {
        /// Validated tool request.
        request: animus_adapters::ToolRequest,
    },
    /// Return a final response from the loop.
    Respond {
        /// Validated response message.
        response: animus_adapters::ModelResponse,
    },
    /// Stop the loop without producing another adapter request.
    Stop,
}

impl Decision {
    pub(crate) fn from_value(value: steel::SteelVal) -> Result<Self, crate::LoopError> {
        let items = list_items(value)?;
        let tag = required_symbol(&items, TAG_INDEX)?;
        match tag {
            MODEL_TAG => from_model_items(&items),
            TOOL_TAG => from_tool_items(&items),
            RESPOND_TAG => from_respond_items(&items),
            STOP_TAG => from_stop_items(&items),
            other => Err(crate::LoopError::InvalidDecision {
                reason: format!("unknown decision tag {other}"),
            }),
        }
    }
}

const TAG_INDEX: usize = 0;
const MODEL_PROMPT_INDEX: usize = 1;
const TOOL_NAME_INDEX: usize = 1;
const TOOL_SCHEMA_INDEX: usize = 2;
const TOOL_ARGUMENTS_INDEX: usize = 3;
const TOOL_CORRELATION_INDEX: usize = 4;
const TOOL_VISIBLE_BUDGET_INDEX: usize = 5;
const TOOL_MUTATION_TARGETS_INDEX: usize = 6;
const TOOL_CAPABILITIES_INDEX: usize = 7;
const TOOL_APPROVAL_INDEX: usize = 8;
const ARGUMENT_NAME_INDEX: usize = 0;
const ARGUMENT_VALUE_INDEX: usize = 1;
const ARGUMENT_PAIR_ARITY: usize = 2;
const RESPONSE_MESSAGE_INDEX: usize = 1;

fn from_model_items(items: &[steel::SteelVal]) -> Result<Decision, crate::LoopError> {
    validate_arity(items, MODEL_TAG, MODEL_ARITY)?;
    let prompt = required_string(items, MODEL_PROMPT_INDEX)?;
    Ok(Decision::CallModel {
        request: animus_adapters::ModelRequest::new(prompt)?,
    })
}

fn from_tool_items(items: &[steel::SteelVal]) -> Result<Decision, crate::LoopError> {
    validate_arity(items, TOOL_TAG, TOOL_ARITY)?;
    let name = required_string(items, TOOL_NAME_INDEX)?;
    let schema_identity = required_string(items, TOOL_SCHEMA_INDEX)?;
    let argument_pairs = required_argument_pairs(items, TOOL_ARGUMENTS_INDEX)?;
    let allowed_field_names = argument_names(&argument_pairs);
    let argument_drafts = argument_drafts(&argument_pairs);
    let arguments = animus_adapters::tool::PreparedArguments::prepare(
        &animus_adapters::tool::ArgumentPreparationRequest {
            current_schema_identity: schema_identity,
            allowed_field_names: allowed_field_names.as_slice(),
            compatibility_rules: &[],
            fields: argument_drafts.as_slice(),
        },
    )
    .map_err(|error| tool_contract_error(&error))?;
    let mutation_targets = required_string_list(items, TOOL_MUTATION_TARGETS_INDEX)?;
    let mutation_target_refs = string_refs(&mutation_targets);
    let capability_labels = required_string_list(items, TOOL_CAPABILITIES_INDEX)?;
    let capability_label_refs = string_refs(&capability_labels);
    let visible_budget = required_visible_budget(items, TOOL_VISIBLE_BUDGET_INDEX)?;
    let request = animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
        name: Some(name),
        arguments: Some(&arguments),
        correlation_id: Some(required_string(items, TOOL_CORRELATION_INDEX)?),
        output_policy: animus_adapters::tool::OutputPolicy {
            visible_char_budget: visible_budget,
        },
        mutation_targets: mutation_target_refs.as_slice(),
        required_capability_labels: capability_label_refs.as_slice(),
        approval_posture: required_string(items, TOOL_APPROVAL_INDEX)?,
    })
    .map_err(|error| tool_contract_error(&error))?;
    Ok(Decision::CallTool { request })
}

fn from_respond_items(items: &[steel::SteelVal]) -> Result<Decision, crate::LoopError> {
    validate_arity(items, RESPOND_TAG, RESPOND_ARITY)?;
    let message = required_string(items, RESPONSE_MESSAGE_INDEX)?;
    Ok(Decision::Respond {
        response: animus_adapters::ModelResponse::new(message)?,
    })
}

fn from_stop_items(items: &[steel::SteelVal]) -> Result<Decision, crate::LoopError> {
    validate_arity(items, STOP_TAG, STOP_ARITY)?;
    Ok(Decision::Stop)
}

fn list_items(value: steel::SteelVal) -> Result<Vec<steel::SteelVal>, crate::LoopError> {
    match value {
        steel::SteelVal::ListV(list) => Ok(list.iter().cloned().collect()),
        other => Err(crate::LoopError::InvalidDecision {
            reason: format!("expected list decision, got {other:?}"),
        }),
    }
}

fn validate_arity(
    items: &[steel::SteelVal],
    tag: &'static str,
    expected: usize,
) -> Result<(), crate::LoopError> {
    let actual = items.len();
    if actual != expected {
        return Err(crate::LoopError::InvalidDecision {
            reason: format!("decision {tag} expected {expected} fields, got {actual}"),
        });
    }

    Ok(())
}

fn required_symbol(items: &[steel::SteelVal], index: usize) -> Result<&str, crate::LoopError> {
    match items.get(index) {
        Some(steel::SteelVal::SymbolV(symbol)) => Ok(symbol.as_str()),
        Some(other) => Err(crate::LoopError::InvalidDecision {
            reason: format!("expected symbol at index {index}, got {other:?}"),
        }),
        None => Err(crate::LoopError::InvalidDecision {
            reason: format!("missing symbol at index {index}"),
        }),
    }
}

fn required_string(items: &[steel::SteelVal], index: usize) -> Result<&str, crate::LoopError> {
    match items.get(index) {
        Some(steel::SteelVal::StringV(text)) => Ok(text.as_str()),
        Some(other) => Err(crate::LoopError::InvalidDecision {
            reason: format!("expected string at index {index}, got {other:?}"),
        }),
        None => Err(crate::LoopError::InvalidDecision {
            reason: format!("missing string at index {index}"),
        }),
    }
}

fn required_visible_budget(
    items: &[steel::SteelVal],
    index: usize,
) -> Result<u32, crate::LoopError> {
    match items.get(index) {
        Some(steel::SteelVal::IntV(value)) => {
            u32::try_from(*value).map_err(|_error| crate::LoopError::InvalidDecision {
                reason: format!("visible budget at index {index} must fit u32"),
            })
        }
        Some(other) => Err(crate::LoopError::InvalidDecision {
            reason: format!("expected integer at index {index}, got {other:?}"),
        }),
        None => Err(crate::LoopError::InvalidDecision {
            reason: format!("missing integer at index {index}"),
        }),
    }
}

fn required_string_list(
    items: &[steel::SteelVal],
    index: usize,
) -> Result<Vec<String>, crate::LoopError> {
    let values = nested_list_items(items, index)?;
    let mut strings = Vec::with_capacity(values.len());
    for (item_index, value) in values.iter().enumerate() {
        match value {
            steel::SteelVal::StringV(text) => strings.push(String::from(text.as_str())),
            other => {
                return Err(crate::LoopError::InvalidDecision {
                    reason: format!(
                        "expected string in list at index {index}/{item_index}, got {other:?}"
                    ),
                });
            }
        }
    }
    Ok(strings)
}

fn required_argument_pairs(
    items: &[steel::SteelVal],
    index: usize,
) -> Result<Vec<(String, String)>, crate::LoopError> {
    let values = nested_list_items(items, index)?;
    let mut pairs = Vec::with_capacity(values.len());
    for (pair_index, value) in values.iter().enumerate() {
        let pair_items = list_items(value.clone())?;
        validate_arity(pair_items.as_slice(), TOOL_TAG, ARGUMENT_PAIR_ARITY)?;
        let name = required_string(pair_items.as_slice(), ARGUMENT_NAME_INDEX)?;
        let argument_value = required_string(pair_items.as_slice(), ARGUMENT_VALUE_INDEX)?;
        if name.trim().is_empty() || argument_value.contains('\0') {
            return Err(crate::LoopError::InvalidDecision {
                reason: format!("invalid argument pair at index {pair_index}"),
            });
        }
        pairs.push((String::from(name), String::from(argument_value)));
    }
    Ok(pairs)
}

fn nested_list_items(
    items: &[steel::SteelVal],
    index: usize,
) -> Result<Vec<steel::SteelVal>, crate::LoopError> {
    match items.get(index) {
        Some(value) => list_items(value.clone()),
        None => Err(crate::LoopError::InvalidDecision {
            reason: format!("missing list at index {index}"),
        }),
    }
}

fn argument_names(argument_pairs: &[(String, String)]) -> Vec<&str> {
    argument_pairs
        .iter()
        .map(|(name, _value)| name.as_str())
        .collect()
}

fn argument_drafts(
    argument_pairs: &[(String, String)],
) -> Vec<animus_adapters::tool::ArgumentFieldDraft<'_>> {
    argument_pairs
        .iter()
        .map(|(name, value)| animus_adapters::tool::ArgumentFieldDraft {
            name: Some(name.as_str()),
            value: Some(value.as_str()),
        })
        .collect()
}

fn string_refs(strings: &[String]) -> Vec<&str> {
    strings.iter().map(String::as_str).collect()
}

fn tool_contract_error(error: &animus_adapters::tool::ContractError) -> crate::LoopError {
    crate::LoopError::InvalidDecision {
        reason: format!("tool request failed current contract: {error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_PROMPT: &str = "intent: build reliable systems";
    const TOOL_NAME: &str = "search";
    const TOOL_SCHEMA: &str = "schema.search.v1";
    const TOOL_ARGUMENT_NAME: &str = "query";
    const TOOL_ARGUMENT_VALUE: &str = "adapter contracts";
    const TOOL_CORRELATION_ID: &str = "tool-call-1";
    const TOOL_VISIBLE_BUDGET: isize = 10;
    const TOOL_MUTATION_TARGET: &str = "resource:file:README";
    const TOOL_CAPABILITY: &str = "tool.read";
    const TOOL_APPROVAL: &str = "pre_approved";
    const LEGACY_TOOL_INPUT: &str = "query";
    const RESPONSE_MESSAGE: &str = "done";
    const BLANK_TEXT: &str = "   ";

    #[test]
    fn parses_model_decision() {
        let value = steel::list!(steel::SteelVal::SymbolV(MODEL_TAG.into()), MODEL_PROMPT);
        let decision = Decision::from_value(value).expect("model decision should parse");

        assert!(matches!(decision, Decision::CallModel { .. }));
    }

    #[test]
    fn parses_tool_decision() {
        let value = current_tool_value();
        let decision = Decision::from_value(value).expect("tool decision should parse");

        assert!(matches!(decision, Decision::CallTool { .. }));
    }

    #[test]
    fn rejects_legacy_tool_decision_shape() {
        // r[verify agent_harness_runtime.tool_calling_breaking.current_shape.rejects_legacy]
        let value = steel::list!(
            steel::SteelVal::SymbolV(TOOL_TAG.into()),
            TOOL_NAME,
            LEGACY_TOOL_INPUT
        );
        let error = Decision::from_value(value).expect_err("legacy raw tool should fail");

        assert!(matches!(error, crate::LoopError::InvalidDecision { .. }));
    }

    #[test]
    fn parses_response_decision() {
        let value = steel::list!(
            steel::SteelVal::SymbolV(RESPOND_TAG.into()),
            RESPONSE_MESSAGE
        );
        let decision = Decision::from_value(value).expect("response decision should parse");

        assert!(matches!(decision, Decision::Respond { .. }));
    }

    #[test]
    fn parses_stop_decision() {
        let value = steel::list!(steel::SteelVal::SymbolV(STOP_TAG.into()));
        let decision = Decision::from_value(value).expect("stop decision should parse");

        assert_eq!(decision, Decision::Stop);
    }

    #[test]
    fn rejects_invalid_shape() {
        let error = Decision::from_value(steel::SteelVal::BoolV(true))
            .expect_err("non-list decision should fail");

        assert!(matches!(error, crate::LoopError::InvalidDecision { .. }));
    }

    #[test]
    fn rejects_invalid_payload() {
        let value = steel::list!(steel::SteelVal::SymbolV(MODEL_TAG.into()), BLANK_TEXT);
        let error = Decision::from_value(value).expect_err("blank model prompt should fail");

        assert!(matches!(error, crate::LoopError::AdapterContract { .. }));
    }

    fn current_tool_value() -> steel::SteelVal {
        steel::list!(
            steel::SteelVal::SymbolV(TOOL_TAG.into()),
            TOOL_NAME,
            TOOL_SCHEMA,
            steel::list!(steel::list!(TOOL_ARGUMENT_NAME, TOOL_ARGUMENT_VALUE)),
            TOOL_CORRELATION_ID,
            steel::SteelVal::IntV(TOOL_VISIBLE_BUDGET),
            steel::list!(TOOL_MUTATION_TARGET),
            steel::list!(TOOL_CAPABILITY),
            TOOL_APPROVAL
        )
    }
}
