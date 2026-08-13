#![allow(
    unknown_lints,
    non_trait_imports,
    numeric_units,
    reason = "this focused graph mutation component uses local model imports and unitless port identities"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::model::Edge;
use super::model::GraphError;
use super::model::Node;
use super::model::NodeId;
use super::model::Port;
use super::model::PortDirection;
use super::model::PortId;
use super::model::PortSpec;
use super::model::WorkflowGraph;
use super::model::WorkflowNodeIdentity;
use super::model::WorkflowPortIdentity;

const ANY_TYPE_TAG: &str = "any";

#[derive(Debug)]
struct PortReconciliation {
    ports: Vec<Port>,
    removed_or_changed: BTreeSet<PortId>,
    next_port_identity: u64,
}

impl WorkflowGraph {
    #[allow(
        unknown_lints,
        no_unwrap,
        reason = "this compatibility API accepts admitted built-in declarations; fallible adapters use try_add_node"
    )]
    pub fn add_node(&mut self, label: impl Into<String>, inputs: &[PortSpec], outputs: &[PortSpec]) -> NodeId {
        self.try_add_node(label, inputs, outputs)
            .expect("Lattice-owned node templates must contain valid bounded port declarations")
    }

    pub fn try_add_node(
        &mut self,
        label: impl Into<String>,
        inputs: &[PortSpec],
        outputs: &[PortSpec],
    ) -> Result<NodeId, GraphError> {
        validate_unique_labels(inputs)?;
        validate_unique_labels(outputs)?;

        let node_id = NodeId(self.next_node_identity);
        let following_node_identity = self.next_node_identity.checked_add(1).ok_or(GraphError::NodeIdExhausted)?;
        let input_ports = allocate_ports(inputs, PortDirection::Input, self.next_port_identity)?;
        let output_start_identity = advance_port_identity(self.next_port_identity, inputs.len())?;
        let output_ports = allocate_ports(outputs, PortDirection::Output, output_start_identity)?;
        let following_port_identity = advance_port_identity(output_start_identity, outputs.len())?;

        let node = Node {
            id: node_id,
            label: label.into(),
            x: 0,
            y: 0,
            input_ports,
            output_ports,
        };
        for port in node.input_ports.iter().chain(&node.output_ports) {
            let prior = self.port_owner.insert(port.id, node_id);
            debug_assert!(prior.is_none());
        }
        let prior = self.nodes.insert(node_id, node);
        debug_assert!(prior.is_none());
        self.next_node_identity = following_node_identity;
        self.next_port_identity = following_port_identity;
        Ok(node_id)
    }

    pub fn remove_node(&mut self, node_id: impl WorkflowNodeIdentity) -> Result<(), GraphError> {
        let node_id = node_id.workflow_node_id();
        let node = self.nodes.remove(&node_id).ok_or(GraphError::NodeNotFound(node_id))?;
        let port_ids = node.input_ports.iter().chain(&node.output_ports).map(|port| port.id).collect::<BTreeSet<_>>();
        self.edges.retain(|edge| !port_ids.contains(&edge.source) && !port_ids.contains(&edge.target));
        for port_id in port_ids {
            self.port_owner.remove(&port_id);
        }
        Ok(())
    }

    pub fn reconcile_node_ports(
        &mut self,
        node_id: impl WorkflowNodeIdentity,
        direction: PortDirection,
        requested: &[PortSpec],
    ) -> Result<Vec<PortId>, GraphError> {
        let node_id = node_id.workflow_node_id();
        let existing = {
            let node = self.nodes.get(&node_id).ok_or(GraphError::NodeNotFound(node_id))?;
            match direction {
                PortDirection::Input => node.input_ports.clone(),
                PortDirection::Output => node.output_ports.clone(),
            }
        };
        let reconciliation = reconcile_ports(&existing, requested, direction, self.next_port_identity)?;
        let replacement_ids = reconciliation.ports.iter().map(|port| port.id).collect::<Vec<_>>();

        let node = self.nodes.get_mut(&node_id).ok_or(GraphError::NodeNotFound(node_id))?;
        match direction {
            PortDirection::Input => node.input_ports.clone_from(&reconciliation.ports),
            PortDirection::Output => node.output_ports.clone_from(&reconciliation.ports),
        }
        for prior in existing {
            self.port_owner.remove(&prior.id);
        }
        for port in &reconciliation.ports {
            self.port_owner.insert(port.id, node_id);
        }
        self.edges.retain(|edge| {
            !reconciliation.removed_or_changed.contains(&edge.source)
                && !reconciliation.removed_or_changed.contains(&edge.target)
        });
        self.next_port_identity = reconciliation.next_port_identity;
        Ok(replacement_ids)
    }

    pub fn add_edge(
        &mut self,
        source: impl WorkflowPortIdentity,
        target: impl WorkflowPortIdentity,
    ) -> Result<(), GraphError> {
        let source = source.workflow_port_id();
        let target = target.workflow_port_id();
        let source_port = self.port(source).ok_or(GraphError::PortNotFound(source))?;
        let target_port = self.port(target).ok_or(GraphError::PortNotFound(target))?;
        if source_port.direction != PortDirection::Output || target_port.direction != PortDirection::Input {
            return Err(GraphError::WrongDirection);
        }
        let source_node = self.port_owner.get(&source).copied().ok_or(GraphError::PortNotFound(source))?;
        let target_node = self.port_owner.get(&target).copied().ok_or(GraphError::PortNotFound(target))?;
        if source_node == target_node {
            return Err(GraphError::SelfLoop);
        }
        let has_compatible_type_tags = source_port.type_tag == target_port.type_tag
            || source_port.type_tag == ANY_TYPE_TAG
            || target_port.type_tag == ANY_TYPE_TAG;
        if !has_compatible_type_tags {
            return Err(GraphError::IncompatibleTypes {
                source_tag: source_port.type_tag.clone(),
                target_tag: target_port.type_tag.clone(),
            });
        }
        if self.dag_mode && self.path_exists(target_node, source_node) {
            return Err(GraphError::WouldCycle);
        }
        self.edges.push(Edge { source, target });
        Ok(())
    }

    pub fn remove_edge(
        &mut self,
        source: impl WorkflowPortIdentity,
        target: impl WorkflowPortIdentity,
    ) -> Result<(), GraphError> {
        let source = source.workflow_port_id();
        let target = target.workflow_port_id();
        let index = self
            .edges
            .iter()
            .position(|edge| edge.source == source && edge.target == target)
            .ok_or(GraphError::EdgeNotFound { source, target })?;
        self.edges.remove(index);
        Ok(())
    }

    fn path_exists(&self, start: NodeId, target: NodeId) -> bool {
        let mut pending = vec![start];
        let mut visited = BTreeSet::new();
        while let Some(node_id) = pending.pop() {
            if node_id == target {
                return true;
            }
            if !visited.insert(node_id) {
                continue;
            }
            let Some(node) = self.nodes.get(&node_id) else {
                continue;
            };
            let output_ids = node.output_ports.iter().map(|port| port.id).collect::<BTreeSet<_>>();
            for edge in &self.edges {
                if output_ids.contains(&edge.source)
                    && let Some(next_node) = self.port_owner.get(&edge.target)
                {
                    pending.push(*next_node);
                }
            }
        }
        false
    }
}

fn validate_unique_labels(specs: &[PortSpec]) -> Result<(), GraphError> {
    let mut labels = BTreeSet::new();
    for spec in specs {
        if !labels.insert(spec.label.as_str()) {
            return Err(GraphError::DuplicatePortLabel(spec.label.clone()));
        }
    }
    Ok(())
}

fn allocate_ports(
    specs: &[PortSpec],
    direction: PortDirection,
    first_port_identity: u64,
) -> Result<Vec<Port>, GraphError> {
    let following_port_identity = advance_port_identity(first_port_identity, specs.len())?;
    debug_assert!(following_port_identity >= first_port_identity);
    specs
        .iter()
        .enumerate()
        .map(|(port_offset_index, spec)| {
            let port_offset_index = u64::try_from(port_offset_index).map_err(|_| GraphError::PortIdExhausted)?;
            let raw_identity = first_port_identity.checked_add(port_offset_index).ok_or(GraphError::PortIdExhausted)?;
            Ok(Port {
                id: PortId(raw_identity),
                label: spec.label.clone(),
                type_tag: spec.type_tag.clone(),
                direction,
            })
        })
        .collect()
}

fn advance_port_identity(first_port_identity: u64, additional_ports: usize) -> Result<u64, GraphError> {
    let additional_ports = u64::try_from(additional_ports).map_err(|_| GraphError::PortIdExhausted)?;
    first_port_identity.checked_add(additional_ports).ok_or(GraphError::PortIdExhausted)
}

fn reconcile_ports(
    existing: &[Port],
    requested: &[PortSpec],
    direction: PortDirection,
    next_port_identity: u64,
) -> Result<PortReconciliation, GraphError> {
    validate_unique_labels(requested)?;
    let existing_by_label = existing.iter().map(|port| (port.label.as_str(), port)).collect::<BTreeMap<_, _>>();
    if existing_by_label.len() != existing.len() {
        let duplicate = existing
            .iter()
            .find(|candidate| existing.iter().filter(|port| port.label == candidate.label).count() > 1)
            .map_or_else(String::new, |port| port.label.clone());
        return Err(GraphError::DuplicatePortLabel(duplicate));
    }

    let requested_labels = requested.iter().map(|spec| spec.label.as_str()).collect::<BTreeSet<_>>();
    let new_port_total = requested.iter().filter(|spec| !existing_by_label.contains_key(spec.label.as_str())).count();
    let following_port_identity = advance_port_identity(next_port_identity, new_port_total)?;
    let mut fresh_port_identity = next_port_identity;
    let mut ports = Vec::with_capacity(requested.len());
    let mut removed_or_changed = existing
        .iter()
        .filter(|port| !requested_labels.contains(port.label.as_str()))
        .map(|port| port.id)
        .collect::<BTreeSet<_>>();

    for spec in requested {
        if let Some(prior) = existing_by_label.get(spec.label.as_str()) {
            if prior.type_tag != spec.type_tag {
                removed_or_changed.insert(prior.id);
            }
            ports.push(Port {
                id: prior.id,
                label: spec.label.clone(),
                type_tag: spec.type_tag.clone(),
                direction,
            });
        } else {
            ports.push(Port {
                id: PortId(fresh_port_identity),
                label: spec.label.clone(),
                type_tag: spec.type_tag.clone(),
                direction,
            });
            fresh_port_identity = fresh_port_identity.checked_add(1).ok_or(GraphError::PortIdExhausted)?;
        }
    }
    debug_assert_eq!(fresh_port_identity, following_port_identity);
    Ok(PortReconciliation {
        ports,
        removed_or_changed,
        next_port_identity: following_port_identity,
    })
}
