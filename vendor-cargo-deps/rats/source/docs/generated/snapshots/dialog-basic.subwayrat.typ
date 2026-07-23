#let fixture_id = "dialog.basic"
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
