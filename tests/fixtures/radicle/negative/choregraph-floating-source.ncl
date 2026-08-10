(import "../choregraph-source-admission-v1.ncl")
& {
  admission.revision | force = "main",
  admission.source_url | force = "https://github.com/onixresearch/choregraph",
  admission.evidence.single_step_proof | force = false,
}
