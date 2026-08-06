let policy = import "../../session-branch-explorer-policy.ncl" in
let invalid = policy.session_branch_explorer_policy & { destructive_actions_allowed = true } in
invalid | policy.session_branch_explorer_policy_contract
