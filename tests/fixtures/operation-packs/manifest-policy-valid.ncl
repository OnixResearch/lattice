let contracts = import "../../../policy/operation-packs.ncl" in
let manifest = import "valid/manifest.ncl" in
manifest | contracts.Manifest
