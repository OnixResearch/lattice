const TEXT_INPUT_HEADING: &str = "Text input";
const IMAGE_INPUT_HEADING: &str = "Image input";
const CONTEXT_REFS_HEADING: &str = "Context references";
const TOOL_POLICY_HEADING: &str = "Tool policy";
const MODEL_HINT_HEADING: &str = "Model hint";
const DISPLAY_INDEX_BASE: usize = 1;

pub fn render_command(request: &super::Request) -> Result<String, super::Error> {
    render_with_projection(request, crate::agent_prompt::StructuredInputProjection::JsonPretty)
}

// r[impl lattice.agent_runtime.projection_boundaries]
pub fn render_with_projection(
    request: &super::Request,
    projection: crate::agent_prompt::StructuredInputProjection,
) -> Result<String, super::Error> {
    let mut sections = Sections(vec![request.prompt.clone()]);
    sections.append_text(&request.text_inputs);
    sections.append_json(&request.json_inputs, projection)?;
    sections.append_images(&request.image_inputs);
    sections.append_list(CONTEXT_REFS_HEADING, &request.context_refs);
    sections.append_optional(OptionalSection {
        heading: TOOL_POLICY_HEADING,
        value: &request.tool_policy,
    });
    if let Some(model_hint) = request.model_hint.as_ref() {
        sections.append_optional(OptionalSection {
            heading: MODEL_HINT_HEADING,
            value: model_hint,
        });
    }
    Ok(sections.0.join("\n\n"))
}

struct Sections(Vec<String>);

struct OptionalSection<'a> {
    heading: &'a str,
    value: &'a str,
}

impl Sections {
    fn append_text(&mut self, inputs: &[String]) {
        self.0.reserve(inputs.len());
        for (index, input) in inputs.iter().enumerate() {
            self.0.push(format!("{TEXT_INPUT_HEADING} {}:\n{input}", index.saturating_add(DISPLAY_INDEX_BASE)));
        }
    }

    fn append_json(
        &mut self,
        inputs: &[serde_json::Value],
        projection: crate::agent_prompt::StructuredInputProjection,
    ) -> Result<(), super::Error> {
        let rendered = crate::agent_prompt::render_structured_inputs(inputs, projection)
            .map_err(|error| super::Error::PreflightRejected(error.to_string()))?;
        self.0.reserve(rendered.len());
        for input in rendered {
            self.0.push(format!("{}:\n{}", input.heading, input.body));
        }
        Ok(())
    }

    fn append_images(&mut self, inputs: &[crate::engine::value::ImageValue]) {
        self.0.reserve(inputs.len());
        for (index, input) in inputs.iter().enumerate() {
            let source = input.metadata.source_label.as_deref().unwrap_or("inline image");
            self.0.push(format!(
                "{IMAGE_INPUT_HEADING} {}: {} at {}",
                index.saturating_add(DISPLAY_INDEX_BASE),
                input.summary(),
                source
            ));
        }
    }

    fn append_list(&mut self, heading: &str, values: &[String]) {
        if values.is_empty() {
            return;
        }
        let rendered = values.iter().map(|value| format!("- {value}")).collect::<Vec<_>>().join("\n");
        self.0.push(format!("{heading}:\n{rendered}"));
    }

    fn append_optional(&mut self, section: OptionalSection<'_>) {
        let OptionalSection { heading, value } = section;
        if value.trim().is_empty() {
            return;
        }
        self.0.push(format!("{heading}:\n{value}"));
    }
}
