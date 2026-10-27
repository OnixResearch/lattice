## Tasks

- [x] [serial] r[basalt.ast_grep_policy_audits.profile] Define ast-grep policy-boundary audit profiles and release posture.
- [x] [depends:ast-grep-policy-profile] r[basalt.ast_grep_policy_audits.identity] Bind tool identity, rule bundle BLAKE3 hash, scan scope, release verifier identity, and non-claims into receipts.
- [x] [depends:ast-grep-policy-profile] r[basalt.ast_grep_policy_audits.fixtures] Add positive and negative fixtures for policy-boundary source shapes before any rule becomes blocking.
- [x] [depends:ast-grep-policy-fixtures] r[basalt.ast_grep_policy_audits.release_receipts] Report ast-grep audit results in release verification receipts with candidate-evidence non-claims.
- [x] [depends:ast-grep-policy-release-receipts] r[basalt.ast_grep_policy_audits.shell_boundary] Keep ast-grep process execution and file scans in release verifier shell code.
- [x] [depends:ast-grep-policy-shell-boundary] r[basalt.ast_grep_policy_audits.validation] Run ast-grep rule tests, fixture scans, release verifier receipt checks, Cairn gates, and focused Basalt release rails.
