use crate::layout::{
    Layout,
    NodeLayout,
};

pub(crate) struct LayoutBuilder {
    stack: Vec<String>,
    layout: Layout,
}

impl LayoutBuilder {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            layout: Layout::new(),
        }
    }

    pub fn push(&mut self, segment: impl Into<String>) {
        self.stack.push(segment.into());
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn record(&mut self, node: NodeLayout) {
        self.layout.insert(self.current_path(), node);
    }

    pub fn finish(self) -> Layout {
        self.layout
    }

    fn current_path(&self) -> String {
        if self.stack.is_empty() {
            return String::new();
        }
        let mut out = String::with_capacity(self.stack.iter().map(|s| s.len() + 1).sum());
        for segment in &self.stack {
            out.push('/');
            for ch in segment.chars() {
                match ch {
                    '~' => out.push_str("~0"),
                    '/' => out.push_str("~1"),
                    _ => out.push(ch),
                }
            }
        }
        out
    }
}
