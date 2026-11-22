# Seed runbook-catalog fixture bundle; purpose-made for catalog tests.
{
  schema_version = 1,
  id = "hello-runbook",
  display_name = "Hello runbook",
  description = "Print one fixed line from the seed runbook catalog.",
  category = "runbooks",
  workflow = {
    path = "workflow.ncl",
    blake3 = "c638275fb5cc6abc49f47750368647f2ec2940c227493c91ed2ff85cf75fa75e",
  },
  assets = [],
  compatibility = {
    workflow_schema = 1,
    node_kinds = ["shell"],
    runtime_kinds = ["shell"],
    feature_ids = ["lattice.workflow-template-bundle.v1"],
  },
  operator_notes = ["Installation and preview grant no effect authority."],
  non_claims = ["Bundle identity proves exact reviewed bytes only."],
}
