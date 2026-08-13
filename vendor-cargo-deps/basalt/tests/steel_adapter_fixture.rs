use std::collections::BTreeMap;

use basalt::CapabilityGrant;
use basalt::ContractEffectClass;
use basalt::ContractEnvelope;
use basalt::SteelEvaluationReceipt;
use basalt::SteelEvaluationRequest;
use basalt::steel_receipt_hash;
use basalt::steel_request_hash;
use basalt::validate_steel_evaluation_receipt;
use basalt::validate_steel_evaluation_request;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureHostAction {
    None,
    Filesystem,
    Process,
    Network,
    Credential,
    Mutation,
    UndeclaredCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiptMutation {
    None,
    DriftRequestHash,
    DriftSourceHash,
    OmitAuthorityEvidence,
    MalformedBackend,
}

#[derive(Debug)]
struct FixtureSteelAdapter {
    action: FixtureHostAction,
    mutation: ReceiptMutation,
}

impl FixtureSteelAdapter {
    fn evaluate(&self, request: &SteelEvaluationRequest) -> SteelEvaluationReceipt {
        if self.action != FixtureHostAction::None {
            return SteelEvaluationReceipt {
                backend: "steel".to_owned(),
                contract_id: request.envelope.contract_id.clone(),
                normalized_source_hash: request.envelope.normalized_source_hash.clone(),
                request_hash: steel_request_hash(request),
                allowed: false,
                reason: format!("blocked host action: {:?}", self.action),
                caveat_evidence: BTreeMap::from([("blocked_host_action".to_owned(), format!("{:?}", self.action))]),
                effect_class: ContractEffectClass::HostEffect,
                required_capabilities: Vec::new(),
                checked_capabilities: Vec::new(),
                receipt_hash: None,
            };
        }

        let capability = CapabilityGrant::new("did:key:z6Mkresource/docs/report", "document/write");
        let mut receipt = SteelEvaluationReceipt {
            backend: if self.mutation == ReceiptMutation::MalformedBackend {
                "lua"
            } else {
                "steel"
            }
            .to_owned(),
            contract_id: request.envelope.contract_id.clone(),
            normalized_source_hash: if self.mutation == ReceiptMutation::DriftSourceHash {
                "blake3:drifted-source"
            } else {
                request.envelope.normalized_source_hash.as_str()
            }
            .to_owned(),
            request_hash: if self.mutation == ReceiptMutation::DriftRequestHash {
                "blake3:drifted-request".to_owned()
            } else {
                steel_request_hash(request)
            },
            allowed: true,
            reason: "allowed".to_owned(),
            caveat_evidence: BTreeMap::from([("predicate".to_owned(), "passed".to_owned())]),
            effect_class: ContractEffectClass::ReadsDeclaredCapabilities,
            required_capabilities: vec![capability.clone()],
            checked_capabilities: if self.mutation == ReceiptMutation::OmitAuthorityEvidence {
                Vec::new()
            } else {
                vec![capability]
            },
            receipt_hash: None,
        };
        receipt.receipt_hash = Some(steel_receipt_hash(&receipt));
        receipt
    }
}

fn request() -> SteelEvaluationRequest {
    SteelEvaluationRequest {
        envelope: ContractEnvelope::new(
            "steel",
            "document-write-predicate",
            "v1",
            "blake3:steel-source",
            "ucan.contract.input.v1",
            "ucan.contract.output.v1",
            "ucan.contract.receipt.v1",
        ),
        input: serde_json::json!({"resource":"did:key:z6Mkresource/docs/report"}),
        max_input_bytes: 256,
        callable: None,
        requested_host_capability: None,
    }
}

const fn valid_adapter() -> FixtureSteelAdapter {
    FixtureSteelAdapter {
        action: FixtureHostAction::None,
        mutation: ReceiptMutation::None,
    }
}

fn rust_accepts_adapter_receipt(request: &SteelEvaluationRequest, receipt: &SteelEvaluationReceipt) -> bool {
    validate_steel_evaluation_request(request).is_accepted()
        && receipt.request_hash == steel_request_hash(request)
        && receipt.normalized_source_hash == request.envelope.normalized_source_hash
        && validate_steel_evaluation_receipt(receipt).is_accepted()
}

#[test]
fn steel_adapter_fixture_happy_path_is_rust_validated() {
    let request = request();
    let receipt = valid_adapter().evaluate(&request);

    assert!(rust_accepts_adapter_receipt(&request, &receipt));
}

#[test]
fn steel_adapter_fixture_blocks_ambient_host_authority() {
    for action in [
        FixtureHostAction::Filesystem,
        FixtureHostAction::Process,
        FixtureHostAction::Network,
        FixtureHostAction::Credential,
        FixtureHostAction::Mutation,
        FixtureHostAction::UndeclaredCapability,
    ] {
        let mut adapter = valid_adapter();
        adapter.action = action;
        let request = request();
        let receipt = adapter.evaluate(&request);

        assert!(!receipt.allowed, "{action:?}");
        assert!(receipt.reason.contains("blocked host action"), "{action:?}");
    }
}

#[test]
fn steel_adapter_fixture_rejects_malformed_or_drifted_receipts() {
    for mutation in [
        ReceiptMutation::DriftRequestHash,
        ReceiptMutation::DriftSourceHash,
        ReceiptMutation::OmitAuthorityEvidence,
        ReceiptMutation::MalformedBackend,
    ] {
        let mut adapter = valid_adapter();
        adapter.mutation = mutation;
        let request = request();
        let receipt = adapter.evaluate(&request);

        assert!(!rust_accepts_adapter_receipt(&request, &receipt));
    }
}
