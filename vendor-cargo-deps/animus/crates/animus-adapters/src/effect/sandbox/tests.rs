#[cfg(test)]
mod tests {
    use super::*;

    const DESCRIPTOR_ID: &str = "sandbox:linux-dry-run";
    const ROOT_SRC: &str = "src";
    const ROOT_TARGET: &str = "target/tmp";
    const POLICY_ID: &str = "policy:sandbox-standard";
    const PLATFORM_ID: &str = "linux:sandlock-inspired";
    const OUTPUT_BODY: &[u8] = b"bounded output summary";
    const EXPECTED_WRITABLE_ROOTS: usize = 2;
    const EXPECTED_BASE_CAPABILITIES: usize = 6;
    const COW_SUMMARY_COUNT: u32 = 1;
    const COW_CAPABILITY_DELTA: usize = 1;
    const LEAK_CASES: usize = 7;

    #[test]
    fn descriptor_normalizes_portable_intent_without_kernel_details() {
        // r[verify agent_harness_runtime.sandlock_sandbox.descriptor]
        // r[verify agent_harness_runtime.sandlock_sandbox.normalization]
        let descriptor = valid_descriptor();

        assert_eq!(descriptor.descriptor_id(), DESCRIPTOR_ID);
        assert_eq!(
            descriptor.filesystem_mode(),
            SandboxFilesystemMode::WorkspaceWrite
        );
        assert_eq!(descriptor.workdir_mode(), SandboxWorkdirMode::CowStaged);
        assert_eq!(descriptor.cow_disposition(), SandboxCowDisposition::DryRun);
        assert_eq!(descriptor.writable_roots().len(), EXPECTED_WRITABLE_ROOTS);
        assert_eq!(
            descriptor.enforcement_requirement(),
            SandboxEnforcementRequirement::Required
        );
        assert!(!descriptor.descriptor_hash().as_str().is_empty());
    }

    #[test]
    fn policy_platform_dispatch_and_receipt_succeed_for_linux_capability_matrix() {
        // r[verify agent_harness_runtime.sandlock_sandbox.policy]
        // r[verify agent_harness_runtime.sandlock_sandbox.platform]
        // r[verify agent_harness_runtime.sandlock_sandbox.fail_closed]
        // r[verify agent_harness_runtime.sandlock_sandbox.receipts]
        // r[verify agent_harness_runtime.sandlock_sandbox.verification]
        let descriptor = valid_descriptor();
        let policy = allowing_policy(&[
            CLASS_FILESYSTEM,
            CLASS_WORKDIR_COW,
            CLASS_PROCESS,
            CLASS_ENVIRONMENT,
            CLASS_NETWORK,
        ]);
        let platform = platform(&[
            CAP_LANDLOCK_FILESYSTEM,
            CAP_SECCOMP_BPF,
            CAP_COW_STAGING,
            CAP_RESOURCE_LIMITS,
            CAP_NETWORK_ACL,
        ]);
        let decision = evaluate_sandbox_dispatch(&descriptor, &policy, &platform);
        let output_hash = super::super::SafeHash::from_blake3_bytes(OUTPUT_BODY);
        let receipt = SandboxDispatchReceipt::new(SandboxDispatchReceiptDraft {
            descriptor: &descriptor,
            decision: &decision,
            platform: &platform,
            cow_summary_count: COW_SUMMARY_COUNT,
            output_hash: Some(&output_hash),
        });

        assert!(decision.dispatch_allowed());
        assert_eq!(decision.issue(), SandboxDispatchIssue::None);
        assert_eq!(receipt.status(), SandboxDispatchStatus::Ready);
        assert_eq!(receipt.descriptor_hash(), descriptor.descriptor_hash());
        assert_eq!(receipt.cow_summary_count(), COW_SUMMARY_COUNT);
        assert_eq!(receipt.output_hash(), Some(&output_hash));
        let expected_capabilities = EXPECTED_BASE_CAPABILITIES
            .checked_sub(COW_CAPABILITY_DELTA)
            .expect("fixture capability count should not underflow");
        assert_eq!(platform.capabilities().len(), expected_capabilities);
    }

    #[test]
    fn descriptor_rejects_conflicting_network_and_cow_rules() {
        // r[verify agent_harness_runtime.sandlock_sandbox.normalization.conflict]
        // r[verify agent_harness_runtime.sandlock_sandbox.verification]
        let http_conflict = SandboxDescriptor::new(SandboxDescriptorDraft {
            network_policy: NETWORK_DENY_ALL,
            http_policy: HTTP_METHOD_HOST_PATH,
            ..valid_descriptor_draft()
        })
        .expect_err("HTTP allowlist cannot coexist with deny-all network mode");
        let cow_conflict = SandboxDescriptor::new(SandboxDescriptorDraft {
            workdir_mode: WORKDIR_HOST,
            cow_disposition: COW_COMMIT,
            ..valid_descriptor_draft()
        })
        .expect_err("COW commit requires COW-staged workdir");

        assert!(matches!(
            http_conflict,
            crate::AdapterContractError::UnsupportedWorkspaceValue {
                field: "sandbox_http_policy",
                ..
            }
        ));
        assert!(matches!(
            cow_conflict,
            crate::AdapterContractError::UnsupportedWorkspaceValue {
                field: "sandbox_cow_disposition",
                ..
            }
        ));
    }

    #[test]
    fn policy_denial_and_partial_platform_fail_required_dispatch_before_host_execution() {
        // r[verify agent_harness_runtime.sandlock_sandbox.policy.denied]
        // r[verify agent_harness_runtime.sandlock_sandbox.fail_closed.partial]
        // r[verify agent_harness_runtime.sandlock_sandbox.verification]
        let descriptor = valid_descriptor();
        let denied_policy = allowing_policy(&[CLASS_FILESYSTEM, CLASS_PROCESS, CLASS_ENVIRONMENT]);
        let full_platform = platform(&[
            CAP_LANDLOCK_FILESYSTEM,
            CAP_SECCOMP_BPF,
            CAP_COW_STAGING,
            CAP_RESOURCE_LIMITS,
            CAP_NETWORK_ACL,
        ]);
        let partial_platform = platform(&[CAP_LANDLOCK_FILESYSTEM, CAP_SECCOMP_BPF]);
        let policy_decision =
            evaluate_sandbox_dispatch(&descriptor, &denied_policy, &full_platform);
        let platform_decision =
            evaluate_sandbox_dispatch(&descriptor, &allow_all_policy(), &partial_platform);

        assert!(!policy_decision.dispatch_allowed());
        assert_eq!(policy_decision.issue(), SandboxDispatchIssue::PolicyDenied);
        assert!(!platform_decision.dispatch_allowed());
        assert_eq!(
            platform_decision.issue(),
            SandboxDispatchIssue::UnsupportedPlatformCapability
        );
        assert!(!platform_decision.unsupported_capabilities().is_empty());
    }

    #[test]
    fn receipt_validation_rejects_sensitive_sandbox_evidence() {
        // r[verify agent_harness_runtime.sandlock_sandbox.receipts.rejects_leaks]
        let cases = [
            ("RAW_PROMPT: hidden", SandboxReceiptLeak::RawPrompt),
            ("API_KEY=secret", SandboxReceiptLeak::Credential),
            ("/home/user/private", SandboxReceiptLeak::FullPath),
            ("RAW_NETWORK: body", SandboxReceiptLeak::RawNetworkPayload),
            ("RAW_FILE: contents", SandboxReceiptLeak::RawFileContent),
            (
                "RAW_PROCESS_OUTPUT: stdout",
                SandboxReceiptLeak::RawProcessOutput,
            ),
            (
                "UNBOUNDED_EVENTS: stream",
                SandboxReceiptLeak::UnboundedEventStream,
            ),
        ];

        assert_eq!(cases.len(), LEAK_CASES);
        for (text, expected) in cases {
            assert_eq!(validate_sandbox_receipt_text(text), Err(expected));
        }
        assert!(validate_sandbox_receipt_text("sandbox=ready descriptor_hash=redacted").is_ok());
    }

    fn valid_descriptor() -> SandboxDescriptor {
        SandboxDescriptor::new(valid_descriptor_draft())
            .expect("descriptor fixture should validate")
    }

    fn valid_descriptor_draft() -> SandboxDescriptorDraft<'static> {
        SandboxDescriptorDraft {
            descriptor_id: DESCRIPTOR_ID,
            filesystem_mode: FILESYSTEM_WORKSPACE_WRITE,
            writable_roots: &[ROOT_SRC, ROOT_TARGET],
            workdir_mode: WORKDIR_COW_STAGED,
            cow_disposition: COW_DRY_RUN,
            process_profile: PROCESS_RESOURCE_LIMITED,
            environment_policy: ENV_MINIMAL,
            network_policy: NETWORK_EGRESS_ALLOWLIST,
            http_policy: HTTP_NONE,
            determinism_policy: DETERMINISM_HOST,
            enforcement_requirement: ENFORCEMENT_REQUIRED,
        }
    }

    fn allow_all_policy() -> SandboxPolicy {
        allowing_policy(&[
            CLASS_FILESYSTEM,
            CLASS_WORKDIR_COW,
            CLASS_PROCESS,
            CLASS_ENVIRONMENT,
            CLASS_NETWORK,
            CLASS_HTTP,
            CLASS_DETERMINISM,
        ])
    }

    fn allowing_policy(classes: &'static [&'static str]) -> SandboxPolicy {
        SandboxPolicy::new(SandboxPolicyDraft {
            policy_id: POLICY_ID,
            allowed_constraint_classes: classes,
        })
        .expect("policy fixture should validate")
    }

    fn platform(capabilities: &'static [&'static str]) -> SandboxPlatformSupport {
        SandboxPlatformSupport::new(SandboxPlatformSupportDraft {
            platform_id: PLATFORM_ID,
            capabilities,
        })
        .expect("platform fixture should validate")
    }
}
