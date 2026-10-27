const METRIC_NAME: &str = "runtime.events";
const METRIC_LABEL_KEY: &str = "event_kind";
const METRIC_LABEL_VALUE: &str = "runtime_started";
const METRIC_VALUE: u64 = 1;
const UNSAFE_LABEL_VALUE: &str = "token=secret";
const SAFE_RECEIPT_LABEL: &str = "adapter-family-split";

#[test]
fn observability_types_match_facade() -> Result<(), crate::ObservabilityError> {
    let labels = [animus_adapter_observability::MetricLabelDraft::new(
        METRIC_LABEL_KEY,
        METRIC_LABEL_VALUE,
    )];
    let direct = animus_adapter_observability::MetricSample::new(
        animus_adapter_observability::MetricSampleDraft {
            name: METRIC_NAME,
            kind: animus_adapter_observability::MetricKind::Counter,
            unit: animus_adapter_observability::MetricUnit::Count,
            value: METRIC_VALUE,
            labels: &labels,
        },
    )?;
    let facade: crate::MetricSample = direct.clone();
    let direct_ref: &animus_adapter_observability::MetricSample = &facade;

    assert_eq!(facade.name(), direct_ref.name());
    assert_eq!(
        facade.kind(),
        animus_adapter_observability::MetricKind::Counter
    );
    assert_eq!(
        facade.unit(),
        animus_adapter_observability::MetricUnit::Count
    );
    assert_eq!(facade.value(), METRIC_VALUE);
    Ok(())
}

#[test]
fn unsafe_metric_labels_are_rejected() {
    let labels = [crate::MetricLabelDraft::new(
        METRIC_LABEL_KEY,
        UNSAFE_LABEL_VALUE,
    )];
    assert!(matches!(
        crate::MetricSample::new(crate::MetricSampleDraft {
            name: METRIC_NAME,
            kind: crate::MetricKind::Gauge,
            unit: crate::MetricUnit::Count,
            value: METRIC_VALUE,
            labels: &labels,
        }),
        Err(crate::ObservabilityError::UnsafeMetadata(_))
    ));
}

#[test]
fn receipt_constants_match_facade() -> Result<(), animus_adapter_safe::Error> {
    assert_eq!(crate::SAFE_RECEIPT_SCHEMA, animus_adapter_safe::SCHEMA);
    assert_eq!(
        crate::MAX_SAFE_RECEIPT_LABEL_CHARS,
        animus_adapter_safe::MAX_LABEL_CHARS
    );
    assert_eq!(
        crate::MAX_SAFE_RECEIPT_COUNT,
        animus_adapter_safe::MAX_COUNT
    );

    let receipt = animus_adapter_safe::Record::new(
        animus_adapter_safe::Draft::new(
            animus_adapter_safe::Status::Completed,
            animus_adapter_safe::IssueCode::None,
            animus_adapter_safe::Operation::ProvenanceObservation,
        )
        .with_labels(&[SAFE_RECEIPT_LABEL])
        .with_redactions(&[animus_adapter_safe::Redaction::PayloadHashOnly]),
    )?;
    assert_eq!(receipt.schema(), crate::SAFE_RECEIPT_SCHEMA);
    assert!(receipt.review_summary().contains(SAFE_RECEIPT_LABEL));
    Ok(())
}

#[test]
fn unsafe_receipt_labels_are_rejected() {
    assert!(matches!(
        animus_adapter_safe::Record::new(
            animus_adapter_safe::Draft::new(
                animus_adapter_safe::Status::Blocked,
                animus_adapter_safe::IssueCode::LeakRejected,
                animus_adapter_safe::Operation::RuntimeBlock,
            )
            .with_labels(&[UNSAFE_LABEL_VALUE])
            .with_redactions(&[animus_adapter_safe::Redaction::CredentialOmitted]),
        ),
        Err(animus_adapter_safe::Error::UnsafeMetadata { .. })
    ));
}

#[test]
fn rlm_kernel_types_match_facade() {
    let direct = animus_adapter_rlm_kernel::HostOperationClass::ContextRead;
    let facade: crate::rlm_kernel::HostOperationClass = direct;
    assert_eq!(facade.as_str(), "context_read");
    assert_eq!(
        crate::rlm_kernel::STEEL_REPL_TOOL_NAME,
        animus_adapter_rlm_kernel::STEEL_REPL_TOOL_NAME
    );
    assert_eq!(
        crate::rlm_kernel::RLM_KERNEL_POLICY_SCHEMA,
        animus_adapter_rlm_kernel::RLM_KERNEL_POLICY_SCHEMA
    );
}
