## 1. Core credential evidence completion

- [x] [serial] Establish focused credential, store, history, inventory, and audit baselines. r[n8nclone.credentials.secret_refs.model]
- [x] [serial] Add typed redaction-safe plaintext import rejection before store access. r[n8nclone.credentials.secret_refs.model.reject_plaintext]
- [x] [serial] Make credential redaction overlap-safe and prove all maintained output/persistence surfaces. r[n8nclone.credentials.redaction.output]
- [x] [serial] Add exact save/export and validation-rendering assertions. r[n8nclone.credentials.secret_refs.model.store_ref]
- [x] [serial] Reclassify all 12 audited rows as exact while preserving missing coverage. [depends:lattice.repository_validation.credential_traceability_audit]
- [x] [serial] Run repository, Cairn, Octet, and Nix validation; archive evidence and commit. r[n8nclone.credentials.redaction]
