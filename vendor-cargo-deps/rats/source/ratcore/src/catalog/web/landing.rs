/// Renders a deterministic Markdown browser showcase landing surface.
#[must_use]
pub fn render_browser_showcase_portal_markdown(entries: &[super::BrowserShowcaseEntry]) -> String {
    let mut sorted_entries = entries.to_vec();
    sorted_entries.sort_by(|left, right| left.id.cmp(&right.id));
    let mut markdown = String::from("# Browser showcase portal\n\n");
    markdown
        .push_str("This landing surface is generated from the renderer-neutral Ratcore browser showcase catalog.\n\n");
    debug_assert!(markdown.starts_with("# Browser showcase portal"));
    debug_assert_eq!(sorted_entries.len(), entries.len());
    for entry in sorted_entries {
        markdown.push_str(&entry_markdown(entry));
    }
    markdown
}

fn entry_markdown(entry: super::BrowserShowcaseEntry) -> String {
    debug_assert!(!entry.id.trim().is_empty());
    debug_assert!(!entry.title.trim().is_empty());
    let mut markdown = String::new();
    markdown.push_str("## ");
    markdown.push_str(&entry.title);
    markdown.push_str("\n\n");
    markdown.push_str(&markdown_field(MarkdownField::new("Entry", &entry.id)));
    markdown.push_str(&markdown_field(MarkdownField::new("Renderer", &entry.renderer_label)));
    markdown.push_str(&markdown_field(MarkdownField::new("Package", &entry.package_name)));
    markdown.push_str(&markdown_field(MarkdownField::new("Path", &entry.crate_path)));
    markdown.push_str(&markdown_field(MarkdownField::new("Launch", &entry.launch_command)));
    markdown.push_str(&markdown_field(MarkdownField::new("Bundle", &entry.bundle_command)));
    markdown.push_str(&markdown_field(MarkdownField::new("Validation", &entry.validation_profile)));
    markdown.push_str(&markdown_field(MarkdownField::new("Evidence", entry.evidence_status.label())));
    markdown
        .push_str(&markdown_field(MarkdownField::new("Fixtures", &display_fixture_ids(&entry.supported_fixture_ids))));
    markdown.push_str(&markdown_field(MarkdownField::new("Evidence links", &display_list(&entry.evidence_links))));
    markdown.push_str(&markdown_field(MarkdownField::new("Visible gaps", &display_list(&entry.unsupported_reasons))));
    markdown.push('\n');
    markdown
}

struct MarkdownField<'a> {
    label: &'a str,
    value: &'a str,
}

impl<'a> MarkdownField<'a> {
    fn new(label: &'a str, value: &'a str) -> Self {
        Self { label, value }
    }
}

fn markdown_field(field: MarkdownField<'_>) -> String {
    let mut markdown = String::new();
    markdown.push_str("- **");
    markdown.push_str(field.label);
    markdown.push_str("**: ");
    markdown.push_str(field.value);
    markdown.push('\n');
    markdown
}

fn display_fixture_ids(values: &[super::super::FixtureId]) -> String {
    if values.is_empty() {
        super::EMPTY_VALUE.to_string()
    } else {
        values.iter().map(ToString::to_string).collect::<Vec<_>>().join(super::LIST_SEPARATOR)
    }
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        super::EMPTY_VALUE.to_string()
    } else {
        values.join(super::LIST_SEPARATOR)
    }
}
