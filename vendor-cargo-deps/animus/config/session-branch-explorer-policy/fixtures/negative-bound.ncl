let policy = import "../../session-branch-explorer-policy.ncl" in
let invalid = policy.session_branch_explorer_policy & { max_nodes = 0 } in
invalid | policy.session_branch_explorer_policy_contract
