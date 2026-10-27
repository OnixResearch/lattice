//! Exact typed Mantle bundle remeasurement; runtime authority remains separate.
use mantle_wasm_consumer_verifier::CONSUMER_VERIFIER_NON_CLAIMS;
use mantle_wasm_consumer_verifier::ConsumerLayer;
use mantle_wasm_consumer_verifier::ConsumerMemberKind;
use mantle_wasm_consumer_verifier::ConsumerMemberStatus;
use mantle_wasm_consumer_verifier::ConsumerRoot;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;
use mantle_wasm_consumer_verifier::MaterializationBundle;

use crate::operation_pack::Error;
use crate::operation_pack::model;
use crate::operation_pack::wasm;

pub(super) struct VerifiedBundle {
    pub component_bytes: Vec<u8>,
    pub component_identity: String,
    pub wit_identity: String,
    pub bundle_content_identity: String,
    pub canonical_bundle_identity: String,
}

pub(super) fn verify(
    request: &super::super::execution::Request<'_>,
    runtime: &model::RuntimeArtifact,
    descriptor: &wasm::Descriptor,
) -> Result<VerifiedBundle, Error> {
    let root =
        crate::capability::open_root_path(request.pack_root).map_err(|_| Error::Io("component capability root"))?;
    let candidate = &request.admitted.candidate;
    let component_bytes = super::verification::read_content(&root, candidate, &runtime.content_id)?;
    let wit_bytes = super::verification::read_content(&root, candidate, &descriptor.wit_content_id)?;
    if wit_bytes != wasm::FIRST_WORLD_WIT.as_bytes() {
        return Err(Error::Invalid("first component WIT world"));
    }
    let bundle_bytes = super::verification::read_content(&root, candidate, &descriptor.bundle_content_id)?;
    if bundle_bytes.len() > model::MAXIMUM_MANIFEST_BYTES as usize {
        return Err(Error::Invalid("component bundle document bound"));
    }
    let bundle: MaterializationBundle =
        serde_json::from_slice(&bundle_bytes).map_err(|_| Error::Invalid("component bundle schema"))?;
    let declared = mantle_wasm_consumer_verifier::facade::declared_member_objects(&bundle);
    let verifier = ConsumerRoot::open(request.pack_root).map_err(|_| Error::Invalid("component bundle root"))?;
    let report = verifier
        .verify_bundle(bundle.clone())
        .map_err(|_| Error::Invalid("component bundle verification"))?;
    if report.status != ConsumerVerificationStatus::Verified
        || report.layers_completed != [ConsumerLayer::Structural, ConsumerLayer::Bytes]
        || !report.blockers.is_empty()
        || !report.non_claims.iter().map(String::as_str).eq(CONSUMER_VERIFIER_NON_CLAIMS)
        || report.members.len() != declared.len()
        || report.expected_runtime_profile_blake3.as_deref().map(|digest| format!("b3:{digest}"))
            != Some(descriptor.runtime_profile_identity.clone())
    {
        return Err(Error::Invalid("component bundle verdict"));
    }
    let component_identity = crate::operation_pack::identity::content_identity(&component_bytes);
    let wit_identity = crate::operation_pack::identity::content_identity(&wit_bytes);
    let component_content = content(candidate, &runtime.content_id)?;
    let wit_content = content(candidate, &descriptor.wit_content_id)?;
    let mut component_members = 0_usize;
    let mut wit_members = 0_usize;
    for ((role, object), member) in declared.iter().zip(&report.members) {
        if member.role != *role
            || member.digest_blake3 != object.digest_blake3.to_string()
            || member.size_bytes != object.size_bytes
            || member.status != Some(ConsumerMemberStatus::Matched)
        {
            return Err(Error::Invalid("component bundle member observation"));
        }
        let relative = relative(&object.logical_path)?;
        let identity = format!("b3:{}", member.digest_blake3);
        let admitted = match member.kind {
            Some(ConsumerMemberKind::File) => {
                member.source_files.is_empty()
                    && candidate.manifest.contents.iter().any(|content| {
                        content.path == relative
                            && content.content_identity == identity
                            && content.byte_length == member.size_bytes
                    })
            }
            Some(ConsumerMemberKind::SourceTree) => {
                *role == "source-closure"
                    && source_tree_is_declared(relative, &member.source_files, &candidate.manifest.contents)
            }
            None => false,
        };
        if !admitted {
            return Err(Error::Invalid("undeclared component bundle member"));
        }
        if *role == "final-portable" {
            component_members += 1;
            if member.kind != Some(ConsumerMemberKind::File)
                || relative != component_content.path
                || identity != component_identity
            {
                return Err(Error::Invalid("component bundle artifact binding"));
            }
        }
        if *role == "wit-input" {
            wit_members += 1;
            if member.kind != Some(ConsumerMemberKind::File) || relative != wit_content.path || identity != wit_identity
            {
                return Err(Error::Invalid("component bundle WIT binding"));
            }
        }
    }
    if component_members != 1 || wit_members != 1 {
        return Err(Error::Invalid("component bundle cohort"));
    }
    let canonical_bundle_identity =
        report.bundle_identity_blake3.ok_or(Error::Invalid("component canonical bundle identity"))?;
    Ok(VerifiedBundle {
        component_bytes,
        component_identity,
        wit_identity,
        bundle_content_identity: crate::operation_pack::identity::content_identity(&bundle_bytes),
        canonical_bundle_identity,
    })
}

fn content<'a>(candidate: &'a model::Candidate, id: &str) -> Result<&'a model::ContentItem, Error> {
    candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == id)
        .ok_or(Error::Invalid("component content role"))
}

fn relative(logical: &str) -> Result<&str, Error> {
    let path = logical.strip_prefix('/').ok_or(Error::Invalid("component member path"))?;
    if path.is_empty() || path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return Err(Error::Invalid("component member path"));
    }
    Ok(path)
}

fn source_tree_is_declared(
    root: &str,
    files: &[mantle_wasm_consumer_verifier::ConsumerSourceFile],
    contents: &[model::ContentItem],
) -> bool {
    let mut count = 0_usize;
    for content in contents {
        let Some(path) = content.path.strip_prefix(root).and_then(|suffix| suffix.strip_prefix('/')) else {
            continue;
        };
        count += 1;
        if !files.iter().any(|file| {
            file.relative_path == path
                && content.byte_length == file.size_bytes
                && content.content_identity == format!("b3:{}", file.digest_blake3)
        }) {
            return false;
        }
    }
    count == files.len()
}
