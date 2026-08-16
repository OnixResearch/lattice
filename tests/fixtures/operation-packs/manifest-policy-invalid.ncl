let contracts = import "../../../policy/operation-packs.ncl" in
let manifest = import "invalid-unknown-field/manifest.ncl" in
manifest | contracts.Manifest
