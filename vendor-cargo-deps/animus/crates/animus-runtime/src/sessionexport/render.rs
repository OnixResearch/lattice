//! Pure deterministic redacted artifact renderers.

type SessionTreeExportArtifact = super::data::SessionTreeExportArtifact;
type SessionTreeExportError = super::data::SessionTreeExportError;
type SessionTreeExportFormat = super::data::SessionTreeExportFormat;
type SessionTreeExportIssue = super::data::SessionTreeExportIssue;
type SessionTreeExportPolicy = super::data::SessionTreeExportPolicy;
type SessionTreeExportProjection = super::data::SessionTreeExportProjection;

const SESSION_TREE_EXPORT_ARTIFACT_HASH_DOMAIN: &str = super::data::SESSION_TREE_EXPORT_ARTIFACT_HASH_DOMAIN;
const FIELD_ARTIFACT: &str = "session_tree_export_artifact";
const HTML_PREFIX: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Redacted session tree</title></head><body><main>";
const HTML_SUFFIX: &str = "</main></body></html>\n";
const SAFE_LINES_PREFIX: &str = "schema\tanimus.session-tree-export-lines.v1\n";

#[derive(Clone, Copy)]
struct RenderPair<'a> {
    label: &'a str,
    value: &'a str,
}

struct RenderBuffer(String);

impl RenderBuffer {
    fn from_prefix(prefix: &str) -> Self {
        Self(String::from(prefix))
    }

    fn push_str(&mut self, value: &str) {
        self.0.push_str(value);
    }

    fn push_char(&mut self, value: char) {
        self.0.push(value);
    }

    fn push_html_pair(&mut self, pair: RenderPair<'_>) {
        self.push_str("<dt>");
        self.push_html_escaped(pair.label);
        self.push_str("</dt><dd>");
        self.push_html_escaped(pair.value);
        self.push_str("</dd>");
    }

    fn push_html_escaped(&mut self, value: &str) {
        for character in value.chars() {
            match character {
                '&' => self.push_str("&amp;"),
                '<' => self.push_str("&lt;"),
                '>' => self.push_str("&gt;"),
                '"' => self.push_str("&quot;"),
                '\'' => self.push_str("&#39;"),
                other => self.push_char(other),
            }
        }
    }

    fn push_safe_line(&mut self, pair: RenderPair<'_>) {
        self.push_line_escaped(pair.label);
        self.push_char('\t');
        self.push_line_escaped(pair.value);
        self.push_char('\n');
    }

    fn push_line_escaped(&mut self, value: &str) {
        for character in value.chars() {
            match character {
                '\\' => self.push_str("\\\\"),
                '\t' => self.push_str("\\t"),
                '\n' => self.push_str("\\n"),
                '\r' => self.push_str("\\r"),
                other => self.push_char(other),
            }
        }
    }

    fn into_string(self) -> String {
        self.0
    }
}

/// Renders a deterministic redacted artifact under the admitted format policy.
///
/// # Errors
///
/// Returns [`SessionTreeExportError`] when the format is not allowed or the
/// rendered artifact exceeds its configured byte budget.
// r[impl agent_harness_runtime.session_tree_export.html]
// r[impl agent_harness_runtime.session_tree_export.determinism]
pub fn render_session_tree_export(
    policy: &SessionTreeExportPolicy,
    projection: &SessionTreeExportProjection,
    format: SessionTreeExportFormat,
) -> Result<SessionTreeExportArtifact, SessionTreeExportError> {
    if !policy.formats.contains(&format) {
        return Err(SessionTreeExportError::new(SessionTreeExportIssue::UnsupportedFormat, FIELD_ARTIFACT));
    }
    let rendered = match format {
        SessionTreeExportFormat::Html => render_html(projection),
        SessionTreeExportFormat::SafeLines => render_safe_lines(projection),
    };
    let byte_count = u32::try_from(rendered.len())
        .map_err(|_error| SessionTreeExportError::new(SessionTreeExportIssue::BoundExceeded, FIELD_ARTIFACT))?;
    if byte_count > policy.artifact_byte_limit(format) {
        return Err(SessionTreeExportError::new(SessionTreeExportIssue::BoundExceeded, FIELD_ARTIFACT));
    }
    debug_assert!(policy.formats.contains(&format));
    debug_assert!(!rendered.is_empty());
    let artifact_hash = super::core::hash_parts(SESSION_TREE_EXPORT_ARTIFACT_HASH_DOMAIN, &[
        projection.projection_hash.as_str(),
        format.as_str(),
        rendered.as_str(),
    ]);
    Ok(SessionTreeExportArtifact {
        format,
        bytes: rendered.into_bytes(),
        projection_hash: projection.projection_hash.clone(),
        byte_count,
        artifact_hash,
    })
}

fn render_html(projection: &SessionTreeExportProjection) -> String {
    debug_assert!(!projection.entries.is_empty());
    debug_assert!(!projection.projection_hash.is_empty());
    let mut output = RenderBuffer::from_prefix(HTML_PREFIX);
    output.push_str("<h1>Redacted session tree</h1><dl>");
    output.push_html_pair(RenderPair {
        label: "session",
        value: projection.session_id.as_str(),
    });
    output.push_html_pair(RenderPair {
        label: "active leaf",
        value: projection.active_leaf_id.as_str(),
    });
    output.push_html_pair(RenderPair {
        label: "projection",
        value: projection.projection_hash.as_str(),
    });
    output.push_str("</dl>");
    push_topology_view(&mut output, projection);
    push_storage_view(&mut output, projection);
    output.push_str("<p>redaction: ");
    output.push_str(super::data::REDACTION_MARKER);
    output.push_str("</p>");
    output.push_str(HTML_SUFFIX);
    let rendered = output.into_string();
    debug_assert!(rendered.starts_with(HTML_PREFIX));
    debug_assert!(rendered.ends_with(HTML_SUFFIX));
    rendered
}

fn push_topology_view(output: &mut RenderBuffer, projection: &SessionTreeExportProjection) {
    debug_assert!(!projection.entries.is_empty());
    debug_assert!(!projection.active_leaf_id.is_empty());
    output.push_str("<section><h2>Topology</h2><ul>");
    for entry in &projection.entries {
        output.push_str("<li data-depth=\"");
        output.push_str(entry.depth.to_string().as_str());
        output.push_str("\" data-active=\"");
        output.push_str(if entry.is_active_path { "true" } else { "false" });
        output.push_str("\"><code>");
        output.push_html_escaped(entry.entry_id.as_str());
        output.push_str("</code> parent=<code>");
        output.push_html_escaped(entry.parent_id.as_deref().unwrap_or("root"));
        output.push_str("</code> kind=<strong>");
        output.push_html_escaped(entry.kind.as_str());
        output.push_str("</strong></li>");
    }
    output.push_str("</ul></section>");
}

fn push_storage_view(output: &mut RenderBuffer, projection: &SessionTreeExportProjection) {
    debug_assert!(!projection.entries.is_empty());
    debug_assert!(!projection.projection_hash.is_empty());
    output.push_str("<section><h2>Storage order</h2><ol>");
    for entry in &projection.entries {
        output.push_str("<li><strong>");
        output.push_html_escaped(entry.kind.as_str());
        output.push_str("</strong> <code>");
        output.push_html_escaped(entry.entry_id.as_str());
        output.push_str("</code> <span>");
        output.push_html_escaped(entry.safe_summary.as_str());
        output.push_str("</span> <small>status=");
        output.push_html_escaped(entry.status.as_str());
        output.push_str(" repair-target=");
        output.push_html_escaped(entry.repair_target_id.as_deref().unwrap_or("none"));
        output.push_str(" children=");
        output.push_str(entry.child_count.to_string().as_str());
        output.push_str(" omitted=");
        output.push_str(entry.omitted_field_count.to_string().as_str());
        output.push_str("</small></li>");
    }
    output.push_str("</ol></section>");
}

fn render_safe_lines(projection: &SessionTreeExportProjection) -> String {
    debug_assert!(!projection.entries.is_empty());
    debug_assert!(!projection.projection_hash.is_empty());
    let mut output = RenderBuffer::from_prefix(SAFE_LINES_PREFIX);
    output.push_safe_line(RenderPair {
        label: "session",
        value: projection.session_id.as_str(),
    });
    output.push_safe_line(RenderPair {
        label: "active_leaf",
        value: projection.active_leaf_id.as_str(),
    });
    output.push_safe_line(RenderPair {
        label: "projection",
        value: projection.projection_hash.as_str(),
    });
    for entry in &projection.entries {
        output.push_str("entry\t");
        output.push_line_escaped(entry.entry_id.as_str());
        output.push_char('\t');
        output.push_line_escaped(entry.parent_id.as_deref().unwrap_or("root"));
        output.push_char('\t');
        output.push_line_escaped(entry.kind.as_str());
        output.push_char('\t');
        output.push_line_escaped(entry.repair_target_id.as_deref().unwrap_or("none"));
        output.push_char('\t');
        output.push_str(entry.storage_sequence.to_string().as_str());
        output.push_char('\t');
        output.push_str(entry.depth.to_string().as_str());
        output.push_char('\t');
        output.push_str(if entry.is_active_path { "active" } else { "sibling" });
        output.push_char('\t');
        output.push_line_escaped(entry.status.as_str());
        output.push_char('\t');
        output.push_line_escaped(entry.safe_summary.as_str());
        output.push_char('\n');
    }
    output.push_safe_line(RenderPair {
        label: "redaction",
        value: super::data::REDACTION_MARKER,
    });
    let rendered = output.into_string();
    debug_assert!(rendered.starts_with(SAFE_LINES_PREFIX));
    debug_assert!(rendered.ends_with('\n'));
    rendered
}
