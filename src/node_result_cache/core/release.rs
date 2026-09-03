//! Pure deferred-release plan.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub keys: Vec<super::Key>,
}

// r[impl lattice.node_result_cache.deferral]
pub fn plan(keys: Vec<super::Key>) -> Plan {
    Plan { keys }
}
