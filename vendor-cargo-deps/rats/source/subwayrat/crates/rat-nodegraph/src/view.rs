//! Ratatui rendering and interaction for the node graph.
//!
//! Provides [`NodeGraphWidget`] (a `StatefulWidget`) and [`NodeGraphState`] that
//! hold the graph, viewport, selection, and interaction state. Input is handled
//! via [`NodeGraphState::handle_mouse`] and [`NodeGraphState::handle_key`], which
//! return [`GraphAction`] events for the caller to act on.

use crate::model::{Graph, NodeId, PortDirection, PortId};
use rat_canvas::{Position, Viewport};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::StatefulWidget;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// Events produced by interaction handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphAction {
    SelectionChanged { selected: Vec<NodeId> },
    NodeMoved { node: NodeId, x: i32, y: i32 },
    EdgeCreated { source: PortId, target: PortId },
    EdgeDeleted { source: PortId, target: PortId },
    WiringStarted { source: PortId },
    WiringCancelled,
}

// ---------------------------------------------------------------------------
// Interaction state
// ---------------------------------------------------------------------------

/// Tracks transient interaction mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionMode {
    /// Default idle state.
    Normal,
    /// Dragging selected nodes. `origin` is the canvas pos where drag started.
    Dragging { origin_x: i32, origin_y: i32 },
    /// Drawing a wire from a source port.
    Wiring {
        source: PortId,
        cursor_x: i32,
        cursor_y: i32,
    },
    /// Box-selecting with a rectangle.
    BoxSelect {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
}

// ---------------------------------------------------------------------------
// Color mapping
// ---------------------------------------------------------------------------

/// Default port-type → color palette.
pub fn default_type_color(type_tag: &str) -> Color {
    match type_tag {
        "string" => Color::Cyan,
        "number" => Color::Green,
        "bool" | "boolean" => Color::Yellow,
        "any" => Color::White,
        "error" => Color::Red,
        "object" | "json" => Color::Magenta,
        _ => Color::Gray,
    }
}

/// Type alias for custom color mapping function.
pub type TypeColorFn = Box<dyn Fn(&str) -> Color + Send + Sync>;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Full state for the node graph widget.
pub struct NodeGraphState {
    pub graph: Graph,
    pub viewport: Viewport,
    pub selected: HashSet<NodeId>,
    pub focused: Option<NodeId>,
    pub focused_port: Option<PortId>,
    pub mode: InteractionMode,
    /// Last rendered area (set by widget during render).
    pub area: Rect,
    /// Selected edge (for deletion).
    pub selected_edge: Option<(PortId, PortId)>,
    /// Tab-order cache of node IDs (rebuilt on render).
    tab_order: Vec<NodeId>,
}

impl NodeGraphState {
    pub fn new(graph: Graph, width: u16, height: u16) -> Self {
        Self {
            graph,
            viewport: Viewport::new(width, height),
            selected: HashSet::new(),
            focused: None,
            focused_port: None,
            mode: InteractionMode::Normal,
            area: Rect::default(),
            selected_edge: None,
            tab_order: Vec::new(),
        }
    }

    /// Rebuild the tab-order list from current graph nodes.
    fn rebuild_tab_order(&mut self) {
        self.tab_order = self.graph.node_ids();
        self.tab_order.sort_by_key(|id| {
            let n = self.graph.node(*id).unwrap();
            (n.y, n.x)
        });
    }

    /// Apply a graph action, performing the actual mutation.
    /// Call this after handle_mouse_click / handle_key / handle_mouse_drag
    /// to commit the returned intents.
    pub fn apply_action(&mut self, action: &GraphAction) {
        match action {
            GraphAction::EdgeCreated { source, target } => {
                let _ = self.graph.add_edge(*source, *target);
            }
            GraphAction::EdgeDeleted { source, target } => {
                let _ = self.graph.remove_edge(*source, *target);
            }
            GraphAction::NodeMoved { node, x, y } => {
                if let Some(n) = self.graph.node_mut(*node) {
                    n.x = *x;
                    n.y = *y;
                }
            }
            // Selection changes and wiring state are view-layer, no graph mutation needed
            GraphAction::SelectionChanged { .. }
            | GraphAction::WiringStarted { .. }
            | GraphAction::WiringCancelled => {}
        }
    }

    /// Apply all actions from a handler result.
    pub fn apply_actions(&mut self, actions: &[GraphAction]) {
        for action in actions {
            self.apply_action(action);
        }
    }
}

// ---------------------------------------------------------------------------
// Node bounding box
// ---------------------------------------------------------------------------

/// Computed bounding box for a rendered node (in canvas coordinates).
#[derive(Debug, Clone, Copy)]
pub struct NodeBounds {
    pub node_id: NodeId,
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl NodeBounds {
    pub fn contains(&self, cx: i32, cy: i32) -> bool {
        cx >= self.x
            && cx < self.x + self.width as i32
            && cy >= self.y
            && cy < self.y + self.height as i32
    }
}

/// Compute the bounding box for a node based on its label and port count.
pub fn node_bounds(graph: &Graph, node_id: NodeId) -> Option<NodeBounds> {
    let node = graph.node(node_id)?;

    // Width: max of label length, longest input label + longest output label + padding.
    let label_width = node.label.len() as u16 + 4; // "│ Label │"
    let max_input_label = node
        .input_ports
        .iter()
        .map(|p| p.label.len() as u16)
        .max()
        .unwrap_or(0);
    let max_output_label = node
        .output_ports
        .iter()
        .map(|p| p.label.len() as u16)
        .max()
        .unwrap_or(0);
    // "│● label    label ●│" — marker(2) + label + gap(3) + label + marker(2) + borders(2)
    let port_row_width = 2 + max_input_label + 3 + max_output_label + 2 + 2;

    let width = label_width.max(port_row_width).max(10);

    // Height: border top + label row + separator + max(input_ports, output_ports) + border bottom.
    let port_rows = node.input_ports.len().max(node.output_ports.len()) as u16;
    let height = 3 + port_rows; // top border + label + separator + ports (bottom border shares last line)
    let height = height.max(4); // minimum 4 rows

    Some(NodeBounds {
        node_id,
        x: node.x,
        y: node.y,
        width,
        height,
    })
}

/// Compute bounding boxes for all nodes.
pub fn all_node_bounds(graph: &Graph) -> Vec<NodeBounds> {
    graph
        .node_ids()
        .iter()
        .filter_map(|&id| node_bounds(graph, id))
        .collect()
}

// ---------------------------------------------------------------------------
// Port position helpers
// ---------------------------------------------------------------------------

/// Screen position of a port marker (in canvas coordinates).
pub fn port_canvas_position(graph: &Graph, port_id: PortId) -> Option<(i32, i32)> {
    let node_id = graph.port_owner(port_id)?;
    let bounds = node_bounds(graph, node_id)?;
    let node = graph.node(node_id)?;

    let port = graph.port(port_id)?;

    let (port_list, side_x) = match port.direction {
        PortDirection::Input => (&node.input_ports, bounds.x),
        PortDirection::Output => (&node.output_ports, bounds.x + bounds.width as i32 - 1),
    };

    let port_index = port_list.iter().position(|p| p.id == port_id)?;
    let port_y = bounds.y + 2 + port_index as i32; // skip top border + label row

    Some((side_x, port_y))
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/// What a canvas position hits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    Node(NodeId),
    Port(PortId),
    Edge(PortId, PortId),
    Empty,
}

/// Hit-test a canvas position against nodes, ports, and edges.
pub fn hit_test(state: &NodeGraphState, canvas_x: i32, canvas_y: i32) -> HitTarget {
    let bounds_list = all_node_bounds(&state.graph);

    // Check ports first (they're on node edges, more specific).
    for bounds in &bounds_list {
        let node = state.graph.node(bounds.node_id).unwrap();

        // Input ports — left edge.
        for (i, port) in node.input_ports.iter().enumerate() {
            let px = bounds.x;
            let py = bounds.y + 2 + i as i32;
            if canvas_x == px && canvas_y == py {
                return HitTarget::Port(port.id);
            }
        }

        // Output ports — right edge.
        for (i, port) in node.output_ports.iter().enumerate() {
            let px = bounds.x + bounds.width as i32 - 1;
            let py = bounds.y + 2 + i as i32;
            if canvas_x == px && canvas_y == py {
                return HitTarget::Port(port.id);
            }
        }
    }

    // Check node body.
    for bounds in &bounds_list {
        if bounds.contains(canvas_x, canvas_y) {
            return HitTarget::Node(bounds.node_id);
        }
    }

    // Check edges (coarse — test if point is near any edge segment).
    for edge in state.graph.edges() {
        if let (Some((sx, sy)), Some((tx, ty))) = (
            port_canvas_position(&state.graph, edge.source),
            port_canvas_position(&state.graph, edge.target),
        ) && point_near_manhattan_path(canvas_x, canvas_y, sx, sy, tx, ty)
        {
            return HitTarget::Edge(edge.source, edge.target);
        }
    }

    HitTarget::Empty
}

/// Check if a point is within 1 cell of a Manhattan path between two points.
fn point_near_manhattan_path(px: i32, py: i32, sx: i32, sy: i32, tx: i32, ty: i32) -> bool {
    let mid_x = (sx + tx) / 2;

    // Horizontal segment from source.
    if py == sy && px >= sx.min(mid_x) && px <= sx.max(mid_x) {
        return true;
    }
    // Vertical segment.
    if px == mid_x && py >= sy.min(ty) && py <= sy.max(ty) {
        return true;
    }
    // Horizontal segment to target.
    if py == ty && px >= mid_x.min(tx) && px <= mid_x.max(tx) {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

impl NodeGraphState {
    /// Handle a mouse click at screen coordinates. Returns actions.
    pub fn handle_mouse_click(
        &mut self,
        screen_x: u16,
        screen_y: u16,
        shift: bool,
    ) -> Vec<GraphAction> {
        let canvas_pos = self.viewport.screen_to_canvas(
            screen_x.saturating_sub(self.area.x),
            screen_y.saturating_sub(self.area.y),
        );
        let cx = canvas_pos.x;
        let cy = canvas_pos.y;

        let hit = hit_test(self, cx, cy);
        let mut actions = Vec::new();

        match (&self.mode, hit) {
            // Wiring mode — clicking a port completes the wire.
            (InteractionMode::Wiring { source, .. }, HitTarget::Port(target_port)) => {
                let source = *source;

                // Check compatibility without mutating the graph
                if let (Some(src_port), Some(tgt_port)) =
                    (self.graph.port(source), self.graph.port(target_port))
                {
                    // Verify source is Output, target is Input
                    if src_port.direction == PortDirection::Output
                        && tgt_port.direction == PortDirection::Input
                    {
                        // Verify they're on different nodes
                        if let (Some(src_node), Some(tgt_node)) = (
                            self.graph.port_owner(source),
                            self.graph.port_owner(target_port),
                        ) {
                            if src_node != tgt_node {
                                // Check type compatibility (exact string match)
                                if src_port.type_tag == tgt_port.type_tag {
                                    actions.push(GraphAction::EdgeCreated {
                                        source,
                                        target: target_port,
                                    });
                                }
                            }
                        }
                    }
                }
                self.mode = InteractionMode::Normal;
            }
            // Wiring mode — clicking anything else cancels.
            (InteractionMode::Wiring { .. }, _) => {
                self.mode = InteractionMode::Normal;
                actions.push(GraphAction::WiringCancelled);
            }

            // Normal mode — port click starts wiring.
            (InteractionMode::Normal, HitTarget::Port(port_id)) => {
                let port = self.graph.port(port_id);
                if let Some(p) = port
                    && p.direction == PortDirection::Output
                {
                    self.mode = InteractionMode::Wiring {
                        source: port_id,
                        cursor_x: cx,
                        cursor_y: cy,
                    };
                    actions.push(GraphAction::WiringStarted { source: port_id });
                }
            }

            // Normal mode — node click selects.
            (InteractionMode::Normal, HitTarget::Node(node_id)) => {
                if shift {
                    if self.selected.contains(&node_id) {
                        self.selected.remove(&node_id);
                    } else {
                        self.selected.insert(node_id);
                    }
                } else {
                    self.selected.clear();
                    self.selected.insert(node_id);
                }
                self.focused = Some(node_id);
                actions.push(GraphAction::SelectionChanged {
                    selected: self.selected.iter().copied().collect(),
                });
            }

            // Normal mode — edge click selects edge.
            (InteractionMode::Normal, HitTarget::Edge(src, tgt)) => {
                self.selected_edge = Some((src, tgt));
            }

            // Normal mode — empty click clears selection.
            (InteractionMode::Normal, HitTarget::Empty) => {
                if !self.selected.is_empty() || self.selected_edge.is_some() {
                    self.selected.clear();
                    self.selected_edge = None;
                    self.focused = None;
                    actions.push(GraphAction::SelectionChanged {
                        selected: Vec::new(),
                    });
                }
            }

            _ => {}
        }

        actions
    }

    /// Handle mouse drag to move selected nodes.
    pub fn handle_mouse_drag(
        &mut self,
        screen_x: u16,
        screen_y: u16,
        dx: i32,
        dy: i32,
    ) -> Vec<GraphAction> {
        let mut actions = Vec::new();

        if self.selected.is_empty() {
            // Start box selection.
            let canvas_pos = self.viewport.screen_to_canvas(
                screen_x.saturating_sub(self.area.x),
                screen_y.saturating_sub(self.area.y),
            );
            match &mut self.mode {
                InteractionMode::BoxSelect { end_x, end_y, .. } => {
                    *end_x = canvas_pos.x;
                    *end_y = canvas_pos.y;
                }
                _ => {
                    self.mode = InteractionMode::BoxSelect {
                        start_x: canvas_pos.x - dx,
                        start_y: canvas_pos.y - dy,
                        end_x: canvas_pos.x,
                        end_y: canvas_pos.y,
                    };
                }
            }
            return actions;
        }

        // Move selected nodes.
        let selected: Vec<NodeId> = self.selected.iter().copied().collect();
        for &id in &selected {
            if let Some(node) = self.graph.node(id) {
                let new_x = node.x + dx;
                let new_y = node.y + dy;
                actions.push(GraphAction::NodeMoved {
                    node: id,
                    x: new_x,
                    y: new_y,
                });
            }
        }

        actions
    }

    /// Finalize box selection.
    pub fn finish_box_select(&mut self) -> Vec<GraphAction> {
        let mut actions = Vec::new();

        if let InteractionMode::BoxSelect {
            start_x,
            start_y,
            end_x,
            end_y,
        } = self.mode
        {
            let min_x = start_x.min(end_x);
            let max_x = start_x.max(end_x);
            let min_y = start_y.min(end_y);
            let max_y = start_y.max(end_y);

            self.selected.clear();
            for bounds in all_node_bounds(&self.graph) {
                let bx2 = bounds.x + bounds.width as i32;
                let by2 = bounds.y + bounds.height as i32;
                // Intersection test.
                if bounds.x < max_x && bx2 > min_x && bounds.y < max_y && by2 > min_y {
                    self.selected.insert(bounds.node_id);
                }
            }

            actions.push(GraphAction::SelectionChanged {
                selected: self.selected.iter().copied().collect(),
            });
            self.mode = InteractionMode::Normal;
        }

        actions
    }

    /// Handle a key press. Returns actions.
    pub fn handle_key(&mut self, key: &str, shift: bool) -> Vec<GraphAction> {
        let mut actions = Vec::new();

        match key {
            "Escape" | "Esc" => {
                if matches!(self.mode, InteractionMode::Wiring { .. }) {
                    self.mode = InteractionMode::Normal;
                    actions.push(GraphAction::WiringCancelled);
                }
            }

            "Delete" | "Backspace" => {
                if let Some((src, tgt)) = self.selected_edge.take() {
                    actions.push(GraphAction::EdgeDeleted {
                        source: src,
                        target: tgt,
                    });
                }
            }

            "Tab" => {
                self.rebuild_tab_order();
                if self.tab_order.is_empty() {
                    return actions;
                }

                match &self.mode {
                    InteractionMode::Wiring { source, .. } => {
                        // Cycle through compatible target ports.
                        let source = *source;
                        let source_port = self.graph.port(source);
                        if let Some(sp) = source_port {
                            let src_tag = sp.type_tag.clone();
                            let src_node = self.graph.port_owner(source);

                            // Collect all compatible input ports.
                            let compatible: Vec<PortId> = self
                                .graph
                                .nodes()
                                .filter(|n| Some(n.id) != src_node)
                                .flat_map(|n| n.input_ports.iter())
                                .filter(|p| p.type_tag == src_tag)
                                .map(|p| p.id)
                                .collect();

                            if !compatible.is_empty() {
                                let current_idx = self
                                    .focused_port
                                    .and_then(|fp| compatible.iter().position(|&p| p == fp))
                                    .map(|i| (i + 1) % compatible.len())
                                    .unwrap_or(0);

                                self.focused_port = Some(compatible[current_idx]);
                            }
                        }
                    }
                    _ => {
                        // Cycle node focus.
                        let current_idx = self
                            .focused
                            .and_then(|f| self.tab_order.iter().position(|&id| id == f));
                        let next_idx = match current_idx {
                            Some(i) => (i + 1) % self.tab_order.len(),
                            None => 0,
                        };
                        let next = self.tab_order[next_idx];
                        self.focused = Some(next);
                        if !shift {
                            self.selected.clear();
                            self.selected.insert(next);
                            actions.push(GraphAction::SelectionChanged {
                                selected: vec![next],
                            });
                        }
                    }
                }
            }

            "Enter" | "Return" => {
                match &self.mode {
                    InteractionMode::Wiring { source, .. } => {
                        // Complete wire to focused port.
                        let source = *source;
                        if let Some(target) = self.focused_port {
                            // Check compatibility without mutating the graph
                            if let (Some(src_port), Some(tgt_port)) =
                                (self.graph.port(source), self.graph.port(target))
                            {
                                // Verify source is Output, target is Input
                                if src_port.direction == PortDirection::Output
                                    && tgt_port.direction == PortDirection::Input
                                {
                                    // Verify they're on different nodes
                                    if let (Some(src_node), Some(tgt_node)) = (
                                        self.graph.port_owner(source),
                                        self.graph.port_owner(target),
                                    ) {
                                        if src_node != tgt_node {
                                            // Check type compatibility (exact string match)
                                            if src_port.type_tag == tgt_port.type_tag {
                                                actions.push(GraphAction::EdgeCreated {
                                                    source,
                                                    target,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        self.mode = InteractionMode::Normal;
                        self.focused_port = None;
                    }
                    InteractionMode::Normal => {
                        // Start wiring from focused node's first output port.
                        if let Some(focused) = self.focused
                            && let Some(node) = self.graph.node(focused)
                            && let Some(port) = node.output_ports.first()
                        {
                            let pid = port.id;
                            self.mode = InteractionMode::Wiring {
                                source: pid,
                                cursor_x: node.x,
                                cursor_y: node.y,
                            };
                            actions.push(GraphAction::WiringStarted { source: pid });
                        }
                    }
                    _ => {}
                }
            }

            // Arrow key nudge.
            "Up" => {
                for &id in &self.selected.clone() {
                    if let Some(n) = self.graph.node(id) {
                        let new_y = n.y - 1;
                        actions.push(GraphAction::NodeMoved {
                            node: id,
                            x: n.x,
                            y: new_y,
                        });
                    }
                }
            }
            "Down" => {
                for &id in &self.selected.clone() {
                    if let Some(n) = self.graph.node(id) {
                        let new_y = n.y + 1;
                        actions.push(GraphAction::NodeMoved {
                            node: id,
                            x: n.x,
                            y: new_y,
                        });
                    }
                }
            }
            "Left" => {
                for &id in &self.selected.clone() {
                    if let Some(n) = self.graph.node(id) {
                        let new_x = n.x - 1;
                        actions.push(GraphAction::NodeMoved {
                            node: id,
                            x: new_x,
                            y: n.y,
                        });
                    }
                }
            }
            "Right" => {
                for &id in &self.selected.clone() {
                    if let Some(n) = self.graph.node(id) {
                        let new_x = n.x + 1;
                        actions.push(GraphAction::NodeMoved {
                            node: id,
                            x: new_x,
                            y: n.y,
                        });
                    }
                }
            }

            _ => {}
        }

        actions
    }
}

// ---------------------------------------------------------------------------
// Widget
// ---------------------------------------------------------------------------

/// Configuration for the node graph widget.
pub struct NodeGraphWidget {
    /// Custom type-tag → color mapping. Falls back to default palette.
    pub type_color_fn: Option<TypeColorFn>,
    /// Base style for node borders.
    pub node_style: Style,
    /// Style for selected node borders.
    pub selected_style: Style,
    /// Style for edge lines.
    pub edge_style: Style,
}

impl Default for NodeGraphWidget {
    fn default() -> Self {
        Self {
            type_color_fn: None,
            node_style: Style::default().fg(Color::White),
            selected_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            edge_style: Style::default().fg(Color::DarkGray),
        }
    }
}

impl NodeGraphWidget {
    fn type_color(&self, type_tag: &str) -> Color {
        match &self.type_color_fn {
            Some(f) => f(type_tag),
            None => default_type_color(type_tag),
        }
    }
}

impl StatefulWidget for NodeGraphWidget {
    type State = NodeGraphState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.viewport.resize(area.width, area.height);
        state.rebuild_tab_order();

        let bounds_list = all_node_bounds(&state.graph);

        // -- render edges first (behind nodes) ------------------------------
        for edge in state.graph.edges() {
            let src_pos = port_canvas_position(&state.graph, edge.source);
            let tgt_pos = port_canvas_position(&state.graph, edge.target);

            if let (Some((sx, sy)), Some((tx, ty))) = (src_pos, tgt_pos) {
                let source_port = state.graph.port(edge.source);
                let color = source_port
                    .map(|p| self.type_color(&p.type_tag))
                    .unwrap_or(Color::DarkGray);

                let is_selected = state.selected_edge == Some((edge.source, edge.target));
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };

                render_manhattan_edge(
                    buf,
                    &state.viewport,
                    area,
                    EdgeCoords { sx, sy, tx, ty },
                    style,
                );
            }
        }

        // -- render wiring preview ------------------------------------------
        if let InteractionMode::Wiring {
            source,
            cursor_x,
            cursor_y,
        } = &state.mode
            && let Some((sx, sy)) = port_canvas_position(&state.graph, *source)
        {
            let style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            render_manhattan_edge(
                buf,
                &state.viewport,
                area,
                EdgeCoords {
                    sx,
                    sy,
                    tx: *cursor_x,
                    ty: *cursor_y,
                },
                style,
            );
        }

        // -- render box-selection overlay -----------------------------------
        if let InteractionMode::BoxSelect {
            start_x,
            start_y,
            end_x,
            end_y,
        } = state.mode
        {
            render_box_select(buf, &state.viewport, area, start_x, start_y, end_x, end_y);
        }

        // -- render nodes ---------------------------------------------------
        for bounds in &bounds_list {
            // Viewport culling.
            let screen_tl = state
                .viewport
                .canvas_to_screen(Position::new(bounds.x, bounds.y));
            let screen_br = state.viewport.canvas_to_screen(Position::new(
                bounds.x + bounds.width as i32 - 1,
                bounds.y + bounds.height as i32 - 1,
            ));

            // Skip if entirely off-screen.
            if screen_tl.is_none() && screen_br.is_none() {
                // Check if the node spans the viewport (partially visible).
                let vis = state.viewport.visible_canvas_size();
                let vx = state.viewport.offset_x;
                let vy = state.viewport.offset_y;
                let bx2 = bounds.x + bounds.width as i32;
                let by2 = bounds.y + bounds.height as i32;
                if bx2 <= vx
                    || bounds.x >= vx + vis.0 as i32
                    || by2 <= vy
                    || bounds.y >= vy + vis.1 as i32
                {
                    continue;
                }
            }

            let is_selected = state.selected.contains(&bounds.node_id);
            let _is_focused = state.focused == Some(bounds.node_id);
            let border_style = if is_selected {
                self.selected_style
            } else {
                self.node_style
            };

            let context = NodeRenderContext {
                border_style,
                widget: &self,
            };
            render_node(buf, &state.viewport, area, &state.graph, bounds, &context);
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Write a char to a screen position if it's within the area.
fn put_char(buf: &mut Buffer, area: Rect, sx: u16, sy: u16, ch: char, style: Style) {
    let ax = area.x + sx;
    let ay = area.y + sy;
    if ax < area.x + area.width && ay < area.y + area.height {
        buf[(ax, ay)].set_char(ch).set_style(style);
    }
}

/// Rendering context for node display.
struct NodeRenderContext<'a> {
    border_style: Style,
    widget: &'a NodeGraphWidget,
}

/// Render a single node.
fn render_node(
    buf: &mut Buffer,
    viewport: &Viewport,
    area: Rect,
    graph: &Graph,
    bounds: &NodeBounds,
    context: &NodeRenderContext,
) {
    let node = match graph.node(bounds.node_id) {
        Some(n) => n,
        None => return,
    };

    let w = bounds.width;
    let h = bounds.height;

    // Render each cell of the node box.
    for dy in 0..h {
        for dx in 0..w {
            let canvas_pos = Position::new(bounds.x + dx as i32, bounds.y + dy as i32);
            let screen = viewport.canvas_to_screen(canvas_pos);
            let Some((sx, sy)) = screen else {
                continue;
            };

            let ch;
            let mut style = context.border_style;

            if dy == 0 {
                // Top border.
                ch = if dx == 0 {
                    '┌'
                } else if dx == w - 1 {
                    '┐'
                } else {
                    '─'
                };
            } else if dy == h - 1 {
                // Bottom border.
                ch = if dx == 0 {
                    '└'
                } else if dx == w - 1 {
                    '┘'
                } else {
                    '─'
                };
            } else if dx == 0 || dx == w - 1 {
                // Side borders.
                ch = '│';
            } else if dy == 1 {
                // Label row.
                let label_start = 2u16;
                let label_idx = dx.saturating_sub(label_start);
                let label_bytes: Vec<char> = node.label.chars().collect();
                if dx >= label_start && (label_idx as usize) < label_bytes.len() {
                    ch = label_bytes[label_idx as usize];
                    style = context.border_style.add_modifier(Modifier::BOLD);
                } else {
                    ch = ' ';
                }
            } else if dy == 2 && dx > 0 && dx < w - 1 {
                // Separator line.
                ch = '─';
            } else {
                // Port rows.
                let port_row = (dy as usize).saturating_sub(3);
                ch = render_port_cell(node, port_row, dx, w, context.widget, &mut style);
            }

            put_char(buf, area, sx, sy, ch, style);
        }
    }
}

/// Render a single cell in the port area of a node.
fn render_port_cell(
    node: &crate::model::Node,
    port_row: usize,
    dx: u16,
    w: u16,
    widget: &NodeGraphWidget,
    style: &mut Style,
) -> char {
    let in_port = node.input_ports.get(port_row);
    let out_port = node.output_ports.get(port_row);

    // Left port marker.
    if dx == 1 {
        if let Some(p) = in_port {
            *style = Style::default().fg(widget.type_color(&p.type_tag));
            return '●';
        }
        return ' ';
    }

    // Input label.
    if let Some(p) = in_port {
        let label_start = 2u16;
        let label_end = label_start + p.label.len() as u16;
        if dx >= label_start && dx < label_end {
            let chars: Vec<char> = p.label.chars().collect();
            return chars[(dx - label_start) as usize];
        }
    }

    // Right port marker.
    if dx == w - 2 {
        if let Some(p) = out_port {
            *style = Style::default().fg(widget.type_color(&p.type_tag));
            return '●';
        }
        return ' ';
    }

    // Output label (right-aligned before marker).
    if let Some(p) = out_port {
        let label_len = p.label.len() as u16;
        let label_start = (w - 3).saturating_sub(label_len);
        let label_end = w - 3;
        if dx >= label_start && dx < label_end {
            let chars: Vec<char> = p.label.chars().collect();
            let idx = dx - label_start;
            if (idx as usize) < chars.len() {
                return chars[idx as usize];
            }
        }
    }

    ' '
}

/// Edge coordinates for Manhattan routing.
struct EdgeCoords {
    sx: i32,
    sy: i32,
    tx: i32,
    ty: i32,
}

/// Render a Manhattan-routed edge between two canvas points.
fn render_manhattan_edge(
    buf: &mut Buffer,
    viewport: &Viewport,
    area: Rect,
    coords: EdgeCoords,
    style: Style,
) {
    let mid_x = (coords.sx + coords.tx) / 2;

    // Horizontal from source to mid.
    let (hx_start, hx_end) = if coords.sx <= mid_x {
        (coords.sx + 1, mid_x)
    } else {
        (mid_x, coords.sx - 1)
    };
    for x in hx_start..=hx_end {
        if let Some((scr_x, scr_y)) = viewport.canvas_to_screen(Position::new(x, coords.sy)) {
            put_char(buf, area, scr_x, scr_y, '─', style);
        }
    }

    // Vertical from sy to ty at mid_x.
    let (vy_start, vy_end) = if coords.sy <= coords.ty {
        (coords.sy, coords.ty)
    } else {
        (coords.ty, coords.sy)
    };
    for y in vy_start..=vy_end {
        if let Some((scr_x, scr_y)) = viewport.canvas_to_screen(Position::new(mid_x, y)) {
            put_char(buf, area, scr_x, scr_y, '│', style);
        }
    }

    // Horizontal from mid to target.
    let (hx_start, hx_end) = if mid_x <= coords.tx {
        (mid_x, coords.tx - 1)
    } else {
        (coords.tx + 1, mid_x)
    };
    for x in hx_start..=hx_end {
        if let Some((scr_x, scr_y)) = viewport.canvas_to_screen(Position::new(x, coords.ty)) {
            put_char(buf, area, scr_x, scr_y, '─', style);
        }
    }

    // Corner characters.
    if coords.sy != coords.ty {
        let corner1 = if coords.sy < coords.ty { '┐' } else { '┘' };
        let corner2 = if coords.sy < coords.ty { '└' } else { '┌' };

        if let Some((scr_x, scr_y)) = viewport.canvas_to_screen(Position::new(mid_x, coords.sy)) {
            put_char(buf, area, scr_x, scr_y, corner1, style);
        }
        if let Some((scr_x, scr_y)) = viewport.canvas_to_screen(Position::new(mid_x, coords.ty)) {
            put_char(buf, area, scr_x, scr_y, corner2, style);
        }
    }
}

/// Render a box-selection rectangle.
fn render_box_select(
    buf: &mut Buffer,
    viewport: &Viewport,
    area: Rect,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) {
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);

    let style = Style::default().fg(Color::Blue).add_modifier(Modifier::DIM);

    // Top and bottom edges.
    for x in min_x..=max_x {
        if let Some((sx, sy)) = viewport.canvas_to_screen(Position::new(x, min_y)) {
            put_char(buf, area, sx, sy, '─', style);
        }
        if let Some((sx, sy)) = viewport.canvas_to_screen(Position::new(x, max_y)) {
            put_char(buf, area, sx, sy, '─', style);
        }
    }
    // Left and right edges.
    for y in min_y..=max_y {
        if let Some((sx, sy)) = viewport.canvas_to_screen(Position::new(min_x, y)) {
            put_char(buf, area, sx, sy, '│', style);
        }
        if let Some((sx, sy)) = viewport.canvas_to_screen(Position::new(max_x, y)) {
            put_char(buf, area, sx, sy, '│', style);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PortSpec;

    fn sp(label: &str) -> PortSpec {
        PortSpec::new(label, "string")
    }

    fn make_state() -> NodeGraphState {
        let mut g = Graph::new();
        let a = g.add_node("Transform", &[sp("data")], &[sp("result"), sp("error")]);
        let b = g.add_node("Output", &[sp("input")], &[]);

        // Position them for testing.
        g.node_mut(a).unwrap().x = 0;
        g.node_mut(a).unwrap().y = 0;
        g.node_mut(b).unwrap().x = 30;
        g.node_mut(b).unwrap().y = 0;

        NodeGraphState::new(g, 80, 24)
    }

    #[test]
    fn node_bounds_calculation() {
        let state = make_state();
        let ids = state.graph.node_ids();
        for &id in &ids {
            let b = node_bounds(&state.graph, id);
            assert!(b.is_some());
            let b = b.unwrap();
            assert!(b.width >= 10);
            assert!(b.height >= 4);
        }
    }

    #[test]
    fn node_bounds_width_scales_with_label() {
        let mut g = Graph::new();
        let short = g.add_node("A", &[], &[]);
        let long = g.add_node("Very Long Node Name Here", &[], &[]);

        let bs = node_bounds(&g, short).unwrap();
        let bl = node_bounds(&g, long).unwrap();
        assert!(bl.width > bs.width);
    }

    #[test]
    fn viewport_culling_offscreen() {
        let mut state = make_state();
        // Move viewport far away.
        state.viewport.offset_x = 1000;
        state.viewport.offset_y = 1000;

        // Nodes at (0,0) and (30,0) should be off-screen.
        let ids = state.graph.node_ids();
        for &id in &ids {
            let b = node_bounds(&state.graph, id).unwrap();
            let tl = state.viewport.canvas_to_screen(Position::new(b.x, b.y));
            assert!(tl.is_none());
        }
    }

    #[test]
    fn hit_test_node() {
        let state = make_state();
        let ids = state.graph.node_ids();
        let first_id = ids[0];
        let b = node_bounds(&state.graph, first_id).unwrap();

        // Hit inside the node body.
        let hit = hit_test(&state, b.x + 2, b.y + 1);
        assert!(matches!(hit, HitTarget::Node(id) if id == first_id));
    }

    #[test]
    fn hit_test_empty() {
        let state = make_state();
        let hit = hit_test(&state, 500, 500);
        assert_eq!(hit, HitTarget::Empty);
    }

    #[test]
    fn hit_test_port() {
        let state = make_state();
        let ids = state.graph.node_ids();
        let first_node = state.graph.node(ids[0]).unwrap();
        let input_port = &first_node.input_ports[0];

        let b = node_bounds(&state.graph, ids[0]).unwrap();
        // Input port is on the left edge, row = y + 2.
        let hit = hit_test(&state, b.x, b.y + 2);
        assert!(matches!(hit, HitTarget::Port(pid) if pid == input_port.id));
    }

    #[test]
    fn click_selects_node() {
        let mut state = make_state();
        state.area = Rect::new(0, 0, 80, 24);
        let ids = state.graph.node_ids();
        let first_id = ids[0];
        let b = node_bounds(&state.graph, first_id).unwrap();

        // Simulate click at node center (screen coords = canvas coords when offset=0).
        let actions = state.handle_mouse_click((b.x + 2) as u16, (b.y + 1) as u16, false);

        assert!(!actions.is_empty());
        assert!(state.selected.contains(&first_id));
    }

    #[test]
    fn click_empty_clears_selection() {
        let mut state = make_state();
        state.area = Rect::new(0, 0, 80, 24);
        let ids = state.graph.node_ids();
        state.selected.insert(ids[0]);

        let actions = state.handle_mouse_click(70, 20, false);
        assert!(state.selected.is_empty());
        assert!(actions.iter().any(
            |a| matches!(a, GraphAction::SelectionChanged { selected } if selected.is_empty())
        ));
    }

    #[test]
    fn arrow_key_nudge() {
        let mut state = make_state();
        let ids = state.graph.node_ids();
        let id = ids[0];
        state.selected.insert(id);

        let orig_x = state.graph.node(id).unwrap().x;
        let actions = state.handle_key("Right", false);
        state.apply_actions(&actions);

        assert_eq!(state.graph.node(id).unwrap().x, orig_x + 1);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, GraphAction::NodeMoved { .. }))
        );
    }

    #[test]
    fn escape_cancels_wiring() {
        let mut state = make_state();
        let ids = state.graph.node_ids();
        let node = state.graph.node(ids[0]).unwrap();
        let out_port = node.output_ports[0].id;

        state.mode = InteractionMode::Wiring {
            source: out_port,
            cursor_x: 0,
            cursor_y: 0,
        };

        let actions = state.handle_key("Escape", false);
        assert!(matches!(state.mode, InteractionMode::Normal));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, GraphAction::WiringCancelled))
        );
    }

    #[test]
    fn tab_cycles_focus() {
        let mut state = make_state();
        state.area = Rect::new(0, 0, 80, 24);

        let _actions1 = state.handle_key("Tab", false);
        let first_focus = state.focused;
        assert!(first_focus.is_some());

        let _actions2 = state.handle_key("Tab", false);
        let second_focus = state.focused;
        assert!(second_focus.is_some());
        assert_ne!(first_focus, second_focus);
    }

    #[test]
    fn keyboard_wiring_flow() {
        let mut state = make_state();
        state.area = Rect::new(0, 0, 80, 24);

        // Focus the node that has output ports ("Transform").
        let src_node = state
            .graph
            .nodes()
            .find(|n| !n.output_ports.is_empty())
            .unwrap()
            .id;
        state.focused = Some(src_node);

        // Enter starts wiring from first output port.
        let actions = state.handle_key("Enter", false);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, GraphAction::WiringStarted { .. }))
        );
        assert!(matches!(state.mode, InteractionMode::Wiring { .. }));

        // Tab cycles to a compatible target port.
        state.handle_key("Tab", false);
        assert!(state.focused_port.is_some());

        // Enter completes the wire.
        let actions = state.handle_key("Enter", false);
        state.apply_actions(&actions);
        assert!(matches!(state.mode, InteractionMode::Normal));
        // Edge should have been created (same type "string").
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, GraphAction::EdgeCreated { .. }))
        );
    }

    #[test]
    fn delete_removes_selected_edge() {
        let mut state = make_state();

        // Find the node with output ports ("Transform") and the one with input ports ("Output").
        let src_node = state
            .graph
            .nodes()
            .find(|n| !n.output_ports.is_empty())
            .unwrap()
            .id;
        let tgt_node = state
            .graph
            .nodes()
            .find(|n| n.id != src_node && !n.input_ports.is_empty())
            .unwrap()
            .id;

        // Create an edge.
        let src = state.graph.node(src_node).unwrap().output_ports[0].id;
        let tgt = state.graph.node(tgt_node).unwrap().input_ports[0].id;
        state.graph.add_edge(src, tgt).unwrap();
        assert_eq!(state.graph.edge_count(), 1);

        // Select and delete it.
        state.selected_edge = Some((src, tgt));
        let actions = state.handle_key("Delete", false);
        state.apply_actions(&actions);
        assert_eq!(state.graph.edge_count(), 0);
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, GraphAction::EdgeDeleted { .. }))
        );
    }
}
