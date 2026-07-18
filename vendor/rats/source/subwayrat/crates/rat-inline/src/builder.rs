//! Builder API for constructing inline view trees.

use crate::widget::InlineWidget;
use crate::widgets::InlineText;
use ratcore::inline::{NodeKey, ViewNode, ViewTree};
use std::any::TypeId;

/// Builder for constructing an inline view tree.
///
/// Provides a fluent API for composing widgets into a `ViewTree`
/// that can be passed to `InlineRenderer::rebuild`.
pub struct InlineView {
    nodes: Vec<BuilderNode>,
}

/// A node in the builder — holds the widget and metadata before
/// conversion to a ratcore `ViewNode`.
struct BuilderNode {
    key: Option<NodeKey>,
    type_tag: TypeId,
    widget: Box<dyn InlineWidget>,
}

impl InlineView {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Push an unkeyed widget node.
    pub fn push<W: InlineWidget + 'static>(mut self, widget: W) -> Self {
        self.nodes.push(BuilderNode {
            key: None,
            type_tag: TypeId::of::<W>(),
            widget: Box::new(widget),
        });
        self
    }

    /// Push a keyed widget node. The key provides stable identity
    /// across rebuilds for reconciliation.
    pub fn keyed<W: InlineWidget + 'static>(mut self, key: impl Into<String>, widget: W) -> Self {
        self.nodes.push(BuilderNode {
            key: Some(NodeKey(key.into())),
            type_tag: TypeId::of::<W>(),
            widget: Box::new(widget),
        });
        self
    }

    /// Push a text node (shorthand for `push(InlineText::new(s))`).
    pub fn text(self, content: impl Into<String>) -> Self {
        self.push(InlineText::new(content))
    }

    /// Conditionally add nodes. The closure is only called when
    /// `condition` is true.
    pub fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition { f(self) } else { self }
    }

    /// Add nodes from an iterator.
    pub fn each<I, F>(self, iter: I, mut f: F) -> Self
    where
        I: IntoIterator,
        F: FnMut(Self, I::Item) -> Self,
    {
        let mut this = self;
        for item in iter {
            this = f(this, item);
        }
        this
    }

    /// Build the view tree. Returns the `ViewTree` (for ratcore
    /// reconciliation) and the boxed widgets (for rendering).
    pub fn build(self) -> (ViewTree, Vec<Box<dyn InlineWidget>>) {
        let mut tree = ViewTree::new();
        let mut widgets = Vec::with_capacity(self.nodes.len());
        for node in self.nodes {
            tree.push(ViewNode {
                key: node.key,
                type_tag: node.type_tag,
                state: None,
            });
            widgets.push(node.widget);
        }
        (tree, widgets)
    }
}

impl Default for InlineView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::InlineText;

    #[test]
    fn build_empty() {
        let (tree, widgets) = InlineView::new().build();
        assert!(tree.is_empty());
        assert!(widgets.is_empty());
    }

    #[test]
    fn build_single_text() {
        let (tree, widgets) = InlineView::new().text("hello").build();
        assert_eq!(tree.len(), 1);
        assert_eq!(widgets.len(), 1);
        assert!(tree.nodes[0].key.is_none());
    }

    #[test]
    fn build_keyed() {
        let (tree, _) = InlineView::new()
            .keyed("msg-0", InlineText::new("hi"))
            .build();
        assert_eq!(tree.nodes[0].key.as_ref().unwrap().0, "msg-0");
    }

    #[test]
    fn build_when_true() {
        let (tree, _) = InlineView::new()
            .text("before")
            .when(true, |v| v.text("conditional"))
            .text("after")
            .build();
        assert_eq!(tree.len(), 3);
    }

    #[test]
    fn build_when_false() {
        let (tree, _) = InlineView::new()
            .text("before")
            .when(false, |v| v.text("conditional"))
            .text("after")
            .build();
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn build_each() {
        let items = vec!["a", "b", "c"];
        let (tree, _) = InlineView::new()
            .each(items.iter().enumerate(), |v, (i, item)| {
                v.keyed(format!("item-{i}"), InlineText::new(*item))
            })
            .build();
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.nodes[0].key.as_ref().unwrap().0, "item-0");
        assert_eq!(tree.nodes[2].key.as_ref().unwrap().0, "item-2");
    }
}
