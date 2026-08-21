//! Pure extraction and projection of workflow expression dependencies.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionField {
    pub consumer_node: u64,
    pub field_path: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputPortFact {
    pub producer_node: u64,
    pub output_port: crate::expressions::ExpressionPortId,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
pub struct ExecutionDependency {
    pub producer_node: u64,
    pub consumer_node: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ExpressionDependency {
    pub consumer_node: u64,
    pub config_field_path: String,
    pub producer_node: u64,
    pub producer_output_port: crate::expressions::ExpressionPortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionDependencyIssueCode {
    InvalidSyntax,
    MissingProducerNode,
    MissingProducerOutputPort,
    SelfReference,
    UnsupportedField,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExpressionDependencyIssue {
    pub code: ExpressionDependencyIssueCode,
    pub consumer_node: u64,
    pub config_field_path: String,
    pub producer_node: Option<u64>,
    pub producer_output_port: Option<crate::expressions::ExpressionPortId>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionDependencyProjection {
    pub expression_dependencies: Vec<ExpressionDependency>,
    pub execution_dependencies: Vec<ExecutionDependency>,
}

// r[impl lattice.workflow_validation.explicit_dependencies.extraction]
pub fn project_expression_dependencies(
    fields: &[ExpressionField],
    node_ids: &std::collections::BTreeSet<u64>,
    output_ports: &[OutputPortFact],
    data_dependencies: &[ExecutionDependency],
) -> Result<ExpressionDependencyProjection, Vec<ExpressionDependencyIssue>> {
    let output_port_owners = output_ports
        .iter()
        .map(|fact| (fact.output_port, fact.producer_node))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut ordered_fields = fields.to_vec();
    ordered_fields.sort();
    debug_assert!(output_port_owners.len() <= output_ports.len());
    debug_assert_eq!(ordered_fields.len(), fields.len());

    let mut expression_dependencies = std::collections::BTreeSet::new();
    let mut issues = Vec::new();
    for field in &ordered_fields {
        extract_field(field, node_ids, &output_port_owners, &mut expression_dependencies, &mut issues);
    }
    if !issues.is_empty() {
        return Err(issues);
    }

    let mut execution_dependencies = data_dependencies.iter().copied().collect::<std::collections::BTreeSet<_>>();
    execution_dependencies.extend(expression_dependencies.iter().map(|dependency| ExecutionDependency {
        producer_node: dependency.producer_node,
        consumer_node: dependency.consumer_node,
    }));

    Ok(ExpressionDependencyProjection {
        expression_dependencies: expression_dependencies.into_iter().collect(),
        execution_dependencies: execution_dependencies.into_iter().collect(),
    })
}

#[allow(
    unknown_lints,
    borrowed_argument_types,
    reason = "the extractor appends ordered facts and diagnostics to caller-owned bounded accumulators"
)]
fn extract_field(
    field: &ExpressionField,
    node_ids: &std::collections::BTreeSet<u64>,
    output_port_owners: &std::collections::BTreeMap<crate::expressions::ExpressionPortId, u64>,
    dependencies: &mut std::collections::BTreeSet<ExpressionDependency>,
    issues: &mut Vec<ExpressionDependencyIssue>,
) {
    debug_assert!(dependencies.iter().all(|dependency| dependency.producer_node != dependency.consumer_node));
    debug_assert!(issues.iter().all(|issue| !issue.message.is_empty()));
    if !crate::expressions::contains_expression(&field.value) {
        return;
    }
    let template = match crate::expressions::parse_expression_template(&field.value) {
        Ok(template) => template,
        Err(error) => {
            issues.push(issue(ExpressionDependencyIssueCode::InvalidSyntax, field, None, None, error.to_string()));
            return;
        }
    };

    for segment in template.segments {
        let crate::expressions::ExpressionSegment::Reference(reference) = segment else {
            continue;
        };
        if reference.node_id == field.consumer_node {
            issues.push(issue(
                ExpressionDependencyIssueCode::SelfReference,
                field,
                Some(reference.node_id),
                Some(reference.port_id),
                "expression reference must name another node".to_string(),
            ));
            continue;
        }
        if !node_ids.contains(&reference.node_id) {
            issues.push(issue(
                ExpressionDependencyIssueCode::MissingProducerNode,
                field,
                Some(reference.node_id),
                Some(reference.port_id),
                "expression reference names a missing producer node".to_string(),
            ));
            continue;
        }
        if output_port_owners.get(&reference.port_id) != Some(&reference.node_id) {
            issues.push(issue(
                ExpressionDependencyIssueCode::MissingProducerOutputPort,
                field,
                Some(reference.node_id),
                Some(reference.port_id),
                "expression reference names a missing producer output port".to_string(),
            ));
            continue;
        }
        dependencies.insert(ExpressionDependency {
            consumer_node: field.consumer_node,
            config_field_path: field.field_path.clone(),
            producer_node: reference.node_id,
            producer_output_port: reference.port_id,
        });
    }
}

fn issue(
    code: ExpressionDependencyIssueCode,
    field: &ExpressionField,
    producer_node: Option<u64>,
    producer_output_port: Option<crate::expressions::ExpressionPortId>,
    message: String,
) -> ExpressionDependencyIssue {
    ExpressionDependencyIssue {
        code,
        consumer_node: field.consumer_node,
        config_field_path: field.field_path.clone(),
        producer_node,
        producer_output_port,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRODUCER_NODE: u64 = 10;
    const CONSUMER_NODE: u64 = 20;
    const OUTPUT_PORT: crate::expressions::ExpressionPortId = crate::expressions::ExpressionPortId(30);
    const MISSING_OUTPUT_PORT: u64 = 999;

    fn field(value: &str) -> ExpressionField {
        ExpressionField {
            consumer_node: CONSUMER_NODE,
            field_path: "shell.command".to_string(),
            value: value.to_string(),
        }
    }

    fn project(value: &str) -> Result<ExpressionDependencyProjection, Vec<ExpressionDependencyIssue>> {
        project_expression_dependencies(
            &[field(value)],
            &std::collections::BTreeSet::from([PRODUCER_NODE, CONSUMER_NODE]),
            &[OutputPortFact {
                producer_node: PRODUCER_NODE,
                output_port: OUTPUT_PORT,
            }],
            &[],
        )
    }

    #[test]
    fn extracts_and_deduplicates_supported_references() {
        let value = format!(
            "{{{{node:{PRODUCER_NODE}:port:{}|trim}}}}/{{{{node:{PRODUCER_NODE}:port:{}}}}}",
            OUTPUT_PORT.0, OUTPUT_PORT.0
        );
        let projection = project(&value).expect("supported references project");

        assert_eq!(projection.expression_dependencies.len(), 1);
        assert_eq!(projection.execution_dependencies.len(), 1);
        assert_eq!(projection.expression_dependencies[0].producer_output_port, OUTPUT_PORT);
    }

    #[test]
    fn rejects_missing_output_port() {
        let value = format!("{{{{node:{PRODUCER_NODE}:port:{MISSING_OUTPUT_PORT}}}}}");
        let issues = project(&value).expect_err("missing output port rejects");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, ExpressionDependencyIssueCode::MissingProducerOutputPort);
    }

    #[test]
    fn rejects_self_reference() {
        let value = format!("{{{{node:{CONSUMER_NODE}:port:{}}}}}", OUTPUT_PORT.0);
        let issues = project(&value).expect_err("self reference rejects");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, ExpressionDependencyIssueCode::SelfReference);
    }
}
