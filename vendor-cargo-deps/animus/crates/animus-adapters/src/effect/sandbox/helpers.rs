fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, crate::AdapterContractError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or_else(|| {
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: field_name.as_str(),
            value: normalized,
        }
    })
}

fn required_unique_fields(
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, crate::AdapterContractError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: WRITABLE_ROOTS_FIELD.as_str(),
        });
    }
    let mut normalized = alloc::vec::Vec::with_capacity(values.len());
    for value in values {
        let item = crate::validation::normalized_adapter_field(WRITABLE_ROOT_FIELD, value)?;
        if normalized.contains(&item) {
            return Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
                field: WRITABLE_ROOT_FIELD.as_str(),
                value: item,
            });
        }
        normalized.push(item);
    }
    Ok(normalized)
}

#[derive(Clone, Copy)]
struct DescriptorConflictInput {
    workdir: SandboxWorkdirMode,
    cow: SandboxCowDisposition,
    network: SandboxNetworkPolicy,
    http: SandboxHttpPolicy,
}

fn reject_descriptor_conflicts(
    input: DescriptorConflictInput,
) -> Result<(), crate::AdapterContractError> {
    if input.cow != SandboxCowDisposition::Disabled
        && input.workdir != SandboxWorkdirMode::CowStaged
    {
        return Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: COW_FIELD.as_str(),
            value: alloc::string::String::from(input.cow.as_str()),
        });
    }
    if input.network == SandboxNetworkPolicy::DenyAll && input.http != SandboxHttpPolicy::None {
        return Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: HTTP_FIELD.as_str(),
            value: alloc::string::String::from(input.http.as_str()),
        });
    }
    Ok(())
}

struct DescriptorHashInput<'a> {
    descriptor_id: &'a str,
    filesystem: SandboxFilesystemMode,
    writable_roots: &'a [alloc::string::String],
    workdir: SandboxWorkdirMode,
    cow: SandboxCowDisposition,
    process: SandboxProcessProfile,
    environment: SandboxEnvironmentPolicy,
    network: SandboxNetworkPolicy,
    http: SandboxHttpPolicy,
    determinism: SandboxDeterminismPolicy,
    enforcement: SandboxEnforcementRequirement,
}

fn descriptor_hash(input: &DescriptorHashInput<'_>) -> super::SafeHash {
    let roots = input.writable_roots.join(",");
    safe_hash_parts(&[
        DESCRIPTOR_SCHEMA,
        input.descriptor_id,
        input.filesystem.as_str(),
        roots.as_str(),
        input.workdir.as_str(),
        input.cow.as_str(),
        input.process.as_str(),
        input.environment.as_str(),
        input.network.as_str(),
        input.http.as_str(),
        input.determinism.as_str(),
        input.enforcement.as_str(),
    ])
}

fn denied(
    issue: SandboxDispatchIssue,
    unsupported_capabilities: alloc::vec::Vec<SandboxPlatformCapability>,
) -> SandboxDispatchDecision {
    SandboxDispatchDecision {
        status: SandboxDispatchStatus::Denied,
        issue,
        unsupported_capabilities,
    }
}

struct ReceiptHashInput<'a> {
    descriptor: &'a SandboxDescriptor,
    decision: &'a SandboxDispatchDecision,
    platform: &'a SandboxPlatformSupport,
    cow_summary_count: u32,
    output_hash: Option<&'a super::SafeHash>,
}

fn receipt_hash(input: &ReceiptHashInput<'_>) -> super::SafeHash {
    let output_hash = input
        .output_hash
        .map_or(EMPTY_LABEL, super::SafeHash::as_str);
    let cow_summary = input.cow_summary_count.to_string();
    safe_hash_parts(&[
        RECEIPT_SCHEMA,
        input.descriptor.descriptor_hash().as_str(),
        input.decision.status().as_str(),
        input.decision.issue().as_str(),
        input.platform.platform_id(),
        cow_summary.as_str(),
        output_hash,
    ])
}

fn safe_hash_parts(parts: &[&str]) -> super::SafeHash {
    let mut canonical = alloc::string::String::new();
    for part in parts {
        canonical.push(RECEIPT_PART_SEPARATOR);
        canonical.push_str(part);
    }
    super::SafeHash::from_blake3_bytes(canonical.as_bytes())
}

fn first_leak(text: &str) -> Option<SandboxReceiptLeak> {
    LEAK_MARKERS
        .iter()
        .find(|(_leak, markers)| markers.iter().any(|marker| text.contains(marker)))
        .map(|(leak, _markers)| *leak)
}

const LEAK_MARKERS: &[(SandboxReceiptLeak, &[&str])] = &[
    (
        SandboxReceiptLeak::RawPrompt,
        &["RAW_PROMPT:", "raw prompt"],
    ),
    (SandboxReceiptLeak::Credential, &["API_KEY=", "SECRET="]),
    (SandboxReceiptLeak::FullPath, &["/home/", "/private/"]),
    (
        SandboxReceiptLeak::RawNetworkPayload,
        &["RAW_NETWORK:", "HTTP BODY:"],
    ),
    (
        SandboxReceiptLeak::RawFileContent,
        &["RAW_FILE:", "file body:"],
    ),
    (
        SandboxReceiptLeak::RawProcessOutput,
        &["RAW_PROCESS_OUTPUT:", "stdout body:"],
    ),
    (
        SandboxReceiptLeak::UnboundedEventStream,
        &["UNBOUNDED_EVENTS:", "event stream body"],
    ),
];
