/// Completes a runtime report after Steel policy denial.
#[must_use]
pub fn complete_with_steel_policy_denial(
    state: super::TurnState,
    receipt: &animus_steel::SteelPolicyActivationReceipt,
) -> crate::RunReport {
    let reason = receipt.denied_reason().map_or_else(
        || String::from("unknown"),
        |reason| String::from(reason.as_str()),
    );
    let message = receipt.denial_message();
    let state = state.with_event(crate::RuntimeEvent::SteelPolicyDenied {
        profile: String::from(receipt.profile()),
        script_hash: String::from(receipt.script_hash()),
        receipt_hash: String::from(receipt.receipt_hash()),
        reason,
    });
    state.complete(crate::RuntimeStatus::Failed {
        failure: crate::RuntimeFailure::SteelPolicy { message },
    })
}
