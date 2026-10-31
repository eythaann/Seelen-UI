use std::{
    collections::VecDeque,
    sync::LazyLock,
    time::{Duration, Instant},
};

use seelen_core::{rect::Rect, system_state::WindowPeekStatus};
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        WindowsAndMessaging::*,
    },
};

use crate::{error::Result, utils::lock_free::TracedMutex, windows_api::window::Window};

const POLL_TTL: Duration = Duration::from_secs(2);
const LEAVE_GRACE: Duration = Duration::from_millis(220);
const MAX_STACK_REFERENCES: usize = 16;
const POSITION_FLAGS: SET_WINDOW_POS_FLAGS = SET_WINDOW_POS_FLAGS(
    SWP_NOMOVE.0 | SWP_NOSIZE.0 | SWP_NOACTIVATE.0 | SWP_NOOWNERZORDER.0 | SWP_ASYNCWINDOWPOS.0,
);

/// One temporary presentation at a time. A watchdog also restores it if its webview disappears.
pub struct WindowPeekManager {
    state: TracedMutex<PeekState>,
}

impl WindowPeekManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<WindowPeekManager> = LazyLock::new(|| {
            std::thread::Builder::new()
                .name("WindowPeekWatchdog".into())
                .spawn(|| {
                    loop {
                        std::thread::sleep(Duration::from_millis(32));
                        WindowPeekManager::instance().tick();
                    }
                })
                .expect("window peek watchdog");
            WindowPeekManager {
                state: TracedMutex::new(PeekState::default()),
            }
        });
        &MANAGER
    }

    pub fn begin(
        &self,
        owner: isize,
        hwnd: isize,
        session: String,
        rect: Rect,
    ) -> Result<WindowPeekStatus> {
        if session.is_empty() || session.len() > 128 {
            return Err("Invalid preview session".into());
        }
        if rect.left < 0
            || rect.top < 0
            || rect.right <= rect.left
            || rect.bottom <= rect.top
            || rect.right > 32768
            || rect.bottom > 32768
        {
            return Err("Invalid thumbnail bounds".into());
        }
        if !unsafe { IsWindowVisible(HWND(owner as _)) }.as_bool() {
            return Err("The preview popup is hidden".into());
        }
        let mut state = self.state.lock();
        state.prune_cancellations();
        if state
            .cancelled
            .iter()
            .any(|(window, token, _)| *window == owner && token == &session)
        {
            return Ok(WindowPeekStatus::default());
        }
        if state
            .active
            .as_ref()
            .is_some_and(|active| active.committing)
        {
            return Err("A preview click is already being committed".into());
        }
        state.finish(false);
        let mut view = TemporaryWindow::capture(hwnd)?;
        view.show()?;
        unsafe {
            SetWindowPos(
                HWND(owner as _),
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                POSITION_FLAGS,
            )?;
        }
        state.active = Some(PeekSession {
            owner,
            token: session,
            rect,
            view,
            refreshed: Instant::now(),
            outside_since: None,
            committing: false,
            button_down: unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0,
        });
        Ok(state.active_status())
    }

    pub fn poll(&self, owner: isize, token: &str, rect: Option<Rect>) -> WindowPeekStatus {
        let mut state = self.state.lock();
        if let Some(active) = state
            .active
            .as_mut()
            .filter(|active| active.matches(owner, token))
        {
            active.refreshed = Instant::now();
            if let Some(rect) = rect {
                active.rect = rect;
            }
        }
        state.status(owner, token)
    }

    pub fn cancel(&self, owner: isize, token: &str) {
        let mut state = self.state.lock();
        state.prune_cancellations();
        state
            .cancelled
            .push_back((owner, token.to_owned(), Instant::now()));
        if state
            .active
            .as_ref()
            .is_some_and(|active| active.matches(owner, token) && !active.committing)
        {
            let keep = state
                .active
                .as_ref()
                .is_some_and(|active| active.view.activated());
            state.finish(keep);
        }
    }

    pub fn commit(&self, owner: isize, token: &str) -> Result<bool> {
        let target = {
            let mut state = self.state.lock();
            if state.status(owner, token).committed {
                return Ok(true);
            }
            let active = state
                .active
                .as_mut()
                .filter(|active| active.matches(owner, token))
                .ok_or("The temporary preview has ended")?;
            if active.committing {
                return Err("A preview click is already being committed".into());
            }
            active.committing = true;
            active.refreshed = Instant::now();
            active.view.hwnd
        };
        // No mutex is held while Windows processes the foreground request.
        unsafe {
            let _ = SetForegroundWindow(HWND(target as _));
        }
        let started = Instant::now();
        let mut activated = false;
        while started.elapsed() < Duration::from_millis(350) {
            if is_foreground(target) {
                activated = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut state = self.state.lock();
        if let Some(active) = state
            .active
            .as_mut()
            .filter(|active| active.matches(owner, token))
        {
            active.committing = false;
            if activated {
                state.finish(true);
            }
        }
        Ok(activated)
    }

    pub fn cancel_all(&self) {
        let mut state = self.state.lock();
        let keep = state
            .active
            .as_ref()
            .is_some_and(|active| active.view.activated());
        state.finish(keep);
    }

    fn tick(&self) {
        let mut state = self.state.lock();
        let Some(active) = state.active.as_mut() else {
            return;
        };
        if active.committing {
            return;
        }
        if active.view.activated() {
            state.finish(true);
            return;
        }
        if !active.view.valid()
            || !unsafe { IsWindowVisible(HWND(active.owner as _)) }.as_bool()
            || active.refreshed.elapsed() > POLL_TTL
        {
            state.finish(false);
            return;
        }
        let mut point = POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_err() {
            state.finish(false);
            return;
        }
        let over_real = active.view.contains(point);
        let down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
        if down && !active.button_down && over_real {
            state.finish(true);
            return;
        }
        active.button_down = down;
        let foreground = unsafe { GetForegroundWindow() }.0 as isize;
        if foreground != active.view.foreground
            && foreground != active.owner
            && foreground != active.view.hwnd
        {
            state.finish(false);
            return;
        }
        if active.contains_thumbnail(point) || over_real {
            active.outside_since = None;
        } else if active
            .outside_since
            .get_or_insert_with(Instant::now)
            .elapsed()
            > LEAVE_GRACE
        {
            state.finish(false);
        }
    }
}

#[derive(Default)]
struct PeekState {
    active: Option<PeekSession>,
    completed: Option<(isize, String, isize, bool)>,
    cancelled: VecDeque<(isize, String, Instant)>,
}

impl PeekState {
    fn active_status(&self) -> WindowPeekStatus {
        WindowPeekStatus {
            active: self.active.is_some(),
            committed: false,
            hwnd: self.active.as_ref().map(|active| active.view.hwnd),
        }
    }
    fn status(&self, owner: isize, token: &str) -> WindowPeekStatus {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.matches(owner, token))
        {
            return self.active_status();
        }
        if let Some((window, session, hwnd, committed)) = &self.completed
            && *window == owner
            && session == token
        {
            return WindowPeekStatus {
                active: false,
                committed: *committed,
                hwnd: Some(*hwnd),
            };
        }
        WindowPeekStatus::default()
    }
    fn finish(&mut self, keep: bool) {
        if let Some(mut active) = self.active.take() {
            active.view.kept = keep;
            self.completed = Some((active.owner, active.token.clone(), active.view.hwnd, keep));
        }
    }
    fn prune_cancellations(&mut self) {
        while self
            .cancelled
            .front()
            .is_some_and(|(_, _, time)| time.elapsed() > Duration::from_secs(30))
            || self.cancelled.len() >= 128
        {
            self.cancelled.pop_front();
        }
    }
}

struct PeekSession {
    owner: isize,
    token: String,
    rect: Rect,
    view: TemporaryWindow,
    refreshed: Instant,
    outside_since: Option<Instant>,
    committing: bool,
    button_down: bool,
}

impl PeekSession {
    fn matches(&self, owner: isize, token: &str) -> bool {
        self.owner == owner && self.token == token
    }
    fn contains_thumbnail(&self, point: POINT) -> bool {
        let mut origin = POINT::default();
        if !unsafe { ClientToScreen(HWND(self.owner as _), &mut origin) }.as_bool() {
            return false;
        }
        point.x >= origin.x + self.rect.left
            && point.x < origin.x + self.rect.right
            && point.y >= origin.y + self.rect.top
            && point.y < origin.y + self.rect.bottom
            && unsafe { GetAncestor(WindowFromPoint(point), GA_ROOT) }.0 as isize == self.owner
    }
}

struct TemporaryWindow {
    hwnd: isize,
    pid: u32,
    order: StackingOrder,
    topmost: bool,
    placement: WINDOWPLACEMENT,
    minimized: bool,
    foreground: isize,
    kept: bool,
}

impl TemporaryWindow {
    fn capture(hwnd: isize) -> Result<Self> {
        let handle = HWND(hwnd as _);
        if !unsafe { IsWindow(Some(handle)) }.as_bool() {
            return Err("The preview window has closed".into());
        }
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(handle, Some(&mut pid));
        }
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        unsafe {
            GetWindowPlacement(handle, &mut placement)?;
        }
        let topmost = is_topmost(hwnd);
        let order = StackingOrder::capture(hwnd, topmost, std::process::id());
        Ok(Self {
            hwnd,
            pid,
            order,
            topmost,
            placement,
            minimized: unsafe { IsIconic(handle) }.as_bool(),
            foreground: unsafe { GetForegroundWindow() }.0 as isize,
            kept: false,
        })
    }
    fn show(&mut self) -> Result<()> {
        let handle = HWND(self.hwnd as _);
        if self.minimized {
            unsafe {
                ShowWindowAsync(handle, SW_SHOWNOACTIVATE).ok()?;
            }
        }
        unsafe {
            SetWindowPos(
                handle,
                Some(if self.topmost { HWND_TOPMOST } else { HWND_TOP }),
                0,
                0,
                0,
                0,
                POSITION_FLAGS,
            )?;
        }
        Ok(())
    }
    fn valid(&self) -> bool {
        let handle = HWND(self.hwnd as _);
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(handle, Some(&mut pid));
        }
        unsafe { IsWindow(Some(handle)) }.as_bool() && pid == self.pid
    }
    fn activated(&self) -> bool {
        self.foreground != self.hwnd && is_foreground(self.hwnd)
    }
    fn contains(&self, point: POINT) -> bool {
        let handle = HWND(self.hwnd as _);
        let mut rect = RECT::default();
        if !self.valid()
            || unsafe { IsIconic(handle) }.as_bool()
            || unsafe { GetWindowRect(handle, &mut rect) }.is_err()
        {
            return false;
        }
        point.x >= rect.left
            && point.x < rect.right
            && point.y >= rect.top
            && point.y < rect.bottom
            && unsafe { GetAncestor(WindowFromPoint(point), GA_ROOT) } == handle
    }
}

impl Drop for TemporaryWindow {
    fn drop(&mut self) {
        if self.kept || !self.valid() {
            return;
        }
        let handle = HWND(self.hwnd as _);
        if self.minimized {
            let mut restore = self.placement;
            restore.flags |= WPF_ASYNCWINDOWPLACEMENT;
            restore.showCmd = SW_SHOWMINNOACTIVE.0 as u32;
            unsafe {
                let _ = SetWindowPlacement(handle, &restore);
            }
        }
        self.order.restore(self.hwnd);
    }
}

/// Keep several nearest neighbors on both sides; any individual window may disappear during a peek.
struct StackingOrder {
    above: Vec<WindowReference>,
    below: Vec<WindowReference>,
    topmost: bool,
    ignored_process: u32,
}

impl StackingOrder {
    fn capture(source: isize, topmost: bool, ignored_process: u32) -> Self {
        Self {
            above: stack_neighbors(source, GW_HWNDPREV, topmost, ignored_process),
            below: stack_neighbors(source, GW_HWNDNEXT, topmost, ignored_process),
            topmost,
            ignored_process,
        }
    }

    fn restore(&self, source: isize) {
        for reference in &self.above {
            if reference.usable(self.topmost) && position_after(source, HWND(reference.hwnd as _)) {
                return;
            }
        }
        for reference in &self.below {
            if !reference.usable(self.topmost) {
                continue;
            }
            // Insert just before the surviving lower neighbor, using its current predecessor.
            let previous = unsafe { GetWindow(HWND(reference.hwnd as _), GW_HWNDPREV) }.ok();
            if previous.is_some_and(|window| window.0 as isize == source) {
                return;
            }
            let after = previous
                .filter(|window| is_topmost(window.0 as isize) == self.topmost)
                .unwrap_or(if self.topmost { HWND_TOPMOST } else { HWND_TOP });
            if position_after(source, after) {
                return;
            }
        }
        // If every saved reference vanished, demote behind surviving visible windows in the same band.
        // Do not use HWND_BOTTOM, which can put the source behind the desktop or clear its topmost flag.
        // Walk the source's own desktop/Z-order chain rather than assuming the caller's desktop.
        let mut current = unsafe { GetWindow(HWND(source as _), GW_HWNDNEXT) }.ok();
        let mut fallbacks = Vec::new();
        let desktop = unsafe { GetDesktopWindow() }.0 as isize;
        for _ in 0..512 {
            let Some(window) = current else {
                break;
            };
            let address = window.0 as isize;
            if address != source
                && is_topmost(address) == self.topmost
                && unsafe { IsWindowVisible(window) }.as_bool()
                && let Some(reference) = WindowReference::capture(address)
                && reference.pid != self.ignored_process
                && address != desktop
                && !matches!(
                    Window::from(address).class().as_str(),
                    "Progman" | "WorkerW"
                )
            {
                fallbacks.push(reference);
            }
            current = unsafe { GetWindow(window, GW_HWNDNEXT) }.ok();
        }
        // Some desktop bands reject an insertion reference even when its HWND is valid.
        for reference in fallbacks.into_iter().rev() {
            if reference.usable(self.topmost) && position_after(source, HWND(reference.hwnd as _)) {
                return;
            }
        }
    }
}

#[derive(Clone, Copy)]
struct WindowReference {
    hwnd: isize,
    pid: u32,
}
impl WindowReference {
    fn capture(hwnd: isize) -> Option<Self> {
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(HWND(hwnd as _), Some(&mut pid));
        }
        (pid != 0).then_some(Self { hwnd, pid })
    }
    fn usable(&self, topmost: bool) -> bool {
        unsafe { IsWindow(Some(HWND(self.hwnd as _))) }.as_bool()
            && Self::capture(self.hwnd).is_some_and(|window| window.pid == self.pid)
            && is_topmost(self.hwnd) == topmost
    }
}

fn stack_neighbors(
    source: isize,
    direction: GET_WINDOW_CMD,
    topmost: bool,
    ignored_process: u32,
) -> Vec<WindowReference> {
    let mut neighbors = Vec::with_capacity(MAX_STACK_REFERENCES);
    let mut current = unsafe { GetWindow(HWND(source as _), direction) }.ok();
    for _ in 0..512 {
        let Some(window) = current else {
            break;
        };
        let address = window.0 as isize;
        if address == source {
            break;
        }
        if is_topmost(address) == topmost
            && let Some(reference) = WindowReference::capture(address)
            && reference.pid != ignored_process
        {
            neighbors.push(reference);
            if neighbors.len() == MAX_STACK_REFERENCES {
                break;
            }
        }
        current = unsafe { GetWindow(window, direction) }.ok();
    }
    neighbors
}

fn position_after(source: isize, after: HWND) -> bool {
    unsafe { SetWindowPos(HWND(source as _), Some(after), 0, 0, 0, 0, POSITION_FLAGS) }.is_ok()
}

fn is_topmost(hwnd: isize) -> bool {
    (unsafe { GetWindowLongPtrW(HWND(hwnd as _), GWL_EXSTYLE) } & WS_EX_TOPMOST.0 as isize) != 0
}
fn is_foreground(hwnd: isize) -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    foreground.0 as isize == hwnd
        || unsafe { GetAncestor(foreground, GA_ROOTOWNER) }.0 as isize == hwnd
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;

    // Only these transparent offscreen fixtures are shown. No user window is focused or moved.
    struct TestWindow(HWND);
    impl TestWindow {
        fn new() -> Self {
            unsafe {
                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                    w!("STATIC"),
                    w!("Seelen peek isolated test"),
                    WS_OVERLAPPEDWINDOW,
                    -30000,
                    -30000,
                    320,
                    240,
                    None,
                    None,
                    None,
                    None,
                )
                .expect("offscreen fixture");
                SetLayeredWindowAttributes(
                    hwnd,
                    windows::Win32::Foundation::COLORREF(0),
                    0,
                    LWA_ALPHA,
                )
                .unwrap();
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                Self(hwnd)
            }
        }
        fn address(&self) -> isize {
            self.0.0 as isize
        }
        fn wait_for_minimized(&self, expected: bool) {
            // ShowWindowAsync posts to the target queue; the fixture must run its message loop.
            let deadline = Instant::now() + Duration::from_millis(500);
            loop {
                let mut msg = MSG::default();
                for _ in 0..256 {
                    if !unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                        break;
                    }
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                if unsafe { IsIconic(self.0) }.as_bool() == expected {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "fixture did not process its asynchronous show state"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
    impl Drop for TestWindow {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    #[test]
    fn exhausted_references_demote_within_the_normal_band() {
        let source = TestWindow::new();
        let other = TestWindow::new();
        arrange(&[&source, &other]);
        let mut view = TemporaryWindow::capture(source.address()).unwrap();
        view.order = StackingOrder {
            above: vec![],
            below: vec![],
            topmost: false,
            ignored_process: 0,
        };
        view.show().unwrap();
        source.wait_for_minimized(false);
        drop(view);
        source.wait_for_minimized(false);
        assert!(above_window(other.address(), source.address()));
        assert!(!is_topmost(source.address()));
    }

    #[test]
    fn exhausted_references_preserve_the_topmost_band() {
        let source = TestWindow::new();
        let other = TestWindow::new();
        assert!(position_after(other.address(), HWND_TOPMOST));
        other.wait_for_minimized(false);
        assert!(position_after(source.address(), HWND_TOPMOST));
        source.wait_for_minimized(false);
        let mut view = TemporaryWindow::capture(source.address()).unwrap();
        view.order = StackingOrder {
            above: vec![],
            below: vec![],
            topmost: true,
            ignored_process: 0,
        };
        view.show().unwrap();
        source.wait_for_minimized(false);
        drop(view);
        source.wait_for_minimized(false);
        assert!(above_window(other.address(), source.address()));
        assert!(is_topmost(source.address()));
    }

    #[test]
    fn closed_predecessor_uses_a_surviving_upper_reference() {
        let above = TestWindow::new();
        let previous = TestWindow::new();
        let source = TestWindow::new();
        let below = TestWindow::new();
        arrange(&[&above, &previous, &source, &below]);
        let mut view = TemporaryWindow::capture(source.address()).unwrap();
        view.order = StackingOrder::capture(source.address(), false, 0);
        view.show().unwrap();
        source.wait_for_minimized(false);
        drop(previous);
        drop(view);
        source.wait_for_minimized(false);
        assert!(above_window(above.address(), source.address()));
        assert!(above_window(source.address(), below.address()));
        assert!(!is_topmost(source.address()));
    }

    #[test]
    fn predecessor_changing_topmost_band_does_not_leave_the_source_raised() {
        let above = TestWindow::new();
        let previous = TestWindow::new();
        let source = TestWindow::new();
        let below = TestWindow::new();
        arrange(&[&above, &previous, &source, &below]);
        let mut view = TemporaryWindow::capture(source.address()).unwrap();
        view.order = StackingOrder::capture(source.address(), false, 0);
        view.show().unwrap();
        source.wait_for_minimized(false);
        assert!(position_after(previous.address(), HWND_TOPMOST));
        previous.wait_for_minimized(false);
        drop(view);
        source.wait_for_minimized(false);
        assert!(above_window(above.address(), source.address()));
        assert!(above_window(source.address(), below.address()));
        assert!(is_topmost(previous.address()));
        assert!(!is_topmost(source.address()));
    }

    #[test]
    fn missing_upper_references_restore_before_a_surviving_lower_neighbor() {
        let above = TestWindow::new();
        let previous = TestWindow::new();
        let source = TestWindow::new();
        let below = TestWindow::new();
        arrange(&[&above, &previous, &source, &below]);
        let mut view = TemporaryWindow::capture(source.address()).unwrap();
        view.order = StackingOrder::capture(source.address(), false, 0);
        view.show().unwrap();
        source.wait_for_minimized(false);
        drop(previous);
        drop(above);
        let newcomer = TestWindow::new();
        assert!(position_after(source.address(), HWND_TOP));
        source.wait_for_minimized(false);
        drop(view);
        source.wait_for_minimized(false);
        assert!(above_window(newcomer.address(), source.address()));
        assert!(above_window(source.address(), below.address()));
    }

    fn arrange(windows: &[&TestWindow]) {
        for window in windows.iter().rev() {
            assert!(position_after(window.address(), HWND_TOP));
            window.wait_for_minimized(false);
        }
    }

    fn above_window(upper: isize, lower: isize) -> bool {
        let mut current = unsafe { GetWindow(HWND(lower as _), GW_HWNDPREV) }.ok();
        for _ in 0..512 {
            let Some(window) = current else {
                return false;
            };
            if window.0 as isize == upper {
                return true;
            }
            current = unsafe { GetWindow(window, GW_HWNDPREV) }.ok();
        }
        false
    }

    #[test]
    fn temporary_show_preserves_position_and_foreground() {
        let window = TestWindow::new();
        let mut before = RECT::default();
        unsafe {
            GetWindowRect(window.0, &mut before).unwrap();
        }
        let foreground = unsafe { GetForegroundWindow() };
        {
            let mut view = TemporaryWindow::capture(window.address()).unwrap();
            view.show().unwrap();
            assert_eq!(unsafe { GetForegroundWindow() }, foreground);
        }
        let mut after = RECT::default();
        unsafe {
            GetWindowRect(window.0, &mut after).unwrap();
        }
        assert_eq!(
            (before.left, before.top, before.right, before.bottom),
            (after.left, after.top, after.right, after.bottom)
        );
    }

    #[test]
    fn minimized_window_is_restored_to_minimized_when_hover_ends() {
        let window = TestWindow::new();
        unsafe {
            let _ = ShowWindow(window.0, SW_SHOWMINNOACTIVE);
        }
        assert!(unsafe { IsIconic(window.0) }.as_bool());
        {
            let mut view = TemporaryWindow::capture(window.address()).unwrap();
            view.show().unwrap();
            window.wait_for_minimized(false);
        }
        window.wait_for_minimized(true);
    }

    #[test]
    fn cancelled_begin_and_wrong_owner_cannot_change_a_live_session() {
        let owner = TestWindow::new();
        let target = TestWindow::new();
        let manager = WindowPeekManager {
            state: TracedMutex::new(PeekState::default()),
        };
        let rect = Rect {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
        };
        manager.cancel(owner.address(), "cancelled");
        assert!(
            !manager
                .begin(
                    owner.address(),
                    target.address(),
                    "cancelled".into(),
                    rect.clone()
                )
                .unwrap()
                .active
        );
        assert!(
            manager
                .begin(owner.address(), target.address(), "current".into(), rect)
                .unwrap()
                .active
        );
        manager.cancel(target.address(), "current");
        assert!(manager.poll(owner.address(), "current", None).active);
        manager.cancel(owner.address(), "current");
        assert!(!manager.poll(owner.address(), "current", None).active);
    }

    #[test]
    fn kept_minimized_window_stays_open_and_keeps_the_same_handle() {
        let window = TestWindow::new();
        unsafe {
            let _ = ShowWindow(window.0, SW_SHOWMINNOACTIVE);
        }
        let mut view = TemporaryWindow::capture(window.address()).unwrap();
        view.show().unwrap();
        window.wait_for_minimized(false);
        view.kept = true;
        drop(view);
        assert!(unsafe { IsWindow(Some(window.0)) }.as_bool());
        assert!(!unsafe { IsIconic(window.0) }.as_bool());
    }
}
