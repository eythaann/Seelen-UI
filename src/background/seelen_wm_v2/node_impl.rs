use evalexpr::{context_map, eval_with_context, HashMapContext};
use itertools::Itertools;
use seelen_core::state::WmNode;

use crate::{error_handler::Result, modules::input::domain::Point, windows_api::window::Window};

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

    fn _trace<'a>(root: &'a WmNode, window: &Window) -> Vec<&'a WmNode> {
        let mut nodes = Vec::new();
        match root {
            WmNode::Leaf(leaf) => {
                if leaf.handle == Some(window.address()) {
                    nodes.push(root);
                }
            }
            WmNode::Stack(stack) => {
                if stack.handles.contains(&window.address()) {
                    nodes.push(root);
                }
            }
            WmNode::Fallback(fallback) => {
                if fallback.handles.contains(&window.address()) {
                    nodes.push(root);
                }
            }
            WmNode::Vertical(vertical) => {
                for child in vertical.children.iter() {
                    let mut sub_trace = Self::_trace(child, window);
                    if !sub_trace.is_empty() {
                        nodes.push(root);
                        nodes.append(&mut sub_trace);
                        break;
                    }
                }
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter() {
                    let mut sub_trace = Self::_trace(child, window);
                    if !sub_trace.is_empty() {
                        nodes.push(root);
                        nodes.append(&mut sub_trace);
                        break;
                    }
                }
            }
        }
        nodes
    }

    fn _get_node_at_point<'a>(
        root: &'a mut WmNode,
        point: &Point,
    ) -> Result<Option<&'a mut WmNode>> {
        match root {
            WmNode::Leaf(leaf) => {
                if let Some(handle) = leaf.handle {
                    let window = Window::from(handle);
                    if point.is_inside_rect(&window.inner_rect()?) {
                        return Ok(Some(root));
                    }
                }
            }
            WmNode::Stack(stack) => {
                if let Some(handle) = stack.active {
                    let window = Window::from(handle);
                    if point.is_inside_rect(&window.inner_rect()?) {
                        return Ok(Some(root));
                    }
                }
            }
            WmNode::Fallback(fallback) => {
                if let Some(handle) = fallback.active {
                    let window = Window::from(handle);
                    if point.is_inside_rect(&window.inner_rect()?) {
                        return Ok(Some(root));
                    }
                }
            }
            WmNode::Vertical(vertical) => {
                for child in vertical.children.iter_mut() {
                    let node = Self::_get_node_at_point(child, point)?;
                    if node.is_some() {
                        return Ok(node);
                    }
                }
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter_mut() {
                    let node = Self::_get_node_at_point(child, point)?;
                    if node.is_some() {
                        return Ok(node);
                    }
                }
            }
        };
        Ok(None)
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
        let len = self.inner().len();
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
        !Self::_trace(self.inner(), window).is_empty()
    }

    pub fn trace(&self, window: &Window) -> Vec<&WmNode> {
        Self::_trace(self.inner(), window)
    }

    pub fn get_node_at_point(&mut self, point: &Point) -> Result<Option<&mut WmNode>> {
        Self::_get_node_at_point(self.inner_mut(), point)
    }
}
