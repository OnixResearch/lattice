let schema = import "../../pueue-host.ncl" in
{
  expected_issue = "PUEUE_LOG_LINE_BOUND",
  fixture | schema.pueue_host_contract = schema.pueue_host & {
    bounds | force = schema.pueue_host.bounds & {
      max_log_lines | force = 0,
    },
  },
}
