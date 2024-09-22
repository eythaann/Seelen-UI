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
                for child in vertical.children.iter_mut() {
                    handles.append(&mut Self::_drain(child));
                }
            }
            WmNode::Horizontal(horizontal) => {
                for child in horizontal.children.iter_mut() {
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

    /// will fail if the node is full
    pub fn try_add_window(&mut self, window: &Window) -> Result<()> {
        Self::_try_add_window(self.inner_mut(), window)
    }

    /// will make a reindexing after removing the window, if the reindexing fails,
    /// will return the window handles that was not reindexed
    pub fn remove_window(&mut self, window: &Window) -> Vec<isize> {
        let handles = Self::_drain(self.inner_mut());
        let mut residual = Vec::new();
        for handle in handles {
            if handle != window.address() && self.try_add_window(&Window::from(handle)).is_err() {
                residual.push(handle);
            }
        }
        residual
    }

    pub fn contains(&self, window: &Window) -> bool {
        Self::_contains(self.inner(), window)
    }
}
