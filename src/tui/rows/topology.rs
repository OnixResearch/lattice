//! Deterministic dependency ordering and frontier connector geometry.

type Outgoing = std::collections::BTreeMap<
    crate::engine::workflow::NodeId,
    std::collections::BTreeSet<crate::engine::workflow::NodeId>,
>;

pub(super) fn ordered_nodes(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<(Vec<(usize, crate::engine::workflow::NodeId)>, Outgoing), super::Error> {
    let mut indegree =
        workflow.graph.nodes().map(|node| (node.id, 0_usize)).collect::<std::collections::BTreeMap<_, _>>();
    let mut outgoing = Outgoing::new();
    for edge in workflow.graph.edges() {
        let Some(source) = workflow.graph.port_owner(edge.source) else {
            continue;
        };
        let Some(target) = workflow.graph.port_owner(edge.target) else {
            continue;
        };
        if source != target && outgoing.entry(source).or_default().insert(target) {
            let count = indegree.entry(target).or_default();
            *count = count.checked_add(1).ok_or(super::Error::DegreeOverflow)?;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<std::collections::BTreeSet<_>>();
    let mut depths = std::collections::BTreeMap::<crate::engine::workflow::NodeId, usize>::new();
    let mut order = Vec::with_capacity(indegree.len());
    while let Some(node_id) = ready.pop_first() {
        let depth = depths.get(&node_id).copied().unwrap_or(0);
        order.push((depth, node_id));
        if let Some(targets) = outgoing.get(&node_id) {
            let child_depth = depth.checked_add(1).ok_or(super::Error::DepthOverflow)?;
            for target in targets {
                depths.entry(*target).and_modify(|value| *value = (*value).max(child_depth)).or_insert(child_depth);
                if let Some(count) = indegree.get_mut(target) {
                    *count = count.checked_sub(1).ok_or(super::Error::DependencyUnderflow)?;
                    if *count == 0 {
                        ready.insert(*target);
                    }
                }
            }
        }
    }
    order.sort_unstable();
    // Invalid drafts still display every node, without claiming the cycle is a DAG.
    let visited = order.iter().map(|(_, id)| *id).collect::<std::collections::BTreeSet<_>>();
    debug_assert_eq!(visited.len(), order.len(), "each node enters the ready frontier once");
    for id in indegree.keys().filter(|id| !visited.contains(id)) {
        order.push((depths.get(id).copied().unwrap_or(0), *id));
    }
    debug_assert_eq!(order.len(), indegree.len(), "layout retains every node, including cyclic drafts");
    Ok((order, outgoing))
}

pub(super) struct Reservations {
    pub(super) occupants: Vec<Option<crate::engine::workflow::NodeId>>,
}

impl Reservations {
    pub(super) fn reserve(&mut self, node: crate::engine::workflow::NodeId) -> usize {
        if let Some(index) = self.occupants.iter().position(|value| *value == Some(node)) {
            return index;
        }
        if let Some(index) = self.occupants.iter().position(Option::is_none) {
            self.occupants[index] = Some(node);
            return index;
        }
        let index = self.occupants.len();
        self.occupants.push(Some(node));
        index
    }
}

pub(super) struct Connection {
    pub(super) source_lane: usize,
    pub(super) target_lane: usize,
}

pub(super) struct Frontier {
    max_lanes: usize,
    last_lane: usize,
    strokes: Vec<u8>,
}

impl Frontier {
    pub(super) fn new(max_lanes: usize) -> Result<Self, super::Error> {
        let last_lane = max_lanes.checked_sub(1).ok_or(super::Error::LaneGeometry)?;
        let columns =
            max_lanes.checked_mul(2).and_then(|width| width.checked_sub(1)).ok_or(super::Error::LaneGeometry)?;
        Ok(Self {
            max_lanes,
            last_lane,
            strokes: vec![0; columns],
        })
    }

    pub(super) fn visible_lane(&self, lane: usize) -> usize {
        lane.min(self.last_lane)
    }

    fn column(&self, lane: usize) -> Result<usize, super::Error> {
        self.visible_lane(lane).checked_mul(2).ok_or(super::Error::LaneGeometry)
    }

    pub(super) fn vertical(
        &mut self,
        lanes: &[Option<crate::engine::workflow::NodeId>],
        mask: u8,
    ) -> Result<(), super::Error> {
        for (lane, occupant) in lanes.iter().enumerate() {
            if occupant.is_some() {
                let column = self.column(lane)?;
                self.strokes[column] |= mask;
            }
        }
        Ok(())
    }

    pub(super) fn connect(&mut self, connection: Connection) -> Result<(), super::Error> {
        let from = self.column(connection.source_lane)?;
        let to = self.column(connection.target_lane)?;
        // Each adjacent pair receives both halves of the horizontal stroke.
        for column in from.min(to)..from.max(to) {
            let next = column.checked_add(1).ok_or(super::Error::LaneGeometry)?;
            self.strokes[column] |= 8; // right
            self.strokes[next] |= 4; // left
        }
        Ok(())
    }

    pub(super) fn finish(self, lane: usize) -> Result<String, super::Error> {
        let node_column = self.column(lane)?;
        let is_collapsed = lane >= self.max_lanes;
        let mut connectors = self
            .strokes
            .into_iter()
            .enumerate()
            .map(|(column, mask)| {
                if column == node_column {
                    if is_collapsed { '◆' } else { '●' }
                } else {
                    connector(mask)
                }
            })
            .collect::<String>();
        connectors.truncate(connectors.trim_end().len());
        Ok(connectors)
    }
}

fn connector(mask: u8) -> char {
    match mask {
        0 => ' ',
        1..=3 => '│',
        4 | 8 | 12 => '─',
        5 => '┘',
        6 => '┐',
        7 => '┤',
        9 => '└',
        10 => '┌',
        11 => '├',
        13 => '┴',
        14 => '┬',
        _ => '┼',
    }
}
