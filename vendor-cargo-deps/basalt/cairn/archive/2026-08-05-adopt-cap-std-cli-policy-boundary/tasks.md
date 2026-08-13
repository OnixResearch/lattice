## Tasks

- [x] [serial] Add `cap-std` only to Basalt CLI-feature code that owns filesystem effects. r[crate_consumability.cap_std_cli_policy_boundary.dependency]
- [x] [serial] Introduce typed capability roots for policy input, generated policy, and receipt/output directories. r[crate_consumability.cap_std_cli_policy_boundary.root_wrappers]
- [x] [serial] Convert CLI policy reads and receipt/output writes to capability-relative operations while preserving no-default library builds. r[crate_consumability.cap_std_cli_policy_boundary.conversion]
- [x] [serial] Add positive fixtures for valid policy reads, packaged policy fallback, and output writes under declared roots. r[crate_consumability.cap_std_cli_policy_boundary.tests.positive]
- [x] [serial] Add negative fixtures for traversal, absolute paths, symlink escapes, missing roots, and output escape attempts. r[crate_consumability.cap_std_cli_policy_boundary.tests.negative]
- [x] [serial] Document the CLI filesystem-authority boundary and policy/UCAN non-claims. r[crate_consumability.cap_std_cli_policy_boundary.docs]
- [x] [serial] Run focused CLI/no-default tests plus Cairn validation and gates. r[crate_consumability.cap_std_cli_policy_boundary.validation]
