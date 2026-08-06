use super::*;

#[test]
fn exact_prefix_and_caller_order_policies_are_explicit() {
    for (policy, query, expected) in [
        (PromptCompletionPolicy::Exact, "Settings", 1),
        (PromptCompletionPolicy::Prefix, "Open", 1),
        (PromptCompletionPolicy::CallerOrder, "unmatched", 3),
    ] {
        let configured = PromptConfiguration {
            completion: policy,
            ..super::configuration()
        };
        let original =
            build_prompt_session(PromptSessionId::new("policy"), "Policy", None, configured).expect("valid policy");
        let with_candidates = reduce_prompt(&original, PromptMessage::ReplaceCandidates(super::candidates(1))).session;
        let queried = reduce_prompt(&with_candidates, PromptMessage::ReplaceInput {
            input: query.to_string(),
            expected_revision: PromptInputRevision(0),
        });
        assert!(queried.accepted);
        assert_eq!(project_prompt(&queried.session).candidates.len(), expected);
    }
}

#[test]
fn free_text_and_cancel_outputs_are_non_authorizing() {
    let configured = PromptConfiguration {
        completion: PromptCompletionPolicy::None,
        history: PromptHistoryPolicy::Disabled,
        acceptance: PromptAcceptancePolicy::FreeText,
        ..configuration()
    };
    let original = build_prompt_session(PromptSessionId::new("capture"), "Capture", None, configured)
        .expect("valid free text prompt");
    let input = reduce_prompt(&original, PromptMessage::ReplaceInput {
        input: "  note  ".to_string(),
        expected_revision: PromptInputRevision(0),
    })
    .session;
    let submitted = reduce_prompt(&input, PromptMessage::Submit {
        input_revision: PromptInputRevision(1),
        candidate_generation: None,
    });
    assert!(matches!(submitted.output, Some(PromptOutput::FreeText { ref value, .. }) if value == "note"));
    let cancelled = reduce_prompt(&original, PromptMessage::Cancel);
    assert!(matches!(cancelled.output, Some(PromptOutput::Cancelled { .. })));
    assert_eq!(cancelled.session.lifecycle(), PromptLifecycle::ClosePending);
}
