# Captured construction program for the Lattice local shell profile.
# This source is inert until an operator selects a separate exact v3 effect grant.
let g = import "./choregraph/graph-construction-v1.ncl" in
let effect = [{
  key = "invoke", from_state = "ready", action = "start",
  resource = "lattice://process/construction-shell-v1", ability = "process/execute",
  request = g.slot_binding "dispatch-request" "invocation" "dispatch-request",
  payload = g.slot_binding "resolved-payload" "invocation" "resolved-payload",
}] in
let worker = g.template "worker" ["command"]
  (g.wire "stage" [
    g.with_origin
      (g.operation "run" "shell-stdin" {command = g.parameter "command"} effect)
      {template = "worker", location = {logical_module = "local-shell-construction.ncl", start = 624, end = 696}},
  ] []
  [g.expose "stdin" (g.input "local-shell-demo" "run" "stdin")]
  [g.expose "stdout" (g.output "local-shell-demo" "run" "stdout")]) in
let make_worker = fun key => g.instantiate key "worker" {command = g.literal "cat"} in
let generated = g.generate 2 ["worker-a", "worker-b"] make_worker in
let fork = g.operation "fork" "shell-source" {command = g.literal "printf 'alpha\\n'"} effect in
let source = g.output "local-shell-demo" "fork" "stdout" in
let group = g.wire "fanout" ([fork, make_worker "left", make_worker "right"] @ generated)
  [
    g.connect source (g.input "local-shell-demo" "left" "stdin"),
    g.connect source (g.input "local-shell-demo" "right" "stdin"),
    g.connect source (g.input "local-shell-demo" "worker-a" "stdin"),
    g.connect source (g.input "local-shell-demo" "worker-b" "stdin"),
  ] [] [] in
let join = g.wire "join" [
  g.operation "done" "shell-source" {command = g.literal "printf 'joined\\n'"} effect,
] [] [] [] in
g.finish "local-shell-demo" "lattice-local-shell-v1" "lattice-local-shell-registry-v1"
  {} [worker] (g.sequence "root" [group, join])
