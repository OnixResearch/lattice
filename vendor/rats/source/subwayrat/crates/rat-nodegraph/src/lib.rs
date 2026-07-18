//! Node-based graph editor widget for ratatui.
//!
//! Provides a directed graph data model with typed ports, a ratatui
//! `StatefulWidget` for rendering nodes and edges on an infinite canvas,
//! interaction handlers for selection/dragging/wiring, and a Sugiyama-style
//! auto-layout engine.
//!
//! # Architecture
//!
//! - [`model`] — Pure data structures (`Graph`, `Node`, `Port`, `Edge`). No
//!   ratatui dependency. Supports serde behind a feature flag.
//! - [`view`] — `NodeGraphWidget` (implements `StatefulWidget`) and
//!   `NodeGraphState`. Renders nodes as bordered boxes with port markers,
//!   edges as Manhattan-routed box-drawing lines. Interaction via
//!   `handle_mouse_click`, `handle_mouse_drag`, `handle_key` which return
//!   `GraphAction` events.
//! - [`layout`] — `auto_layout()` positions nodes using a layered algorithm
//!   with configurable direction, spacing, and snap-to-grid.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use rat_nodegraph::model::{Graph, PortSpec};
//! use rat_nodegraph::layout::{auto_layout, LayoutConfig};
//! use rat_nodegraph::view::{NodeGraphState, NodeGraphWidget};
//!
//! let mut graph = Graph::new();
//! let a = graph.add_node("Source", &[], &[PortSpec::new("out", "string")]);
//! let b = graph.add_node("Sink", &[PortSpec::new("in", "string")], &[]);
//!
//! let src = graph.node(a).unwrap().output_ports[0].id;
//! let tgt = graph.node(b).unwrap().input_ports[0].id;
//! graph.add_edge(src, tgt).unwrap();
//!
//! auto_layout(&mut graph, &LayoutConfig::default());
//!
//! let mut state = NodeGraphState::new(graph, 80, 24);
//! // In your ratatui render loop:
//! // frame.render_stateful_widget(NodeGraphWidget::default(), area, &mut state);
//! ```

pub mod layout;
pub mod model;
pub mod view;
