# Seed runbook-catalog fixture bundle; purpose-made for catalog tests.
{
  schema_version = 1,
  id = "status-runbook",
  display_name = "Status runbook",
  description = "Report the working directory of a reviewed run.",
  category = "runbooks",
  workflow = {
    path = "workflow.ncl",
    blake3 = "b9d01b44414acbbb212705d6804b56d4199fd920ea60116d7e5d6409541deef8",
  },
  assets = [
    {
      path = "assets/notes.txt",
      media_kind = "text/plain",
      blake3 = "e3acb563cd78594bb77f0ef5a067ae4358a052aa9f76a15cba87374620ed8e7f",
      required = true,
    },
  ],
  compatibility = {
    workflow_schema = 1,
    node_kinds = ["shell"],
    runtime_kinds = ["shell"],
    feature_ids = ["lattice.workflow-template-bundle.v1"],
  },
  operator_notes = ["Installation and preview grant no effect authority."],
  non_claims = ["Bundle identity proves exact reviewed bytes only."],
}
