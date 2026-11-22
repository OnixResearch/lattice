let contract = import "../../../policy/workflow-template-bundle.ncl" in
(std.record.update "mode" "readonly" (import "manifest-base.ncl")) | contract
