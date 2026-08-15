fn validate_type_identities(
    definition: &super::model::OperationType,
    path: &str,
) -> Result<super::model::ValidatedType, super::model::Diagnostic> {
    let computed_revision = super::identity::type_revision(definition).map_err(|_| {
        super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            format!("{path}.revision"),
            "operation type revision could not be computed",
        )
    })?;
    if computed_revision != definition.revision {
        return Err(super::diagnostic(
            super::model::DiagnosticCode::StaleType,
            format!("{path}.revision"),
            "operation type revision does not match its authority-bearing fields",
        ));
    }
    let mut method_identities = BTreeMap::new();
    let mut argument_schema_identities = BTreeMap::new();
    let mut output_schema_identities = BTreeMap::new();
    for method in &definition.methods {
        method_identities.insert(
            method.name.clone(),
            super::identity::method(&definition.revision, method).map_err(|_| encoding_diagnostic(path))?,
        );
        argument_schema_identities.insert(
            method.name.clone(),
            super::identity::schema(&method.arguments).map_err(|_| encoding_diagnostic(path))?,
        );
        output_schema_identities.insert(
            method.name.clone(),
            super::identity::schema(&method.output).map_err(|_| encoding_diagnostic(path))?,
        );
    }
    Ok(super::model::ValidatedType {
        definition: definition.clone(),
        method_identities,
        argument_schema_identities,
        output_schema_identities,
    })
}

fn encoding_diagnostic(path: &str) -> super::model::Diagnostic {
    super::diagnostic(
        super::model::DiagnosticCode::InvalidIdentity,
        path,
        "operation identity material could not be encoded",
    )
}
