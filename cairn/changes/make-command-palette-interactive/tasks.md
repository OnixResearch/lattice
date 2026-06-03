## Phase 1

- [ ] [serial] r[n8nclone.tui_ux.command_palette.interactive] Route palette-open key events to palette input handling before focused-panel handling.
- [ ] [serial] r[n8nclone.tui_ux.command_palette.execute] Execute the selected palette command through `AppAction` and close the palette after success.
- [ ] [parallel] r[n8nclone.tui_ux.command_palette.escape] Add tests for Escape closing the palette without executing an action.
- [ ] [parallel] r[n8nclone.tui_ux.command_palette.no_match] Add tests for Enter with no selectable match leaving workflow state unchanged.
