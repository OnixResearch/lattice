use super::super::*;
use super::support::*;

#[test]
fn rejects_duplicate_declared_tools() {
    let mut draft = safe_stdio_draft();
    draft.declared_tools = DUPLICATE_DECLARED_TOOLS;
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("duplicate tool names should fail");

    assert_eq!(
        error,
        PluginDescriptorError::DuplicateTool {
            name: Text::from(TOOL_NAME),
        }
    );
}

#[test]
fn rejects_built_in_tool_collision() {
    let mut draft = safe_stdio_draft();
    draft.declared_tools = BUILT_IN_COLLISION_TOOLS;
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("built-in tool name collision should fail");

    assert_eq!(
        error,
        PluginDescriptorError::ToolNameCollision {
            name: Text::from(BUILT_IN_TOOL_NAME),
        }
    );
}

#[test]
fn rejects_catalog_tool_collision() {
    let mut draft = safe_stdio_draft();
    draft.declared_tools = CATALOG_COLLISION_TOOLS;
    let error = PluginDescriptor::validate(&draft, BUILT_IN_TOOLS, CATALOG_TOOLS)
        .expect_err("catalog tool name collision should fail");

    assert_eq!(
        error,
        PluginDescriptorError::ToolNameCollision {
            name: Text::from(CATALOG_TOOL_NAME),
        }
    );
}
