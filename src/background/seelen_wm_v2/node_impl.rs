use evalexpr::{context_map, eval_with_context, HashMapContext};
use itertools::Itertools;
use seelen_core::state::WmNode;

use crate::{error_handler::Result, windows_api::window::Window};

#[derive(Debug)]
pub struct WmNodeImpl(WmNode);

impl WmNodeImpl {
    pub fn new(node: WmNode) -> Self {
        Self(node)
    }

    pub fn inner(&self) -> &WmNode {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut WmNode {
        &mut self.0
    }

    fn is_node_enabled(condition: Option<&String>, context: &HashMapContext) -> bool {
        match condition {
            None => true,
            Some(condition) => {
                let result = eval_with_context(condition, context).and_then(|v| v.as_boolean());
                result.is_ok_and(|is_enabled| is_enabled)
            }
        }
    }

    /// will fail if the node is full
    fn _try_add_window(node: &mut WmNode, window: &Window, context: &HashMapContext) -> Result<()> {
        let addr = window.address();

        if !Self::is_node_enabled(node.condition(), context) {
            return Err("DISABLED".into());
        }

        match node {
            WmNode::Leaf(leaf) => {
                if leaf.handle.is_some() {
                    return Err("FULL".into());
                }

                leaf.handle = Some(addr);
            }
            WmNode::Stack(_stack) => {
                // a node of type stack only can add windows when the user uses the stack shortcut
                return Err("FULL".into());
            }
            WmNode::Fallback(fallback) => {
                fallback.handles.push(addr);
                fallback.active = Some(addr);
            }
            WmNode::Vertical(vertical) => {
                for child in vertical
                    .children
                    .iter_mut()
                    .sorted_by(|a, b| a.priority().cmp(&b.priority()))
                {
                    if Self::_try_add_window(child, window, context).is_ok() {
                        return Ok(());
                    }
                }
                return Err("FULL".into());
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal
                    .children
                    .iter_mut()
                    .sorted_by(|a, b| a.priority().cmp(&b.priority()))
                {
                    if Self::_try_add_window(child, window, context).is_ok() {
                        return Ok(());
                    }
                }
                return Err("FULL".into());
            }
        }
        Ok(())
    }

    /// will drain the node and return a list of window handles
    fn _drain(root: &mut WmNode) -> Vec<isize> {
        let mut handles = Vec::new();
        match root {
            WmNode::Leaf(leaf) => {
                if let Some(handle) = leaf.handle.take() {
                    handles.push(handle);
                }
            }
            WmNode::Stack(stack) => {
                handles.append(&mut stack.handles);
            }
            WmNode::Fallback(fallback) => {
                handles.append(&mut fallback.handles);
            }
            WmNode::Vertical(vertical) => {
                for child in vertical
                    .children
                    .iter_mut()
                    .sorted_by(|a, b| a.priority().cmp(&b.priority()))
                {
                    handles.append(&mut Self::_drain(child));
                }
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal
                    .children
                    .iter_mut()
                    .sorted_by(|a, b| a.priority().cmp(&b.priority()))
                {
                    handles.append(&mut Self::_drain(child));
                }
            }
        }
        handles
    }

    fn _contains(node: &WmNode, window: &Window) -> bool {
        match node {
            WmNode::Leaf(leaf) => leaf.handle == Some(window.address()),
            WmNode::Stack(stack) => stack.handles.contains(&window.address()),
            WmNode::Fallback(fallback) => fallback.handles.contains(&window.address()),
            WmNode::Vertical(vertical) => {
                for child in vertical.children.iter() {
                    if Self::_contains(child, window) {
                        return true;
                    }
                }
                false
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter() {
                    if Self::_contains(child, window) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn _len(node: &WmNode) -> usize {
        match node {
            WmNode::Leaf(leaf) => leaf.handle.is_some() as usize,
            WmNode::Stack(stack) => stack.handles.len(),
            WmNode::Fallback(fallback) => fallback.handles.len(),
            WmNode::Vertical(vertical) => vertical.children.iter().map(Self::_len).sum(),
            WmNode::Horizontal(horizontal) => horizontal.children.iter().map(Self::_len).sum(),
        }
    }

    fn create_context(len: usize, is_reindexing: bool) -> HashMapContext {
        context_map! {
            "managed" => len as i64,
            "is_reindexing" => is_reindexing
        }
        .expect("Failed to create context")
    }

    /// If adding the new window is successful, a reindexing will be done.
    ///
    /// **Note:** Reindexing can fail on add some windows so it will return failed handles as residual
    pub fn try_add_window(&mut self, window: &Window) -> Vec<isize> {
        let len = Self::_len(self.inner());
        let context = Self::create_context(len, false);
        if Self::_try_add_window(self.inner_mut(), window, &context).is_err() {
            return vec![window.address()];
        }

        // reindexing to handle logical condition like `managed < 4`
        let context = Self::create_context(len + 1, true);
        let handles = Self::_drain(self.inner_mut());
        let mut residual = Vec::new();
        for handle in handles {
            if Self::_try_add_window(self.inner_mut(), &Window::from(handle), &context).is_err() {
                residual.push(handle);
            }
        }
        residual
    }

    /// Will make a reindexing ignoring the removed window.
    ///
    /// **Note:** Reindexing can fail on add some windows so it will return failed handles as residual
    pub fn remove_window(&mut self, window: &Window) -> Vec<isize> {
        let handles = Self::_drain(self.inner_mut());
        let context = Self::create_context(
            if handles.contains(&window.address()) {
                handles.len() - 1
            } else {
                handles.len()
            },
            true,
        );

        let mut residual = Vec::new();
        for handle in handles {
            if handle != window.address()
                && Self::_try_add_window(self.inner_mut(), &Window::from(handle), &context).is_err()
            {
                residual.push(handle);
            }
        }
        residual
    }

    pub fn contains(&self, window: &Window) -> bool {
        Self::_contains(self.inner(), window)
    }
}
