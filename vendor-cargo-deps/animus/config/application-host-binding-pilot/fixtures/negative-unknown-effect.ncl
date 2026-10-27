let profile = import "../profile.ncl" in
let base = profile.binding in
{
  expected_issue = "APPLICATION_BINDING_WRONG_EFFECT_DOMAIN",
  fixture | profile.binding_contract = base & {
    effect_operation_identities | force = [base.input_schema_identity],
  },
}
