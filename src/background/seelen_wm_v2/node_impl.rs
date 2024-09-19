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

    /// will fail if the node is full
    fn _try_add_window(node: &mut WmNode, window: &Window) -> Result<()> {
        let addr = window.address();
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
                for child in vertical.children.iter_mut() {
                    if Self::_try_add_window(child, window).is_ok() {
                        return Ok(());
                    }
                }
                return Err("FULL".into());
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter_mut() {
                    if Self::_try_add_window(child, window).is_ok() {
                        return Ok(());
                    }
                }
                return Err("FULL".into());
            }
        }
        Ok(())
    }

    fn _remove_window(node: &mut WmNode, window: &Window) {
        match node {
            WmNode::Leaf(leaf) => {
                if leaf.handle == Some(window.address()) {
                    leaf.handle = None;
                }
            }
            WmNode::Stack(stack) => {
                stack.handles.retain(|&x| x != window.address());
            }
            WmNode::Fallback(fallback) => {
                fallback.handles.retain(|&x| x != window.address());
            }
            WmNode::Vertical(vertical) => {
                for child in vertical.children.iter_mut() {
                    Self::_remove_window(child, window);
                }
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter_mut() {
                    Self::_remove_window(child, window);
                }
            }
        }
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

    /// will fail if the node is full
    pub fn try_add_window(&mut self, window: &Window) -> Result<()> {
        Self::_try_add_window(self.inner_mut(), window)
    }

    pub fn remove_window(&mut self, window: &Window) {
        Self::_remove_window(self.inner_mut(), window);
    }

    pub fn contains(&self, window: &Window) -> bool {
        Self::_contains(self.inner(), window)
    }
}
