const INLINE_REASONING_OPEN: &str = "<think>";
const INLINE_REASONING_CLOSE: &str = "</think>";
const INLINE_REASONING_DIAGNOSTIC: &str = "OpenAI-compatible response contains ambiguous inline reasoning markup";
const EMPTY_FINAL_CONTENT_DIAGNOSTIC: &str = "OpenAI-compatible response has no final content after inline reasoning";
const TOKEN_LIMIT_FINISH_REASON: &str = "length";
const TOKEN_LIMIT_DIAGNOSTIC: &str = "OpenAI-compatible response exhausted max_tokens before complete final content; increase the configured max_tokens bound";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FinalContent<'a> {
    pub(super) text: &'a str,
    pub(super) reasoning_suppressed: bool,
}

// r[impl lattice.agent_runtime.inline_reasoning_boundary]
pub(super) fn admit_finish_reason(finish_reason: Option<&str>) -> Result<(), crate::agent_node::AgentError> {
    if finish_reason == Some(TOKEN_LIMIT_FINISH_REASON) {
        return Err(crate::agent_node::AgentError::Adapter(TOKEN_LIMIT_DIAGNOSTIC.to_string()));
    }
    Ok(())
}

// r[impl lattice.agent_runtime.inline_reasoning_boundary]
pub(super) fn admit_final_content(content: &str) -> Result<FinalContent<'_>, crate::agent_node::AgentError> {
    let has_open = content.contains(INLINE_REASONING_OPEN);
    let has_close = content.contains(INLINE_REASONING_CLOSE);
    if !has_open && !has_close {
        return Ok(FinalContent {
            text: content,
            reasoning_suppressed: false,
        });
    }
    if !content.starts_with(INLINE_REASONING_OPEN) {
        return Err(boundary_error(INLINE_REASONING_DIAGNOSTIC));
    }

    let after_open = &content[INLINE_REASONING_OPEN.len()..];
    let close_index =
        after_open.find(INLINE_REASONING_CLOSE).ok_or_else(|| boundary_error(INLINE_REASONING_DIAGNOSTIC))?;
    let reasoning = &after_open[..close_index];
    let final_start = close_index
        .checked_add(INLINE_REASONING_CLOSE.len())
        .ok_or_else(|| boundary_error(INLINE_REASONING_DIAGNOSTIC))?;
    let final_content = after_open[final_start..].trim();
    let has_reasoning_marker = reasoning.contains(INLINE_REASONING_OPEN) || reasoning.contains(INLINE_REASONING_CLOSE);
    let has_final_marker =
        final_content.contains(INLINE_REASONING_OPEN) || final_content.contains(INLINE_REASONING_CLOSE);
    if has_reasoning_marker || has_final_marker {
        return Err(boundary_error(INLINE_REASONING_DIAGNOSTIC));
    }
    if final_content.is_empty() {
        return Err(boundary_error(EMPTY_FINAL_CONTENT_DIAGNOSTIC));
    }
    debug_assert!(!final_content.contains(INLINE_REASONING_OPEN));
    debug_assert!(!final_content.contains(INLINE_REASONING_CLOSE));
    Ok(FinalContent {
        text: final_content,
        reasoning_suppressed: true,
    })
}

fn boundary_error(message: &str) -> crate::agent_node::AgentError {
    crate::agent_node::AgentError::PreflightRejected(message.to_string())
}
