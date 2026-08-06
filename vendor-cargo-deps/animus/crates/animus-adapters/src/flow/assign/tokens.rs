use super::Collection;
use super::SAFE_TEMPLATE_BUILTINS;
use super::TEMPLATE_CLOSE;
use super::TEMPLATE_OPEN;
use super::TEMPLATE_VARIABLE_FIELD;
use super::Text;

pub(super) fn collect_template_variables(
    template: &str,
) -> Result<Collection<Text>, super::super::RecipeWorkflowError> {
    let mut variables = Collection::with_capacity(template.matches(TEMPLATE_OPEN).count());
    let mut remaining = template;

    while let Some(open_index) = remaining.find(TEMPLATE_OPEN) {
        let variable_start = open_index.saturating_add(TEMPLATE_OPEN.len());
        let after_open = &remaining[variable_start..];
        let Some(close_index) = after_open.find(TEMPLATE_CLOSE) else {
            return Err(super::super::RecipeWorkflowError::MalformedTemplate);
        };
        let raw_variable = &after_open[..close_index];
        let variable = super::super::required_text(TEMPLATE_VARIABLE_FIELD, raw_variable)?;
        if !super::super::contains_text(variables.as_slice(), variable.as_str()) {
            variables.push(variable);
        }
        let consumed = close_index.saturating_add(TEMPLATE_CLOSE.len());
        remaining = &after_open[consumed..];
    }

    Ok(variables)
}

pub(super) fn validate_template_variables(
    workflow: &super::super::data::RecipeWorkflow,
    variables: &[Text],
) -> Result<(), super::super::RecipeWorkflowError> {
    assert!(
        workflow.parameters().iter().all(|parameter| !parameter.name().is_empty()),
        "workflow parameter names must be validated"
    );
    assert!(variables.iter().all(|variable| !variable.is_empty()), "template variables must be validated");
    for variable in variables {
        if is_safe_builtin(variable.as_str()) {
            continue;
        }
        if find_parameter(workflow.parameters(), variable.as_str()).is_none() {
            return Err(super::super::RecipeWorkflowError::UndeclaredTemplateVariable {
                variable: variable.clone(),
            });
        }
    }

    for parameter in workflow.parameters() {
        if !variables.iter().any(|variable| variable.as_str() == parameter.name()) {
            return Err(super::super::RecipeWorkflowError::UnusedParameter {
                parameter: Text::from(parameter.name()),
            });
        }
    }

    Ok(())
}
fn is_safe_builtin(variable: &str) -> bool {
    SAFE_TEMPLATE_BUILTINS.contains(&variable)
}

fn find_parameter<'a>(
    parameters: &'a [super::super::field::RecipeParameter],
    name: &str,
) -> Option<&'a super::super::field::RecipeParameter> {
    parameters.iter().find(|parameter| parameter.name() == name)
}
