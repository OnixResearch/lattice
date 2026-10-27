use super::model::{
    AST_GREP_FRESH_STEP_NAME, AUTHORITY_FRESH_STEP_NAME, AUTHORITY_REPLAY_STEP_NAME,
    ISSUER_FRESH_STEP_NAME, ISSUER_REPLAY_STEP_NAME, LICENSE_FRESH_STEP_NAME,
    LICENSE_REPLAY_STEP_NAME, NICKEL_COHORT_STEP_NAME, NICKEL_EVALUATOR_IDENTITY,
    NICKEL_SOURCE_REVISION, NICKEL_VERSION, POLICY_FRESH_STEP_NAME, SEMANTICS_FRESH_STEP_NAME,
    SEMANTICS_REPLAY_STEP_NAME, StepReceipt,
};
use crate::ast_grep_audit_identity::{
    AST_GREP_AUDIT_SCHEMA, DEFAULT_AST_GREP_TOOL, DEFAULT_AST_GREP_VERSION,
};
use crate::authority_identity::{
    AUTHORITY_DECISION_SCHEMA, AUTHORITY_FIXTURE_SCHEMA, AUTHORITY_INPUT_SCHEMA,
    AUTHORITY_RELEASE_RECEIPT_SCHEMA,
};
use crate::license_consumption_identity::{
    LICENSE_CONSUMPTION_FIXTURE_SCHEMA, LICENSE_CONSUMPTION_RECEIPT_SCHEMA,
    LICENSE_CONSUMPTION_RELEASE_RECEIPT_SCHEMA,
};
use crate::secretspec_issuer_identity::{
    ISSUER_DECLARATION_SCHEMA, ISSUER_RELEASE_RECEIPT_SCHEMA, SECRETSPEC_REVIEWED_COMMIT,
    SECRETSPEC_VERSION,
};
use crate::ucan_semantics_identity::{
    BASALT_POLICY_BLAKE3, UCAN_PROTOCOL_PROFILE, UCAN_RUNTIME_REVISION,
    UCAN_SEMANTICS_FIXTURE_SCHEMA, UCAN_SUPPORT_CATALOG_BLAKE3,
    UCAN_SUPPORT_CATALOG_REVISION,
};
use std::fs;
use std::path::Path;

pub(super) fn write_receipts(
    dir: &Path,
    passed: bool,
    receipts: &[StepReceipt],
) -> Result<(), String> {
    fs::write(dir.join("summary.json"), render_json(passed, receipts))
        .map_err(|error| error.to_string())?;
    fs::write(dir.join("summary.md"), render_markdown(passed, receipts))
        .map_err(|error| error.to_string())?;
    fs::write(
        dir.join("nickel-toolchain.json"),
        render_nickel_receipt(passed, receipts),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        dir.join("authority-core.json"),
        render_authority_receipt(passed, receipts),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        dir.join("ucan-semantics.json"),
        render_ucan_semantics_receipt(passed, receipts),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        dir.join("license-consumption.json"),
        render_license_consumption_receipt(passed, receipts),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        dir.join("ast-grep-policy-audits.json"),
        render_ast_grep_receipt(passed, receipts),
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn render_json(passed: bool, receipts: &[StepReceipt]) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"passed\": {},\n", passed));
    output.push_str("  \"nickel_toolchain\": {\n");
    output.push_str(&format!(
        "    \"version\": {},\n",
        json_string(NICKEL_VERSION)
    ));
    output.push_str(&format!(
        "    \"source_revision\": {},\n",
        json_string(NICKEL_SOURCE_REVISION)
    ));
    output.push_str(&format!(
        "    \"evaluator_identity\": {}\n",
        json_string(NICKEL_EVALUATOR_IDENTITY)
    ));
    output.push_str("  },\n");
    output.push_str("  \"issuer_key\": {\n");
    output.push_str(&format!(
        "    \"declaration_schema\": {},\n",
        json_string(ISSUER_DECLARATION_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"secretspec_version\": {},\n",
        json_string(SECRETSPEC_VERSION)
    ));
    output.push_str(&format!(
        "    \"secretspec_commit\": {}\n",
        json_string(SECRETSPEC_REVIEWED_COMMIT)
    ));
    output.push_str("  },\n");
    output.push_str("  \"authority_core\": {\n");
    output.push_str(&format!(
        "    \"input_schema\": {},\n",
        json_string(AUTHORITY_INPUT_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"fixture_schema\": {},\n",
        json_string(AUTHORITY_FIXTURE_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"decision_schema\": {},\n",
        json_string(AUTHORITY_DECISION_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"basalt_policy_blake3\": {}\n",
        json_string(BASALT_POLICY_BLAKE3)
    ));
    output.push_str("  },\n");
    output.push_str("  \"ucan_semantics\": {\n");
    output.push_str(&format!(
        "    \"fixture_schema\": {},\n",
        json_string(UCAN_SEMANTICS_FIXTURE_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"runtime_ucan_revision\": {},\n",
        json_string(UCAN_RUNTIME_REVISION)
    ));
    output.push_str(&format!(
        "    \"support_catalog_revision\": {},\n",
        json_string(UCAN_SUPPORT_CATALOG_REVISION)
    ));
    output.push_str(&format!(
        "    \"support_catalog_blake3\": {},\n",
        json_string(UCAN_SUPPORT_CATALOG_BLAKE3)
    ));
    output.push_str(&format!(
        "    \"basalt_policy_blake3\": {}\n",
        json_string(BASALT_POLICY_BLAKE3)
    ));
    output.push_str("  },\n");
    output.push_str("  \"license_consumption\": {\n");
    output.push_str(&format!(
        "    \"receipt_schema\": {},\n",
        json_string(LICENSE_CONSUMPTION_RECEIPT_SCHEMA)
    ));
    output.push_str(&format!(
        "    \"fixture_schema\": {}\n",
        json_string(LICENSE_CONSUMPTION_FIXTURE_SCHEMA)
    ));
    output.push_str("  },\n");
    output.push_str("  \"steps\": [\n");
    for (index, receipt) in receipts.iter().enumerate() {
        if index > 0 {
            output.push_str(",\n");
        }
        output.push_str("    {\n");
        output.push_str(&format!("      \"name\": {},\n", json_string(receipt.name)));
        output.push_str(&format!(
            "      \"command\": {},\n",
            json_string(&receipt.command)
        ));
        match receipt.status {
            Some(status) => output.push_str(&format!("      \"status\": {status},\n")),
            None => output.push_str("      \"status\": null,\n"),
        }
        output.push_str(&format!("      \"success\": {},\n", receipt.success));
        output.push_str(&format!(
            "      \"elapsed_ms\": {},\n",
            receipt.elapsed.as_millis()
        ));
        output.push_str(&format!(
            "      \"stdout_tail\": {},\n",
            json_string(&receipt.stdout_tail)
        ));
        output.push_str(&format!(
            "      \"stderr_tail\": {}\n",
            json_string(&receipt.stderr_tail)
        ));
        output.push_str("    }");
    }
    output.push_str("\n  ]\n}\n");
    output
}

fn render_nickel_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let cohort_identity_passed = step_succeeded(receipts, NICKEL_COHORT_STEP_NAME);
    format!(
        concat!(
            "{{\n",
            "  \"schema\": \"basalt.nickel-toolchain-release-receipt.v1\",\n",
            "  \"release_passed\": {},\n",
            "  \"version\": {},\n",
            "  \"source_revision\": {},\n",
            "  \"evaluator_identity\": {},\n",
            "  \"cohort_identity_passed\": {},\n",
            "  \"authority_owner\": \"basalt-rust\",\n",
            "  \"non_claim\": \"evaluator alignment does not prove policy correctness or runtime authority\"\n",
            "}}\n"
        ),
        passed,
        json_string(NICKEL_VERSION),
        json_string(NICKEL_SOURCE_REVISION),
        json_string(NICKEL_EVALUATOR_IDENTITY),
        cohort_identity_passed,
    )
}

fn render_authority_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let fixture_freshness_passed = step_succeeded(receipts, AUTHORITY_FRESH_STEP_NAME);
    let replay = receipts
        .iter()
        .find(|receipt| receipt.name == AUTHORITY_REPLAY_STEP_NAME);
    let runtime_replay_passed = replay.is_some_and(|receipt| receipt.success);
    let runtime_replay_executed =
        replay.is_some_and(|receipt| !receipt.command.contains("skipped in sandboxed flake check"));

    format!(
        concat!(
            "{{\n",
            "  \"schema\": {},\n",
            "  \"release_passed\": {},\n",
            "  \"input_schema\": {},\n",
            "  \"fixture_schema\": {},\n",
            "  \"decision_schema\": {},\n",
            "  \"basalt_policy_blake3\": {},\n",
            "  \"fixture_freshness_passed\": {},\n",
            "  \"runtime_replay_executed\": {},\n",
            "  \"runtime_replay_passed\": {}\n",
            "}}\n"
        ),
        json_string(AUTHORITY_RELEASE_RECEIPT_SCHEMA),
        passed,
        json_string(AUTHORITY_INPUT_SCHEMA),
        json_string(AUTHORITY_FIXTURE_SCHEMA),
        json_string(AUTHORITY_DECISION_SCHEMA),
        json_string(BASALT_POLICY_BLAKE3),
        fixture_freshness_passed,
        runtime_replay_executed,
        runtime_replay_passed,
    )
}

fn render_ucan_semantics_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let policy_freshness_passed = step_succeeded(receipts, POLICY_FRESH_STEP_NAME);
    let fixture_freshness_passed = step_succeeded(receipts, SEMANTICS_FRESH_STEP_NAME);
    let replay = receipts
        .iter()
        .find(|receipt| receipt.name == SEMANTICS_REPLAY_STEP_NAME);
    let runtime_replay_passed = replay.is_some_and(|receipt| receipt.success);
    let runtime_replay_executed =
        replay.is_some_and(|receipt| !receipt.command.contains("skipped in sandboxed flake check"));

    format!(
        concat!(
            "{{\n",
            "  \"schema\": \"basalt.ucan-semantics-receipt.v2\",\n",
            "  \"release_passed\": {},\n",
            "  \"fixture_schema\": {},\n",
            "  \"protocol_profile\": {},\n",
            "  \"runtime_ucan_revision\": {},\n",
            "  \"support_catalog_revision\": {},\n",
            "  \"support_catalog_blake3\": {},\n",
            "  \"basalt_policy_blake3\": {},\n",
            "  \"policy_freshness_passed\": {},\n",
            "  \"fixture_freshness_passed\": {},\n",
            "  \"runtime_replay_executed\": {},\n",
            "  \"runtime_replay_passed\": {}\n",
            "}}\n"
        ),
        passed,
        json_string(UCAN_SEMANTICS_FIXTURE_SCHEMA),
        json_string(UCAN_PROTOCOL_PROFILE),
        json_string(UCAN_RUNTIME_REVISION),
        json_string(UCAN_SUPPORT_CATALOG_REVISION),
        json_string(UCAN_SUPPORT_CATALOG_BLAKE3),
        json_string(BASALT_POLICY_BLAKE3),
        policy_freshness_passed,
        fixture_freshness_passed,
        runtime_replay_executed,
        runtime_replay_passed,
    )
}

fn render_markdown(passed: bool, receipts: &[StepReceipt]) -> String {
    let mut output = String::new();
    output.push_str("# Release Verification Summary\n\n");
    output.push_str(if passed {
        "Status: PASS\n\n"
    } else {
        "Status: FAIL\n\n"
    });
    output.push_str("## Nickel toolchain identity\n\n");
    output.push_str(&format!("- Version: `{NICKEL_VERSION}`\n"));
    output.push_str(&format!("- Source revision: `{NICKEL_SOURCE_REVISION}`\n"));
    output.push_str(&format!(
        "- Evaluator identity: `{NICKEL_EVALUATOR_IDENTITY}`\n\n"
    ));
    output.push_str("## Authority core identity\n\n");
    output.push_str(&format!("- Input schema: `{AUTHORITY_INPUT_SCHEMA}`\n"));
    output.push_str(&format!("- Fixture schema: `{AUTHORITY_FIXTURE_SCHEMA}`\n"));
    output.push_str(&format!(
        "- Decision schema: `{AUTHORITY_DECISION_SCHEMA}`\n"
    ));
    output.push_str(&format!(
        "- Basalt policy BLAKE3: `{BASALT_POLICY_BLAKE3}`\n\n"
    ));
    output.push_str("## UCAN semantics identity\n\n");
    output.push_str(&format!(
        "- Fixture schema: `{UCAN_SEMANTICS_FIXTURE_SCHEMA}`\n"
    ));
    output.push_str(&format!(
        "- Protocol profile: `{UCAN_PROTOCOL_PROFILE}`\n"
    ));
    output.push_str(&format!(
        "- Runtime UCAN revision: `{UCAN_RUNTIME_REVISION}`\n"
    ));
    output.push_str(&format!(
        "- Support catalog revision: `{UCAN_SUPPORT_CATALOG_REVISION}`\n"
    ));
    output.push_str(&format!(
        "- Support catalog BLAKE3: `{UCAN_SUPPORT_CATALOG_BLAKE3}`\n"
    ));
    output.push_str(&format!(
        "- Basalt policy BLAKE3: `{BASALT_POLICY_BLAKE3}`\n\n"
    ));
    output.push_str("| Step | Result | Status | Elapsed ms |\n");
    output.push_str("| --- | --- | --- | ---: |\n");
    for receipt in receipts {
        let status = receipt
            .status
            .map_or_else(|| "not-started".to_owned(), |status| status.to_string());
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            receipt.name,
            if receipt.success { "PASS" } else { "FAIL" },
            status,
            receipt.elapsed.as_millis()
        ));
    }
    output
}

fn step_succeeded(receipts: &[StepReceipt], name: &str) -> bool {
    receipts
        .iter()
        .find(|receipt| receipt.name == name)
        .is_some_and(|receipt| receipt.success)
}

fn render_license_consumption_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let fixture_freshness_passed = step_succeeded(receipts, LICENSE_FRESH_STEP_NAME);
    let replay = receipts
        .iter()
        .find(|receipt| receipt.name == LICENSE_REPLAY_STEP_NAME);
    let runtime_replay_passed = replay.is_some_and(|receipt| receipt.success);
    let runtime_replay_executed =
        replay.is_some_and(|receipt| !receipt.command.contains("skipped in sandboxed flake check"));

    format!(
        concat!(
            "{{\n",
            "  \"schema\": {},\n",
            "  \"release_passed\": {},\n",
            "  \"receipt_schema\": {},\n",
            "  \"fixture_schema\": {},\n",
            "  \"fixture_freshness_passed\": {},\n",
            "  \"runtime_replay_executed\": {},\n",
            "  \"runtime_replay_passed\": {}\n",
            "}}\n"
        ),
        json_string(LICENSE_CONSUMPTION_RELEASE_RECEIPT_SCHEMA),
        passed,
        json_string(LICENSE_CONSUMPTION_RECEIPT_SCHEMA),
        json_string(LICENSE_CONSUMPTION_FIXTURE_SCHEMA),
        fixture_freshness_passed,
        runtime_replay_executed,
        runtime_replay_passed,
    )
}

fn render_issuer_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let freshness_passed = step_succeeded(receipts, ISSUER_FRESH_STEP_NAME);
    let replay = receipts
        .iter()
        .find(|receipt| receipt.name == ISSUER_REPLAY_STEP_NAME);
    let runtime_replay_passed = replay.is_some_and(|receipt| receipt.success);
    let runtime_replay_executed = replay
        .is_some_and(|receipt| !receipt.command.contains("skipped in sandboxed flake check"));

    format!(
        concat!(
            "{{\n",
            "  \"schema\": {},\n",
            "  \"release_passed\": {},\n",
            "  \"declaration_schema\": {},\n",
            "  \"secretspec_version\": {},\n",
            "  \"secretspec_commit\": {},\n",
            "  \"fixture_freshness_passed\": {},\n",
            "  \"runtime_replay_executed\": {},\n",
            "  \"runtime_replay_passed\": {}\n",
            "}}\n"
        ),
        json_string(ISSUER_RELEASE_RECEIPT_SCHEMA),
        passed,
        json_string(ISSUER_DECLARATION_SCHEMA),
        json_string(SECRETSPEC_VERSION),
        json_string(SECRETSPEC_REVIEWED_COMMIT),
        freshness_passed,
        runtime_replay_executed,
        runtime_replay_passed,
    )
}

fn render_ast_grep_receipt(passed: bool, receipts: &[StepReceipt]) -> String {
    let audit_passed = step_succeeded(receipts, AST_GREP_FRESH_STEP_NAME);

    format!(
        concat!(
            "{{\n",
            "  \"schema\": {},\n",
            "  \"release_passed\": {},\n",
            "  \"tool\": {},\n",
            "  \"tool_version\": {},\n",
            "  \"audit_freshness_passed\": {},\n",
            "  \"advisory_by_default\": {}\n",
            "}}\n"
        ),
        json_string(AST_GREP_AUDIT_SCHEMA),
        passed,
        json_string(DEFAULT_AST_GREP_TOOL),
        json_string(DEFAULT_AST_GREP_VERSION),
        audit_passed,
        true,
    )
}

fn json_string(input: &str) -> String {
    let mut output = String::from("\"");
    for character in input.chars() {
        match character {
            '\"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if control.is_control() => {
                output.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => output.push(other),
        }
    }
    output.push('\"');
    output
}
