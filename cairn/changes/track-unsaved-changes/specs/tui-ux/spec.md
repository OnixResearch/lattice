## ADDED Requirements

### Requirement: Unsaved-change tracking
r[n8nclone.tui_ux.unsaved.dirty_indicator] The system MUST visibly indicate when the current workflow has unsaved changes.

#### Scenario: Mutation marks workflow unsaved
r[n8nclone.tui_ux.unsaved.mutation_marks_dirty]
- GIVEN the current workflow is saved or newly clean
- WHEN the user mutates the workflow
- THEN the system marks the workflow as unsaved and displays an unsaved indicator

#### Scenario: Successful save clears unsaved indicator
r[n8nclone.tui_ux.unsaved.save_clears]
- GIVEN the current workflow has unsaved changes
- WHEN the workflow saves successfully
- THEN the system clears the unsaved indicator

#### Scenario: Quit with unsaved changes asks for confirmation
r[n8nclone.tui_ux.unsaved.quit_confirm]
- GIVEN the current workflow has unsaved changes
- WHEN the user requests quit
- THEN the system asks for confirmation before exiting

#### Scenario: Failed save keeps workflow dirty
r[n8nclone.tui_ux.unsaved.failed_save]
- GIVEN the current workflow has unsaved changes
- WHEN saving fails
- THEN the system reports the failure and keeps the unsaved indicator visible
