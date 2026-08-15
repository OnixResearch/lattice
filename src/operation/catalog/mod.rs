#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the catalog validator uses its local ordered collection and operation-contract vocabulary"
)]
#![allow(
    assertion_density,
    reason = "fallible catalog admission uses typed diagnostics; positive and adversarial tests cover each invariant"
)]
#![allow(
    borrowed_argument_types,
    reason = "catalog validation appends diagnostics to one caller-owned growable accumulator"
)]
#![allow(
    unbounded_collection_growth,
    reason = "catalog loops apply explicit operation-contract count limits before collection insertion"
)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const ORDER_COMPARISON_WINDOW: usize = 2;
const CAIRN_PILOT_METHODS: &[&str] = &[
    super::runtime::CAIRN_CHANGE_LIST_METHOD,
    super::runtime::CAIRN_VALIDATE_METHOD,
];
const CAIRN_PILOT_PREFLIGHT: &[super::model::PreflightCheck] = &[
    super::model::PreflightCheck::RuntimeAvailable,
    super::model::PreflightCheck::ExecutableAvailable,
    super::model::PreflightCheck::RepositoryRootAdmitted,
    super::model::PreflightCheck::CredentialReferencesValid,
];
const CAIRN_PILOT_CAPABILITIES: &[super::model::CapabilityClass] = &[super::model::CapabilityClass::ProcessRead];

pub fn validate(catalog: super::model::Catalog) -> Result<super::model::ValidatedCatalog, super::Error> {
    let mut diagnostics = Vec::new();
    if catalog.schema != super::model::CATALOG_SCHEMA {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Schema,
            "catalog.schema",
            "catalog schema is unsupported",
        ));
    }
    if catalog.types.is_empty() || catalog.types.len() > super::model::MAXIMUM_CATALOG_TYPES {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            "catalog.types",
            "catalog type count is outside the admitted bounds",
        ));
    }

    let mut type_ids = BTreeSet::new();
    let mut previous_type = None;
    let mut types = BTreeMap::new();
    for definition in catalog.types.iter().take(super::model::MAXIMUM_CATALOG_TYPES) {
        let path = format!("catalog.types.{}", definition.id);
        let diagnostic_start = diagnostics.len();
        validate_type_shape(definition, &path, &mut diagnostics);
        if !type_ids.insert(definition.id.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Duplicate,
                &path,
                "operation type id is duplicated",
            ));
        }
        if previous_type.is_some_and(|value: &str| value >= definition.id.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                "catalog.types",
                "operation types must be ordered by canonical id",
            ));
        }
        previous_type = Some(definition.id.as_str());
        if diagnostics.len() == diagnostic_start {
            match validate_type_identities(definition, &path) {
                Ok(validated) => {
                    types.insert(definition.id.clone(), validated);
                }
                Err(error) => diagnostics.push(error),
            }
        }
    }

    if !diagnostics.is_empty() {
        return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
    }
    let identity = super::identity::catalog(&catalog)?;
    Ok(super::model::ValidatedCatalog { identity, types })
}

include!("shape.rs");
include!("identity.rs");
include!("order.rs");
