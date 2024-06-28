#![allow(clippy::use_self)]

use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::Deserialize;
use serde::Serialize;

use windows::Win32::Foundation::HWND;
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

use crate::utils::constants::IGNORE_FOCUS_AND_FULLSCREEN;
use crate::windows_api::WindowsApi;

lazy_static! {
    pub static ref FULLSCREENED: Mutex<Vec<HWND>> = Mutex::new(Vec::new());
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
    // ================== Synthetic events ==================
    SyntheticFullscreenStart,
    SyntheticFullscreenEnd,
}

impl TryFrom<u32> for WinEvent {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            EVENT_AIA_END => Ok(Self::AiaEnd),
            EVENT_AIA_START => Ok(Self::AiaStart),
            EVENT_CONSOLE_CARET => Ok(Self::ConsoleCaret),
            EVENT_CONSOLE_END => Ok(Self::ConsoleEnd),
            EVENT_CONSOLE_END_APPLICATION => Ok(Self::ConsoleEndApplication),
            EVENT_CONSOLE_LAYOUT => Ok(Self::ConsoleLayout),
            EVENT_CONSOLE_START_APPLICATION => Ok(Self::ConsoleStartApplication),
            EVENT_CONSOLE_UPDATE_REGION => Ok(Self::ConsoleUpdateRegion),
            EVENT_CONSOLE_UPDATE_SCROLL => Ok(Self::ConsoleUpdateScroll),
            EVENT_CONSOLE_UPDATE_SIMPLE => Ok(Self::ConsoleUpdateSimple),
            EVENT_OBJECT_ACCELERATORCHANGE => Ok(Self::ObjectAcceleratorChange),
            EVENT_OBJECT_CLOAKED => Ok(Self::ObjectCloaked),
            EVENT_OBJECT_CONTENTSCROLLED => Ok(Self::ObjectContentScrolled),
            EVENT_OBJECT_CREATE => Ok(Self::ObjectCreate),
            EVENT_OBJECT_DEFACTIONCHANGE => Ok(Self::ObjectDefActionChange),
            EVENT_OBJECT_DESCRIPTIONCHANGE => Ok(Self::ObjectDescriptionChange),
            EVENT_OBJECT_DESTROY => Ok(Self::ObjectDestroy),
            EVENT_OBJECT_DRAGCANCEL => Ok(Self::ObjectDragCancel),
            EVENT_OBJECT_DRAGCOMPLETE => Ok(Self::ObjectDragComplete),
            EVENT_OBJECT_DRAGDROPPED => Ok(Self::ObjectDragDropped),
            EVENT_OBJECT_DRAGENTER => Ok(Self::ObjectDragEnter),
            EVENT_OBJECT_DRAGLEAVE => Ok(Self::ObjectDragLeave),
            EVENT_OBJECT_DRAGSTART => Ok(Self::ObjectDragStart),
            EVENT_OBJECT_END => Ok(Self::ObjectEnd),
            EVENT_OBJECT_FOCUS => Ok(Self::ObjectFocus),
            EVENT_OBJECT_HELPCHANGE => Ok(Self::ObjectHelpChange),
            EVENT_OBJECT_HIDE => Ok(Self::ObjectHide),
            EVENT_OBJECT_HOSTEDOBJECTSINVALIDATED => Ok(Self::ObjectHostedObjectsInvalidated),
            EVENT_OBJECT_IME_CHANGE => Ok(Self::ObjectImeChange),
            EVENT_OBJECT_IME_HIDE => Ok(Self::ObjectImeHide),
            EVENT_OBJECT_IME_SHOW => Ok(Self::ObjectImeShow),
            EVENT_OBJECT_INVOKED => Ok(Self::ObjectInvoked),
            EVENT_OBJECT_LIVEREGIONCHANGED => Ok(Self::ObjectLiveRegionChanged),
            EVENT_OBJECT_LOCATIONCHANGE => Ok(Self::ObjectLocationChange),
            EVENT_OBJECT_NAMECHANGE => Ok(Self::ObjectNameChange),
            EVENT_OBJECT_PARENTCHANGE => Ok(Self::ObjectParentChange),
            EVENT_OBJECT_REORDER => Ok(Self::ObjectReorder),
            EVENT_OBJECT_SELECTION => Ok(Self::ObjectSelection),
            EVENT_OBJECT_SELECTIONADD => Ok(Self::ObjectSelectionAdd),
            EVENT_OBJECT_SELECTIONREMOVE => Ok(Self::ObjectSelectionRemove),
            EVENT_OBJECT_SELECTIONWITHIN => Ok(Self::ObjectSelectionWithin),
            EVENT_OBJECT_SHOW => Ok(Self::ObjectShow),
            EVENT_OBJECT_STATECHANGE => Ok(Self::ObjectStateChange),
            EVENT_OBJECT_TEXTEDIT_CONVERSIONTARGETCHANGED => {
                Ok(Self::ObjectTextEditConversionTargetChanged)
            }
            EVENT_OBJECT_TEXTSELECTIONCHANGED => Ok(Self::ObjectTextSelectionChanged),
            EVENT_OBJECT_UNCLOAKED => Ok(Self::ObjectUncloaked),
            EVENT_OBJECT_VALUECHANGE => Ok(Self::ObjectValueChange),
            EVENT_OEM_DEFINED_END => Ok(Self::OemDefinedEnd),
            EVENT_OEM_DEFINED_START => Ok(Self::OemDefinedStart),
            EVENT_SYSTEM_ALERT => Ok(Self::SystemAlert),
            EVENT_SYSTEM_ARRANGMENTPREVIEW => Ok(Self::SystemArrangementPreview),
            EVENT_SYSTEM_CAPTUREEND => Ok(Self::SystemCaptureEnd),
            EVENT_SYSTEM_CAPTURESTART => Ok(Self::SystemCaptureStart),
            EVENT_SYSTEM_CONTEXTHELPEND => Ok(Self::SystemContextHelpEnd),
            EVENT_SYSTEM_CONTEXTHELPSTART => Ok(Self::SystemContextHelpStart),
            EVENT_SYSTEM_DESKTOPSWITCH => Ok(Self::SystemDesktopSwitch),
            EVENT_SYSTEM_DIALOGEND => Ok(Self::SystemDialogEnd),
            EVENT_SYSTEM_DIALOGSTART => Ok(Self::SystemDialogStart),
            EVENT_SYSTEM_DRAGDROPEND => Ok(Self::SystemDragDropEnd),
            EVENT_SYSTEM_DRAGDROPSTART => Ok(Self::SystemDragDropStart),
            EVENT_SYSTEM_END => Ok(Self::SystemEnd),
            EVENT_SYSTEM_FOREGROUND => Ok(Self::SystemForeground),
            EVENT_SYSTEM_IME_KEY_NOTIFICATION => Ok(Self::SystemImeKeyNotification),
            EVENT_SYSTEM_MENUEND => Ok(Self::SystemMenuEnd),
            EVENT_SYSTEM_MENUPOPUPEND => Ok(Self::SystemMenuPopupEnd),
            EVENT_SYSTEM_MENUPOPUPSTART => Ok(Self::SystemMenuPopupStart),
            EVENT_SYSTEM_MENUSTART => Ok(Self::SystemMenuStart),
            EVENT_SYSTEM_MINIMIZEEND => Ok(Self::SystemMinimizeEnd),
            EVENT_SYSTEM_MINIMIZESTART => Ok(Self::SystemMinimizeStart),
            EVENT_SYSTEM_MOVESIZEEND => Ok(Self::SystemMoveSizeEnd),
            EVENT_SYSTEM_MOVESIZESTART => Ok(Self::SystemMoveSizeStart),
            EVENT_SYSTEM_SCROLLINGEND => Ok(Self::SystemScrollingEnd),
            EVENT_SYSTEM_SCROLLINGSTART => Ok(Self::SystemScrollingStart),
            EVENT_SYSTEM_SOUND => Ok(Self::SystemSound),
            EVENT_SYSTEM_SWITCHEND => Ok(Self::SystemSwitchEnd),
            EVENT_SYSTEM_SWITCHER_APPDROPPED => Ok(Self::SystemSwitcherAppDropped),
            EVENT_SYSTEM_SWITCHER_APPGRABBED => Ok(Self::SystemSwitcherAppGrabbed),
            EVENT_SYSTEM_SWITCHER_APPOVERTARGET => Ok(Self::SystemSwitcherAppOverTarget),
            EVENT_SYSTEM_SWITCHER_CANCELLED => Ok(Self::SystemSwitcherCancelled),
            EVENT_SYSTEM_SWITCHSTART => Ok(Self::SystemSwitchStart),
            EVENT_UIA_EVENTID_END => Ok(Self::UiaEventIdSEnd),
            EVENT_UIA_EVENTID_START => Ok(Self::UiaEventIdStart),
            EVENT_UIA_PROPID_END => Ok(Self::UiaPropIdSEnd),
            EVENT_UIA_PROPID_START => Ok(Self::UiaPropIdStart),

            _ => Err(()),
        }
    }
}

impl WinEvent {
    pub fn should_handle_fullscreen_events(&self, hwnd: HWND) -> bool {
        let title = WindowsApi::get_window_text(hwnd);
        let is_foreground_window = hwnd == WindowsApi::get_foreground_window();

        (is_foreground_window || *self == Self::SystemMinimizeStart)
            && !title.starts_with("Seelen")
            && !IGNORE_FOCUS_AND_FULLSCREEN.contains(&title)
    }

    pub fn get_synthetic(&self, origin: HWND) -> Option<WinEvent> {
        match self {
            Self::ObjectShow
            | Self::ObjectCreate
            | Self::ObjectUncloaked
            | Self::SystemMinimizeEnd => {
                let is_creating_invisible_window =
                    *self == Self::ObjectCreate && !WindowsApi::is_window_visible(origin);
                if is_creating_invisible_window || !self.should_handle_fullscreen_events(origin) {
                    return None;
                }

                if WindowsApi::is_fullscreen(origin).ok()? {
                    FULLSCREENED.lock().push(origin);
                    Some(Self::SyntheticFullscreenStart)
                } else {
                    None
                }
            }
            Self::ObjectHide
            | Self::ObjectDestroy
            | Self::ObjectCloaked
            | Self::SystemMinimizeStart => {
                if !self.should_handle_fullscreen_events(origin) {
                    return None;
                }

                let mut fullscreened = FULLSCREENED.lock();
                if fullscreened.contains(&origin) {
                    fullscreened.retain(|&x| x != origin);
                    Some(Self::SyntheticFullscreenEnd)
                } else {
                    None
                }
            }
            Self::ObjectLocationChange => {
                if !self.should_handle_fullscreen_events(origin) {
                    return None;
                }

                let mut fullscreened = FULLSCREENED.lock();
                let was_fullscreen = fullscreened.contains(&origin);
                let is_fullscreen = WindowsApi::is_fullscreen(origin).ok()?;

                // state no changed
                if was_fullscreen == is_fullscreen {
                    return None;
                }

                if is_fullscreen {
                    fullscreened.push(origin);
                    Some(Self::SyntheticFullscreenStart)
                } else {
                    fullscreened.retain(|&x| x != origin);
                    Some(Self::SyntheticFullscreenEnd)
                }
            }
            _ => None,
        }
    }
}
