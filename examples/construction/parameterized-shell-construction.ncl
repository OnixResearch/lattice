# Explicit construction parameters, independently scoped instances and finite generation.
let g = import "./choregraph/graph-construction-v1.ncl" in
{
  parameters = {extra = false, first = "printf 'first\\n'", second = "printf 'second\\n'", worker = "printf 'worker\\n'"},
  build = fun p =>
    let effect = [{
      key = "invoke", from_state = "ready", action = "start",
      resource = "lattice://process/construction-shell-v1", ability = "process/execute",
      request = g.slot_binding "dispatch-request" "invocation" "dispatch-request",
      payload = g.slot_binding "resolved-payload" "invocation" "resolved-payload",
    }] in
    let stage = g.template "stage" ["command"]
      (g.operation "body" "shell-source" {command = g.parameter "command"} effect) in
    let labels = if p.extra then ["a", "b", "c"] else ["a", "b"] in
    let generated = g.generate 3 labels (fun label =>
      g.instantiate label "stage" {command = g.literal p.worker}) in
    g.finish "param-shell" "lattice-local-shell-v1" "lattice-local-shell-registry-v1"
      {} [stage] (g.wire "root" ([
        g.instantiate "first" "stage" {command = g.literal p.first},
        g.instantiate "second" "stage" {command = g.literal p.second},
      ] @ generated) [] [] []),
}
