use std::sync::LazyLock;

use crate::error::Result;
use crate::hook::HookManager;
use crate::windows_api::window::Window;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum WinEvent {
    AiaEnd = EVENT_AIA_END,
    AiaStart = EVENT_AIA_START,
    ConsoleCaret = EVENT_CONSOLE_CARET,
    ConsoleEnd = EVENT_CONSOLE_END,
    ConsoleEndApplication = EVENT_CONSOLE_END_APPLICATION,
    ConsoleLayout = EVENT_CONSOLE_LAYOUT,
    ConsoleStartApplication = EVENT_CONSOLE_START_APPLICATION,
    ConsoleUpdateRegion = EVENT_CONSOLE_UPDATE_REGION,
    ConsoleUpdateScroll = EVENT_CONSOLE_UPDATE_SCROLL,
    ConsoleUpdateSimple = EVENT_CONSOLE_UPDATE_SIMPLE,
    ObjectAcceleratorChange = EVENT_OBJECT_ACCELERATORCHANGE,
    ObjectCloaked = EVENT_OBJECT_CLOAKED,
    ObjectContentScrolled = EVENT_OBJECT_CONTENTSCROLLED,
    ObjectCreate = EVENT_OBJECT_CREATE,
    ObjectDefActionChange = EVENT_OBJECT_DEFACTIONCHANGE,
    ObjectDescriptionChange = EVENT_OBJECT_DESCRIPTIONCHANGE,
    ObjectDestroy = EVENT_OBJECT_DESTROY,
    ObjectDragCancel = EVENT_OBJECT_DRAGCANCEL,
    ObjectDragComplete = EVENT_OBJECT_DRAGCOMPLETE,
    ObjectDragDropped = EVENT_OBJECT_DRAGDROPPED,
    ObjectDragEnter = EVENT_OBJECT_DRAGENTER,
    ObjectDragLeave = EVENT_OBJECT_DRAGLEAVE,
    ObjectDragStart = EVENT_OBJECT_DRAGSTART,
    ObjectEnd = EVENT_OBJECT_END,
    ObjectFocus = EVENT_OBJECT_FOCUS,
    ObjectHelpChange = EVENT_OBJECT_HELPCHANGE,
    ObjectHide = EVENT_OBJECT_HIDE,
    ObjectHostedObjectsInvalidated = EVENT_OBJECT_HOSTEDOBJECTSINVALIDATED,
    ObjectImeChange = EVENT_OBJECT_IME_CHANGE,
    ObjectImeHide = EVENT_OBJECT_IME_HIDE,
    ObjectImeShow = EVENT_OBJECT_IME_SHOW,
    ObjectInvoked = EVENT_OBJECT_INVOKED,
    ObjectLiveRegionChanged = EVENT_OBJECT_LIVEREGIONCHANGED,
    ObjectLocationChange = EVENT_OBJECT_LOCATIONCHANGE,
    ObjectNameChange = EVENT_OBJECT_NAMECHANGE,
    ObjectParentChange = EVENT_OBJECT_PARENTCHANGE,
    ObjectReorder = EVENT_OBJECT_REORDER,
    ObjectSelection = EVENT_OBJECT_SELECTION,
    ObjectSelectionAdd = EVENT_OBJECT_SELECTIONADD,
    ObjectSelectionRemove = EVENT_OBJECT_SELECTIONREMOVE,
    ObjectSelectionWithin = EVENT_OBJECT_SELECTIONWITHIN,
    ObjectShow = EVENT_OBJECT_SHOW,
    ObjectStateChange = EVENT_OBJECT_STATECHANGE,
    ObjectTextEditConversionTargetChanged = EVENT_OBJECT_TEXTEDIT_CONVERSIONTARGETCHANGED,
    ObjectTextSelectionChanged = EVENT_OBJECT_TEXTSELECTIONCHANGED,
    ObjectUncloaked = EVENT_OBJECT_UNCLOAKED,
    ObjectValueChange = EVENT_OBJECT_VALUECHANGE,
    OemDefinedEnd = EVENT_OEM_DEFINED_END,
    OemDefinedStart = EVENT_OEM_DEFINED_START,
    SystemAlert = EVENT_SYSTEM_ALERT,
    SystemArrangementPreview = EVENT_SYSTEM_ARRANGMENTPREVIEW,
    SystemCaptureEnd = EVENT_SYSTEM_CAPTUREEND,
    SystemCaptureStart = EVENT_SYSTEM_CAPTURESTART,
    SystemContextHelpEnd = EVENT_SYSTEM_CONTEXTHELPEND,
    SystemContextHelpStart = EVENT_SYSTEM_CONTEXTHELPSTART,
    SystemDesktopSwitch = EVENT_SYSTEM_DESKTOPSWITCH,
    SystemDialogEnd = EVENT_SYSTEM_DIALOGEND,
    SystemDialogStart = EVENT_SYSTEM_DIALOGSTART,
    SystemDragDropEnd = EVENT_SYSTEM_DRAGDROPEND,
    SystemDragDropStart = EVENT_SYSTEM_DRAGDROPSTART,
    SystemEnd = EVENT_SYSTEM_END,
    SystemForeground = EVENT_SYSTEM_FOREGROUND,
    SystemImeKeyNotification = EVENT_SYSTEM_IME_KEY_NOTIFICATION,
    SystemMenuEnd = EVENT_SYSTEM_MENUEND,
    SystemMenuPopupEnd = EVENT_SYSTEM_MENUPOPUPEND,
    SystemMenuPopupStart = EVENT_SYSTEM_MENUPOPUPSTART,
    SystemMenuStart = EVENT_SYSTEM_MENUSTART,
    SystemMinimizeEnd = EVENT_SYSTEM_MINIMIZEEND,
    SystemMinimizeStart = EVENT_SYSTEM_MINIMIZESTART,
    SystemMoveSizeEnd = EVENT_SYSTEM_MOVESIZEEND,
    SystemMoveSizeStart = EVENT_SYSTEM_MOVESIZESTART,
    SystemScrollingEnd = EVENT_SYSTEM_SCROLLINGEND,
    SystemScrollingStart = EVENT_SYSTEM_SCROLLINGSTART,
    SystemSound = EVENT_SYSTEM_SOUND,
    SystemSwitchEnd = EVENT_SYSTEM_SWITCHEND,
    SystemSwitchStart = EVENT_SYSTEM_SWITCHSTART,
    SystemSwitcherAppDropped = EVENT_SYSTEM_SWITCHER_APPDROPPED,
    SystemSwitcherAppGrabbed = EVENT_SYSTEM_SWITCHER_APPGRABBED,
    SystemSwitcherAppOverTarget = EVENT_SYSTEM_SWITCHER_APPOVERTARGET,
    SystemSwitcherCancelled = EVENT_SYSTEM_SWITCHER_CANCELLED,
    UiaEventIdSEnd = EVENT_UIA_EVENTID_END,
    UiaEventIdStart = EVENT_UIA_EVENTID_START,
    UiaPropIdSEnd = EVENT_UIA_PROPID_END,
    UiaPropIdStart = EVENT_UIA_PROPID_START,
    /// Fallback for unknown/missing Win32 events
    Unknown(u32),
    // ================== Synthetic events ==================
    /// intended to reduce the amount of events processed by other listeners
    SyntheticForegroundLocationChange,
    SyntheticFullscreenStart,
    SyntheticFullscreenEnd,
    SyntheticMaximizeStart,
    SyntheticMaximizeEnd,
    SyntheticMonitorChanged,
}

impl From<u32> for WinEvent {
    fn from(value: u32) -> Self {
        match value {
            EVENT_AIA_END => Self::AiaEnd,
            EVENT_AIA_START => Self::AiaStart,
            EVENT_CONSOLE_CARET => Self::ConsoleCaret,
            EVENT_CONSOLE_END => Self::ConsoleEnd,
            EVENT_CONSOLE_END_APPLICATION => Self::ConsoleEndApplication,
            EVENT_CONSOLE_LAYOUT => Self::ConsoleLayout,
            EVENT_CONSOLE_START_APPLICATION => Self::ConsoleStartApplication,
            EVENT_CONSOLE_UPDATE_REGION => Self::ConsoleUpdateRegion,
            EVENT_CONSOLE_UPDATE_SCROLL => Self::ConsoleUpdateScroll,
            EVENT_CONSOLE_UPDATE_SIMPLE => Self::ConsoleUpdateSimple,
            EVENT_OBJECT_ACCELERATORCHANGE => Self::ObjectAcceleratorChange,
            EVENT_OBJECT_CLOAKED => Self::ObjectCloaked,
            EVENT_OBJECT_CONTENTSCROLLED => Self::ObjectContentScrolled,
            EVENT_OBJECT_CREATE => Self::ObjectCreate,
            EVENT_OBJECT_DEFACTIONCHANGE => Self::ObjectDefActionChange,
            EVENT_OBJECT_DESCRIPTIONCHANGE => Self::ObjectDescriptionChange,
            EVENT_OBJECT_DESTROY => Self::ObjectDestroy,
            EVENT_OBJECT_DRAGCANCEL => Self::ObjectDragCancel,
            EVENT_OBJECT_DRAGCOMPLETE => Self::ObjectDragComplete,
            EVENT_OBJECT_DRAGDROPPED => Self::ObjectDragDropped,
            EVENT_OBJECT_DRAGENTER => Self::ObjectDragEnter,
            EVENT_OBJECT_DRAGLEAVE => Self::ObjectDragLeave,
            EVENT_OBJECT_DRAGSTART => Self::ObjectDragStart,
            EVENT_OBJECT_END => Self::ObjectEnd,
            EVENT_OBJECT_FOCUS => Self::ObjectFocus,
            EVENT_OBJECT_HELPCHANGE => Self::ObjectHelpChange,
            EVENT_OBJECT_HIDE => Self::ObjectHide,
            EVENT_OBJECT_HOSTEDOBJECTSINVALIDATED => Self::ObjectHostedObjectsInvalidated,
            EVENT_OBJECT_IME_CHANGE => Self::ObjectImeChange,
            EVENT_OBJECT_IME_HIDE => Self::ObjectImeHide,
            EVENT_OBJECT_IME_SHOW => Self::ObjectImeShow,
            EVENT_OBJECT_INVOKED => Self::ObjectInvoked,
            EVENT_OBJECT_LIVEREGIONCHANGED => Self::ObjectLiveRegionChanged,
            EVENT_OBJECT_LOCATIONCHANGE => Self::ObjectLocationChange,
            EVENT_OBJECT_NAMECHANGE => Self::ObjectNameChange,
            EVENT_OBJECT_PARENTCHANGE => Self::ObjectParentChange,
            EVENT_OBJECT_REORDER => Self::ObjectReorder,
            EVENT_OBJECT_SELECTION => Self::ObjectSelection,
            EVENT_OBJECT_SELECTIONADD => Self::ObjectSelectionAdd,
            EVENT_OBJECT_SELECTIONREMOVE => Self::ObjectSelectionRemove,
            EVENT_OBJECT_SELECTIONWITHIN => Self::ObjectSelectionWithin,
            EVENT_OBJECT_SHOW => Self::ObjectShow,
            EVENT_OBJECT_STATECHANGE => Self::ObjectStateChange,
            EVENT_OBJECT_TEXTEDIT_CONVERSIONTARGETCHANGED => {
                Self::ObjectTextEditConversionTargetChanged
            }
            EVENT_OBJECT_TEXTSELECTIONCHANGED => Self::ObjectTextSelectionChanged,
            EVENT_OBJECT_UNCLOAKED => Self::ObjectUncloaked,
            EVENT_OBJECT_VALUECHANGE => Self::ObjectValueChange,
            EVENT_OEM_DEFINED_END => Self::OemDefinedEnd,
            EVENT_OEM_DEFINED_START => Self::OemDefinedStart,
            EVENT_SYSTEM_ALERT => Self::SystemAlert,
            EVENT_SYSTEM_ARRANGMENTPREVIEW => Self::SystemArrangementPreview,
            EVENT_SYSTEM_CAPTUREEND => Self::SystemCaptureEnd,
            EVENT_SYSTEM_CAPTURESTART => Self::SystemCaptureStart,
            EVENT_SYSTEM_CONTEXTHELPEND => Self::SystemContextHelpEnd,
            EVENT_SYSTEM_CONTEXTHELPSTART => Self::SystemContextHelpStart,
            EVENT_SYSTEM_DESKTOPSWITCH => Self::SystemDesktopSwitch,
            EVENT_SYSTEM_DIALOGEND => Self::SystemDialogEnd,
            EVENT_SYSTEM_DIALOGSTART => Self::SystemDialogStart,
            EVENT_SYSTEM_DRAGDROPEND => Self::SystemDragDropEnd,
            EVENT_SYSTEM_DRAGDROPSTART => Self::SystemDragDropStart,
            EVENT_SYSTEM_END => Self::SystemEnd,
            EVENT_SYSTEM_FOREGROUND => Self::SystemForeground,
            EVENT_SYSTEM_IME_KEY_NOTIFICATION => Self::SystemImeKeyNotification,
            EVENT_SYSTEM_MENUEND => Self::SystemMenuEnd,
            EVENT_SYSTEM_MENUPOPUPEND => Self::SystemMenuPopupEnd,
            EVENT_SYSTEM_MENUPOPUPSTART => Self::SystemMenuPopupStart,
            EVENT_SYSTEM_MENUSTART => Self::SystemMenuStart,
            EVENT_SYSTEM_MINIMIZEEND => Self::SystemMinimizeEnd,
            EVENT_SYSTEM_MINIMIZESTART => Self::SystemMinimizeStart,
            EVENT_SYSTEM_MOVESIZEEND => Self::SystemMoveSizeEnd,
            EVENT_SYSTEM_MOVESIZESTART => Self::SystemMoveSizeStart,
            EVENT_SYSTEM_SCROLLINGEND => Self::SystemScrollingEnd,
            EVENT_SYSTEM_SCROLLINGSTART => Self::SystemScrollingStart,
            EVENT_SYSTEM_SOUND => Self::SystemSound,
            EVENT_SYSTEM_SWITCHEND => Self::SystemSwitchEnd,
            EVENT_SYSTEM_SWITCHER_APPDROPPED => Self::SystemSwitcherAppDropped,
            EVENT_SYSTEM_SWITCHER_APPGRABBED => Self::SystemSwitcherAppGrabbed,
            EVENT_SYSTEM_SWITCHER_APPOVERTARGET => Self::SystemSwitcherAppOverTarget,
            EVENT_SYSTEM_SWITCHER_CANCELLED => Self::SystemSwitcherCancelled,
            EVENT_SYSTEM_SWITCHSTART => Self::SystemSwitchStart,
            EVENT_UIA_EVENTID_END => Self::UiaEventIdSEnd,
            EVENT_UIA_EVENTID_START => Self::UiaEventIdStart,
            EVENT_UIA_PROPID_END => Self::UiaPropIdSEnd,
            EVENT_UIA_PROPID_START => Self::UiaPropIdStart,
            _ => Self::Unknown(value),
        }
    }
}

static CACHE: LazyLock<scc::HashMap<isize, WindowCache>> = LazyLock::new(scc::HashMap::new);
static LAZY_LOCATION_CHANGE_EVENT: LazyLock<slu_utils::Throttle<isize>> = LazyLock::new(|| {
    slu_utils::throttle(
        |addr| {
            let window = Window::from(addr);
            if window.is_focused() {
                HookManager::send((WinEvent::SyntheticForegroundLocationChange, window));
            }
        },
        std::time::Duration::from_millis(100),
    )
});

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowCache {
    monitor: String,
    maximized: bool,
    fullscreen: bool,
}

impl From<&Window> for WindowCache {
    fn from(window: &Window) -> Self {
        Self {
            monitor: window.monitor().stable_id().unwrap_or_default(),
            maximized: window.is_maximized(),
            fullscreen: window.is_fullscreen(),
        }
    }
}

impl WinEvent {
    fn get_or_init_cache_for(window: &Window) -> WindowCache {
        match CACHE.entry(window.address()) {
            scc::hash_map::Entry::Occupied(o) => o.get().clone(),
            scc::hash_map::Entry::Vacant(v) => {
                let cache = WindowCache::from(window);
                v.insert_entry(cache.clone());
                cache
            }
        }
    }

    pub fn get_synthetics(&self, origin: &Window) -> Result<Vec<WinEvent>> {
        let mut synthetics = Vec::new();

        match self {
            Self::ObjectLocationChange if origin.is_focused() => {
                LAZY_LOCATION_CHANGE_EVENT.call(origin.address());

                let before = Self::get_or_init_cache_for(origin);
                let now = WindowCache::from(origin);

                if before == now {
                    return Ok(synthetics);
                }

                match (before.maximized, now.maximized) {
                    (false, true) => synthetics.push(Self::SyntheticMaximizeStart),
                    (true, false) => synthetics.push(Self::SyntheticMaximizeEnd),
                    _ => {}
                }

                match (before.fullscreen, now.fullscreen) {
                    (false, true) => synthetics.push(Self::SyntheticFullscreenStart),
                    (true, false) => synthetics.push(Self::SyntheticFullscreenEnd),
                    _ => {}
                }

                if before.monitor != now.monitor {
                    synthetics.push(Self::SyntheticMonitorChanged);
                }

                CACHE.upsert(origin.address(), now);
            }
            Self::SystemMoveSizeEnd => {
                LAZY_LOCATION_CHANGE_EVENT.terminate();
            }
            Self::ObjectDestroy => {
                CACHE.remove(&origin.address());
            }
            _ => {}
        }

        Ok(synthetics)
    }
}
