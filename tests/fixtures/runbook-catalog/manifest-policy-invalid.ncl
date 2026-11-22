let contracts = import "../../../policy/operation-packs.ncl" in
let manifest = import "packs/seed-runbooks/manifest.ncl" in
(manifest & { workflow_bundles | force = [{ id = "hello-runbook", bundle_identity = "b3:short" }] }) | contracts.Manifest
