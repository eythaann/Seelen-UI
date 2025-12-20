use evalexpr::{context_map, eval_with_context, HashMapContext};
use itertools::Itertools;
use seelen_core::{
    state::{WmNode, WmNodeKind},
    Point, Rect,
};
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_RESTORE};

use crate::{
    error::Result,
    widgets::window_manager::{cli::NodeSiblingSide, state::WM_LAYOUT_RECTS},
    windows_api::window::Window,
};

pub trait WmNodeExt {
    /// validates the node condition and returns `true` if the node allows adding windows
    fn is_enabled(&self, context: &HashMapContext) -> bool;

    /// will drain the node and return a list of window handles
    fn drain(&mut self) -> Vec<isize>;
    /// trace the way to the window
    fn trace(&self, window: &Window) -> Vec<&Self>;

    fn get_node_at_point(&self, point: &Point) -> Result<Option<&Self>>;
    fn get_nearest_node_to_rect(&self, rect: &Rect) -> Result<Option<(&Self, i32)>>;

    /// trace and get the inmediate silbings of the node
    fn get_siblings_at_side(&self, window: &Window, side: &NodeSiblingSide) -> Vec<&Self>;

    /// check if the node contains the window
    fn contains(&self, window: &Window) -> bool;
    fn leaf_containing(&self, window: &Window) -> Option<&Self>;
    fn leaf_containing_mut(&mut self, window: &Window) -> Option<&mut Self>;

    /// will fail if the node is full
    fn try_add_window(&mut self, window: &Window, context: &HashMapContext) -> Result<()>;
    fn add_window(&mut self, window: &Window) -> Vec<isize>;
    fn remove_window(&mut self, window: &Window) -> Vec<isize>;

    /// gets the first leaf node having a window, follows node priority.
    fn face(&self) -> Option<Window>;

    fn process_stacks(&self) -> Result<()>;
}

fn create_context(len: usize, is_reindexing: bool) -> HashMapContext {
    context_map! {
        "managed" => len as i64,
        "is_reindexing" => is_reindexing
    }
    .expect("Failed to create context")
}

impl WmNodeExt for WmNode {
    fn is_enabled(&self, context: &HashMapContext) -> bool {
        match &self.condition {
            None => true,
            Some(condition) => {
                let result = eval_with_context(condition, context).and_then(|v| v.as_boolean());
                result.is_ok_and(|is_enabled| is_enabled)
            }
        }
    }

    fn drain(&mut self) -> Vec<isize> {
        let mut drained = Vec::new();
        drained.append(&mut self.windows);
        self.active = None;
        for child in self.children.iter_mut() {
            drained.append(&mut child.drain());
        }
        drained
    }

    fn contains(&self, window: &Window) -> bool {
        self.leaf_containing(window).is_some()
    }

    fn leaf_containing(&self, window: &Window) -> Option<&Self> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if self.windows.contains(&window.address()) {
                    return Some(self);
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self.children.iter() {
                    if let Some(leaf) = child.leaf_containing(window) {
                        return Some(leaf);
                    }
                }
            }
        }
        None
    }

    fn leaf_containing_mut<'a>(&'a mut self, window: &Window) -> Option<&'a mut Self> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if self.windows.contains(&window.address()) {
                    return Some(self);
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self.children.iter_mut() {
                    if let Some(leaf) = child.leaf_containing_mut(window) {
                        return Some(leaf);
                    }
                }
            }
        }
        None
    }

    fn trace(&self, window: &Window) -> Vec<&Self> {
        let mut nodes = Vec::new();
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if self.windows.contains(&window.address()) {
                    nodes.push(self);
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self.children.iter() {
                    let mut sub_trace = child.trace(window);
                    if !sub_trace.is_empty() {
                        nodes.push(self);
                        nodes.append(&mut sub_trace);
                        break;
                    }
                }
            }
        }
        nodes
    }

    fn get_siblings_at_side(&self, window: &Window, side: &NodeSiblingSide) -> Vec<&Self> {
        let trace = self.trace(window);
        let mut siblings = Vec::new();

        let parent_to_search = match side {
            NodeSiblingSide::Left | NodeSiblingSide::Right => WmNodeKind::Horizontal,
            NodeSiblingSide::Up | NodeSiblingSide::Down => WmNodeKind::Vertical,
        };

        // first we search for containers of needed axis that has at least 2 children with some window on it
        let matched_parents = trace.iter().rev().filter(|n| {
            n.kind == parent_to_search && n.children.iter().filter(|n| !n.is_empty()).count() >= 2
        });

        for parent in matched_parents {
            let (node_of_window_idx, _) = parent
                .children
                .iter()
                .find_position(|n| n.contains(window))
                .expect("The algorithm at the top of this function is wrong / broken");

            parent
                .children
                .iter()
                .enumerate()
                .filter(|(idx, n)| {
                    *idx != node_of_window_idx
                        && match side {
                            NodeSiblingSide::Left | NodeSiblingSide::Up => {
                                idx < &node_of_window_idx
                            }
                            NodeSiblingSide::Right | NodeSiblingSide::Down => {
                                idx > &node_of_window_idx
                            }
                        }
                        && !n.is_empty()
                })
                .for_each(|(_, n)| siblings.push(n));
        }

        siblings
    }

    fn try_add_window(&mut self, window: &Window, context: &HashMapContext) -> Result<()> {
        let addr = window.address();

        if !self.is_enabled(context) {
            return Err("Node is disabled by condition".into());
        }

        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if self.is_full() {
                    return Err("FULL".into());
                }

                if self.windows.contains(&addr) {
                    return Ok(());
                }

                self.windows.push(addr);
                self.active = Some(addr);
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self
                    .children
                    .iter_mut()
                    .sorted_by(|a, b| a.priority.cmp(&b.priority))
                {
                    if child.try_add_window(window, context).is_ok() {
                        return Ok(());
                    }
                }
                return Err("FULL".into());
            }
        }

        if self.kind == WmNodeKind::Stack {
            self.process_stacks()?;
        }
        Ok(())
    }

    /// If adding the new window is successful, a reindexing will be done.
    ///
    /// **Note:** Reindexing can fail on add some windows so it will return failed handles as residual
    fn add_window(&mut self, window: &Window) -> Vec<isize> {
        let len = self.len();
        let context = create_context(len, false);
        if self.try_add_window(window, &context).is_err() {
            return vec![window.address()];
        }
        // reindexing to handle logical condition like `managed < 4`
        let context = create_context(len + 1, true);
        let handles = self.drain();
        let mut residual = Vec::new();
        for handle in handles {
            if self
                .try_add_window(&Window::from(handle), &context)
                .is_err()
            {
                residual.push(handle);
            }
        }
        residual
    }

    /// Will make a reindexing ignoring the removed window.
    ///
    /// **Note:** Reindexing can fail on add some windows so it will return failed handles as residual
    fn remove_window(&mut self, window: &Window) -> Vec<isize> {
        let handles = self.drain();
        let context = create_context(
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
                && self
                    .try_add_window(&Window::from(handle), &context)
                    .is_err()
            {
                residual.push(handle);
            }
        }
        residual
    }

    fn face(&self) -> Option<Window> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if let Some(handle) = self.active {
                    return Some(Window::from(handle));
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self
                    .children
                    .iter()
                    .sorted_by(|a, b| a.priority.cmp(&b.priority))
                {
                    if !child.is_empty() {
                        return child.face();
                    }
                }
            }
        }
        None
    }

    fn get_node_at_point(&self, point: &Point) -> Result<Option<&Self>> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if let Some(handle) = self.active {
                    let window = Window::from(handle);
                    // Use expected rect from WM_LAYOUT_RECTS if available, otherwise use current inner rect
                    let window_rect = WM_LAYOUT_RECTS
                        .get(&handle, |v| v.clone())
                        .unwrap_or_else(|| window.inner_rect().unwrap_or_default());

                    if window_rect.contains(point) {
                        return Ok(Some(self));
                    }
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in &self.children {
                    if let Some(node) = child.get_node_at_point(point)? {
                        return Ok(Some(node));
                    }
                }
            }
        }
        Ok(None)
    }

    /// returns None if the node is empty
    /// uses the closest corners algorithm to find the nearest node
    fn get_nearest_node_to_rect(&self, rect: &Rect) -> Result<Option<(&Self, i32)>> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if let Some(handle) = self.active {
                    let window = Window::from(handle);
                    // Use expected rect from WM_LAYOUT_RECTS if available, otherwise use current inner rect
                    let window_rect = WM_LAYOUT_RECTS
                        .get(&handle, |v| v.clone())
                        .unwrap_or_else(|| window.inner_rect().unwrap_or_default());

                    let window_corners = window_rect.corners();
                    let search_corners = rect.corners();

                    // Find minimum distance between corresponding corners
                    // corners() returns: [top-left, top-right, bottom-right, bottom-left]
                    let mut min_distance = i32::MAX;
                    for i in 0..4 {
                        let distance = search_corners[i].distance_squared(&window_corners[i]);
                        if distance < min_distance {
                            min_distance = distance;
                        }
                    }

                    Ok(Some((self, min_distance)))
                } else {
                    Ok(None)
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                let mut nearest: Option<(&Self, i32)> = None;
                for child in &self.children {
                    if let Some((node, distance)) = child.get_nearest_node_to_rect(rect)? {
                        if nearest.is_none() || distance < nearest.unwrap().1 {
                            nearest = Some((node, distance));
                        }
                    }
                }
                Ok(nearest)
            }
        }
    }

    fn process_stacks(&self) -> Result<()> {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => {
                if let Some(active) = self.active {
                    for addr in &self.windows {
                        let window = Window::from(*addr);
                        if *addr == active {
                            if window.is_minimized() {
                                window.show_window(SW_RESTORE)?;
                            }
                        } else if !window.is_minimized() {
                            window.show_window(SW_FORCEMINIMIZE)?;
                        }
                    }
                }
            }
            WmNodeKind::Horizontal | WmNodeKind::Vertical => {
                for child in self.children.iter() {
                    child.process_stacks()?;
                }
            }
        }
        Ok(())
    }
}
