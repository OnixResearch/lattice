use lattice::config::AppConfig;
use lattice::config::NodeTemplate;
use lattice::config::TemplateParameter;
use lattice::config::TemplateParameterType;
use lattice::tui::App;
use lattice::tui::app::AddNodeStep;
use lattice::tui::panels;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tempfile::TempDir;

const FRAME_WIDTH: u16 = 100;
const FRAME_HEIGHT: u16 = 40;
const PARAMETER_NAME: &str = "pattern";
const PARAMETER_DESCRIPTION: &str = "Search pattern";
const PARAMETER_VALUE: &str = "needle";
const REQUIRED_PROMPT: &str = "pattern (String, required)";
const OPTIONAL_PROMPT: &str = "pattern (String, optional)";
const PARAMETER_SEQUENCE: &str = "Template parameter 1 / 1";
const PARAMETER_CONTROLS: &str = "Enter: next/create │ Esc: cancel";

fn test_config(temp_dir: &TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create prompt rendering test data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

fn parameter_template(required: bool) -> NodeTemplate {
    NodeTemplate {
        name: "grep".to_string(),
        description: "Filter text".to_string(),
        kind: "shell".to_string(),
        command: Some("grep -E '${pattern}'".to_string()),
        parameters: vec![TemplateParameter {
            name: PARAMETER_NAME.to_string(),
            type_tag: TemplateParameterType::String,
            description: PARAMETER_DESCRIPTION.to_string(),
            default: None,
            required,
            min: None,
            max: None,
        }],
        input_ports: Vec::new(),
        output_ports: Vec::new(),
    }
}

fn app_at_parameter_prompt(temp_dir: &TempDir, required: bool) -> App {
    let mut app = App::new(test_config(temp_dir));
    app.set_node_templates(vec![parameter_template(required)]);
    app.open_add_node_flow();
    app.start_template_node_flow();
    app.confirm_add_node_flow();
    app.add_node_flow.parameter_values[0] = PARAMETER_VALUE.to_string();
    app
}

fn rendered_frame_text(app: &mut App) -> String {
    let area = Rect::new(0, 0, FRAME_WIDTH, FRAME_HEIGHT);
    let mut buffer = Buffer::empty(area);
    panels::render(app, area, &mut buffer);
    buffer.content().iter().map(|cell| cell.symbol()).collect()
}

#[test]
fn required_template_parameter_prompt_is_visible_before_node_creation() {
    let temp_dir = tempfile::tempdir().expect("temporary prompt rendering directory");
    let mut app = app_at_parameter_prompt(&temp_dir, true);

    assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
    assert_eq!(app.workflow.graph.node_count(), 0);

    let rendered = rendered_frame_text(&mut app);
    assert!(rendered.contains(PARAMETER_SEQUENCE));
    assert!(rendered.contains(REQUIRED_PROMPT));
    assert!(rendered.contains(PARAMETER_DESCRIPTION));
    assert!(rendered.contains(PARAMETER_VALUE));
    assert!(rendered.contains(PARAMETER_CONTROLS));
}

#[test]
fn optional_template_parameter_prompt_is_not_labeled_required() {
    let temp_dir = tempfile::tempdir().expect("temporary prompt rendering directory");
    let mut app = app_at_parameter_prompt(&temp_dir, false);

    assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
    assert_eq!(app.workflow.graph.node_count(), 0);

    let rendered = rendered_frame_text(&mut app);
    assert!(rendered.contains(OPTIONAL_PROMPT));
    assert!(!rendered.contains(REQUIRED_PROMPT));
}
