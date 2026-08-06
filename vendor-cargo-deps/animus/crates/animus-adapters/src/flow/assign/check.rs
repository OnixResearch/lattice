use super::RecipeParameterInput;
use super::RecipeParameterInputKind;
use super::Text;

pub(super) fn validate_input_for_parameter(
    parameter: &super::super::field::RecipeParameter,
    input: &RecipeParameterInput,
) -> Result<(), super::super::RecipeWorkflowError> {
    assert!(!parameter.name().is_empty(), "parameter name was validated");
    assert!(!input.name().is_empty(), "input name was validated");
    if parameter.kind() == super::super::field::RecipeParameterKind::File
        && input.kind() != RecipeParameterInputKind::FileContentRef
    {
        return Err(super::super::RecipeWorkflowError::InvalidParameterSource {
            parameter: Text::from(parameter.name()),
        });
    }
    if parameter.kind() != super::super::field::RecipeParameterKind::File
        && input.kind() == RecipeParameterInputKind::FileContentRef
    {
        return Err(super::super::RecipeWorkflowError::InvalidParameterSource {
            parameter: Text::from(parameter.name()),
        });
    }
    if parameter.kind() == super::super::field::RecipeParameterKind::Select
        && !super::super::contains_text(parameter.select_options(), input.value())
    {
        return Err(super::super::RecipeWorkflowError::InvalidSelectValue {
            parameter: Text::from(parameter.name()),
            value: Text::from(input.value()),
        });
    }
    Ok(())
}

pub(super) fn find_input<'a>(inputs: &'a [RecipeParameterInput], name: &str) -> Option<&'a RecipeParameterInput> {
    inputs.iter().find(|input| input.name() == name)
}
