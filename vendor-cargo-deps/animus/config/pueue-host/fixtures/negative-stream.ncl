let schema = import "../../pueue-host.ncl" in
{
  expected_issue = "PUEUE_STREAM_UNSUPPORTED",
  fixture | schema.pueue_host_contract = schema.pueue_host & {
    capabilities | force = {
      interactions = ["wait", "stream"],
    },
  },
}
