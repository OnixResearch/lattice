pub(crate) const DIALOG_TYPST_ARTIFACT: &str = r#"#let fixture_id = "dialog.basic"
#let renderer = "subwayrat-typst"
#let logical_size = "80x24"

= Dialog basic

A centered modal dialog with title, body copy, primary action, and cancel action.

== States
- default: dialog is visible with primary action selected
- focused: focus ring highlights the primary action
- tiny viewport: frame clips content without panic

== Controls
- Enter accepts
- Esc closes
"#;
pub(crate) const DIALOG_DOM_ARTIFACT: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>Dialog basic snapshot</title>
<section data-fixture="dialog.basic" data-renderer="meteorite-dom" data-size="640x480">
  <dialog open aria-labelledby="dialog-basic-title">
    <h1 id="dialog-basic-title">Delete note?</h1>
    <p>This action keeps the deterministic fixture body visible.</p>
    <button data-focus="true">Delete</button>
    <button>Cancel</button>
  </dialog>
</section>
</html>
"#;
pub(crate) const CHROME_TYPST_ARTIFACT: &str = r#"#let fixture_id = "chrome.shell"
#let renderer = "subwayrat-typst"
#let logical_size = "80x24"

= Chrome shell

A terminal shell with title bar, menu bar, split content panes, status bar, and overlay slot.

== States
- default: editor and sidebar panes are visible
- menu-open: file menu overlays the title bar
- dialog-overlay: command dialog is centered above the shell

== Controls
- Alt+F opens the File menu
- Ctrl+K opens the command palette
- Esc closes the active overlay
"#;
pub(crate) const CHROME_DOM_ARTIFACT: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>Chrome shell snapshot</title>
<div data-fixture="chrome.shell" data-renderer="meteorite-dom" data-size="640x480" class="chrome-shell">
  <header role="banner">Rats Workspace <nav>File Edit View Help</nav></header>
  <main>
    <aside>Explorer</aside>
    <section aria-label="Editor">README.md</section>
  </main>
  <footer role="contentinfo">Ready · branch main · synced</footer>
  <div role="dialog" aria-label="Command palette">Open Command…</div>
</div>
</html>
"#;
pub(crate) const KNOWLEDGE_TYPST_ARTIFACT: &str = r#"#let fixture_id = "knowledge.workflow"
#let renderer = "subwayrat-typst"
#let logical_size = "80x24"

= Knowledge workflow

A shared knowledge workspace with capture, backlink, agenda, metadata, diagnostics, and graph facts.

== Panels
- graph: doc -> other
- agenda: 2026-06-11 Review
- metadata: documents=2,tags=1,backlinks=1
- diagnostics: valid workspace

== Controls
- Search knowledge graph
- Capture to Inbox
"#;
pub(crate) const KNOWLEDGE_DOM_ARTIFACT: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>Knowledge workflow snapshot</title>
<section data-fixture="knowledge.workflow" data-renderer="meteorite-dom" data-size="640x480" class="knowledge-workflow">
  <article aria-label="Graph panel" data-role="graph">doc -> other</article>
  <article aria-label="Agenda panel" data-role="agenda">2026-06-11 Review</article>
  <dl aria-label="Metadata summary">
    <dt>documents</dt><dd>2</dd>
    <dt>tags</dt><dd>1</dd>
    <dt>backlinks</dt><dd>1</dd>
  </dl>
  <output aria-label="Diagnostics">valid workspace</output>
</section>
</html>
"#;
pub(crate) const THEME_TYPST_ARTIFACT: &str = r##"#let fixture_id = "theme.tokens"
#let renderer = "subwayrat-typst"
#let logical_size = "80x24"

= Theme tokens

Semantic role preview for terminal renderers.

== rats-terminal-dark
- primary #5ea1ff
- surface #1b1f27
- text #f4f4f5
- success #22c55e
- warning #facc15
- error #ef4444

== rats-light
- primary #2563eb
- surface #f8fafc
- text #111827
- success #16a34a
- warning #d97706
- error #dc2626
"##;
pub(crate) const THEME_DOM_ARTIFACT: &str = r##"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>Theme tokens snapshot</title>
<style>
:root[data-rats-theme="rats-terminal-dark"] {
  --rats-primary: #5ea1ff;
  --rats-secondary: #66d9ef;
  --rats-background: #101216;
  --rats-surface: #1b1f27;
  --rats-text: #f4f4f5;
  --rats-text-muted: #9ca3af;
  --rats-success: #22c55e;
  --rats-warning: #facc15;
  --rats-error: #ef4444;
}
:root[data-rats-theme="rats-light"] {
  --rats-primary: #2563eb;
  --rats-secondary: #0891b2;
  --rats-background: #ffffff;
  --rats-surface: #f8fafc;
  --rats-text: #111827;
  --rats-text-muted: #6b7280;
  --rats-success: #16a34a;
  --rats-warning: #d97706;
  --rats-error: #dc2626;
}
</style>
<section data-fixture="theme.tokens" data-renderer="meteorite-dom" data-size="640x480">
  <article data-rats-theme="rats-terminal-dark">Dark semantic preview</article>
  <article data-rats-theme="rats-light">Light semantic preview</article>
</section>
</html>
"##;
