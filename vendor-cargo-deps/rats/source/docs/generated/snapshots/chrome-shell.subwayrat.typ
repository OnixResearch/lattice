#let fixture_id = "chrome.shell"
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
