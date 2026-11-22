let contracts = import "../../../policy/operation-packs.ncl" in
let manifest = import "packs/seed-runbooks/manifest.ncl" in
manifest | contracts.Manifest
