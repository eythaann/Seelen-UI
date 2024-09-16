#![allow(clippy::use_self)]

use lazy_static::lazy_static;
use parking_lot::Mutex;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::UI::WindowsAndMessaging::EVENT_AIA_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_AIA_START;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_CARET;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_END_APPLICATION;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_LAYOUT;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_START_APPLICATION;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_UPDATE_REGION;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_UPDATE_SCROLL;
use windows::Win32::UI::WindowsAndMessaging::EVENT_CONSOLE_UPDATE_SIMPLE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_ACCELERATORCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_CLOAKED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_CONTENTSCROLLED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_CREATE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DEFACTIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DESCRIPTIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGCANCEL;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGCOMPLETE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGDROPPED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGENTER;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGLEAVE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_DRAGSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_FOCUS;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_HELPCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_HIDE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_HOSTEDOBJECTSINVALIDATED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_IME_CHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_IME_HIDE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_IME_SHOW;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_INVOKED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LIVEREGIONCHANGED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_NAMECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_PARENTCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_REORDER;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SELECTION;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SELECTIONADD;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SELECTIONREMOVE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SELECTIONWITHIN;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_SHOW;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_STATECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_TEXTEDIT_CONVERSIONTARGETCHANGED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_TEXTSELECTIONCHANGED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_UNCLOAKED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_VALUECHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OEM_DEFINED_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OEM_DEFINED_START;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_ALERT;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_ARRANGMENTPREVIEW;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_CAPTUREEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_CAPTURESTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_CONTEXTHELPEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_CONTEXTHELPSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_DESKTOPSWITCH;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_DIALOGEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_DIALOGSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_DRAGDROPEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_DRAGDROPSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_IME_KEY_NOTIFICATION;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MENUEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MENUPOPUPEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MENUPOPUPSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MENUSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZESTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZESTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SCROLLINGEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SCROLLINGSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SOUND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SWITCHER_APPGRABBED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SWITCHER_APPOVERTARGET;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SWITCHER_CANCELLED;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_SWITCHSTART;
use windows::Win32::UI::WindowsAndMessaging::EVENT_UIA_EVENTID_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_UIA_EVENTID_START;
use windows::Win32::UI::WindowsAndMessaging::EVENT_UIA_PROPID_END;
use windows::Win32::UI::WindowsAndMessaging::EVENT_UIA_PROPID_START;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_SWITCHEND, EVENT_SYSTEM_SWITCHER_APPDROPPED,
};

use crate::error_handler::Result;
use crate::trace_lock;
use crate::windows_api::window::Window;
use crate::windows_api::WindowsApi;

lazy_static! {
    static ref FULLSCREENED: Mutex<Option<SyntheticFullscreenData>> = Mutex::new(None);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SyntheticFullscreenData {
    pub handle: HWND,
    pub monitor: HMONITOR,
}

unsafe impl Send for SyntheticFullscreenData {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    SyntheticFullscreenStart(SyntheticFullscreenData),
    SyntheticFullscreenEnd(SyntheticFullscreenData),
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

impl WinEvent {
    pub fn get_synthetics(&self, origin: HWND) -> Result<Vec<WinEvent>> {
        let mut synthetics = Vec::new();
        match self {
            Self::SystemForeground | Self::ObjectLocationChange => {
                if origin == WindowsApi::get_foreground_window() {
                    let mut latest_fullscreened = trace_lock!(FULLSCREENED);
                    let window = Window::from(origin);
                    let is_origin_fullscreen = window.is_fullscreen()
                        && !window.is_desktop()
                        && !window.is_seelen_overlay();

                    match *latest_fullscreened {
                        Some(latest) if latest.handle == origin => {
                            // exiting fullscreen
                            if !is_origin_fullscreen {
                                *latest_fullscreened = None;
                                synthetics.push(Self::SyntheticFullscreenEnd(latest));
                            }
                        }
                        _ => {
                            // remove fullscreen of latest when foregrounding another window
                            if let Some(old) = latest_fullscreened.take() {
                                synthetics.push(Self::SyntheticFullscreenEnd(old));
                            }
                            // if new foregrounded window is fullscreen emit it
                            if is_origin_fullscreen {
                                let data = SyntheticFullscreenData {
                                    handle: origin,
                                    monitor: WindowsApi::monitor_from_window(origin),
                                };
                                *latest_fullscreened = Some(data);
                                synthetics.push(Self::SyntheticFullscreenStart(data));
                            }
                        }
                    }
                }
            }
            _ => {}
        };
        Ok(synthetics)
    }
}
