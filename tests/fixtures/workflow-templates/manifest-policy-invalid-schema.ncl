let contract = import "../../../policy/workflow-template-bundle.ncl" in
(std.record.remove "params" (std.record.update "schema_version" 1 (import "manifest-base.ncl"))) | contract
