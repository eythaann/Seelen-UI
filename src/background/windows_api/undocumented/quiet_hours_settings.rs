//! Undocumented interfaces behind the "Do not disturb" (Windows 11) / "Focus assist"
//! (Windows 10) toggle. There is no public api to change it, `ToastNotificationManagerForUser`
//! only exposes the current mode as read-only.
//!
//! Reverse engineered by Rafael Rivera from Windows 10 1903 (MIT):
//! - https://gist.github.com/riverar/085d98ffb1343e92225a10817109b2e3
//! - https://withinrafael.com/2019/09/19/determine-if-your-app-is-in-a-focus-assist-profiles-priority-list/
//!
//! The whole interfaces are declared to have an overview of them, even if most methods are
//! unused. Being undocumented, they can change between Windows builds.
//!
//! Every `PWSTR` returned (and the arrays of them) is allocated with `CoTaskMemAlloc` and must
//! be freed by the caller.

#![allow(dead_code)]

use std::{ffi::c_void, mem::zeroed};

use windows::core::{GUID, HRESULT, Interface, PCWSTR, PWSTR, Param, Result};
use windows_core::BOOL;

#[allow(non_upper_case_globals)]
pub const QuietHoursSettings: GUID = GUID::from_u128(0xf53321fa_34f8_4b7f_b9a3_361877cb94cf);

pub const QUIET_HOURS_PROFILE_UNRESTRICTED: &str = "Microsoft.QuietHoursProfile.Unrestricted";
pub const QUIET_HOURS_PROFILE_PRIORITY_ONLY: &str = "Microsoft.QuietHoursProfile.PriorityOnly";
pub const QUIET_HOURS_PROFILE_ALARMS_ONLY: &str = "Microsoft.QuietHoursProfile.AlarmsOnly";

// =============================
// ====IQuietHoursSettings====
// =============================

windows_core::imp::define_interface!(
    IQuietHoursSettings,
    IQuietHoursSettings_Vtbl,
    0x6bff4732_81ec_4ffb_ae67_b6c1bc29631f
);

windows_core::imp::interface_hierarchy!(IQuietHoursSettings, windows_core::IUnknown);
#[allow(non_snake_case)]
impl IQuietHoursSettings {
    pub unsafe fn UserSelectedProfile(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_UserSelectedProfile)(
                Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }

    pub unsafe fn SetUserSelectedProfile(&self, profile_id: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).put_UserSelectedProfile)(
                Interface::as_raw(self),
                profile_id.param().abi(),
            )
            .ok()
        }
    }

    pub unsafe fn GetProfile(&self, profile_id: impl Param<PCWSTR>) -> Result<IQuietHoursProfile> {
        unsafe {
            let mut result__ = zeroed();
            (Interface::vtable(self).GetProfile)(
                Interface::as_raw(self),
                profile_id.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }

    /// `QH_PROFILE_DATA` layout is unknown.
    pub unsafe fn GetAllProfileData(
        &self,
        count: *mut u32,
        data: *mut QH_PROFILE_DATA,
    ) -> Result<()> {
        unsafe {
            (Interface::vtable(self).GetAllProfileData)(Interface::as_raw(self), count, data).ok()
        }
    }

    pub unsafe fn GetDisplayNameForProfile(&self, profile_id: impl Param<PCWSTR>) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).GetDisplayNameForProfile)(
                Interface::as_raw(self),
                profile_id.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }

    pub unsafe fn QuietMomentsManager(&self) -> Result<IQuietMomentsManager> {
        unsafe {
            let mut result__ = zeroed();
            (Interface::vtable(self).get_QuietMomentsManager)(
                Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }

    pub unsafe fn OffProfileId(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_OffProfileId)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn ActiveQuietMomentProfile(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_ActiveQuietMomentProfile)(
                Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }

    pub unsafe fn SetActiveQuietMomentProfile(&self, profile_id: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).put_ActiveQuietMomentProfile)(
                Interface::as_raw(self),
                profile_id.param().abi(),
            )
            .ok()
        }
    }

    /// Effective profile: the user selected one, or the one of an active quiet moment
    /// (automatic rules like gaming, full screen, duplicating display, etc).
    pub unsafe fn ActiveProfile(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_ActiveProfile)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn QuietHoursPinnedContactManager(&self) -> Result<IQuietHoursPinnedContactManager> {
        unsafe {
            let mut result__ = zeroed();
            (Interface::vtable(self).get_QuietHoursPinnedContactManager)(
                Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }

    /// Declared as `[propput]` on the IDL but with an `[out, retval]` param, so it behaves
    /// as a getter.
    pub unsafe fn QuietMomentsShowSummaryEnabled(&self) -> Result<bool> {
        unsafe {
            let mut result__ = zeroed::<BOOL>();
            (Interface::vtable(self).put_QuietMomentsShowSummaryEnabled)(
                Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__.as_bool())
        }
    }

    /// Returns `(count, array)`, both the array and each string must be freed.
    pub unsafe fn GetAlwaysAllowedApps(&self) -> Result<(u32, *mut PWSTR)> {
        unsafe {
            let mut count = 0u32;
            let mut result__ = zeroed::<*mut PWSTR>();
            (Interface::vtable(self).GetAlwaysAllowedApps)(
                Interface::as_raw(self),
                &mut count,
                &mut result__,
            )
            .map(|| (count, result__))
        }
    }

    pub unsafe fn StartProcessing(&self) -> Result<()> {
        unsafe { (Interface::vtable(self).StartProcessing)(Interface::as_raw(self)).ok() }
    }

    pub unsafe fn StopProcessing(&self) -> Result<()> {
        unsafe { (Interface::vtable(self).StopProcessing)(Interface::as_raw(self)).ok() }
    }
}

#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IQuietHoursSettings_Vtbl {
    pub base__: ::windows::core::IUnknown_Vtbl,
    pub get_UserSelectedProfile:
        unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub put_UserSelectedProfile: unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub GetProfile:
        unsafe extern "system" fn(this: *mut c_void, PCWSTR, *mut *mut c_void) -> HRESULT,
    pub GetAllProfileData:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut QH_PROFILE_DATA) -> HRESULT,
    pub GetDisplayNameForProfile:
        unsafe extern "system" fn(this: *mut c_void, PCWSTR, *mut PWSTR) -> HRESULT,
    pub get_QuietMomentsManager:
        unsafe extern "system" fn(this: *mut c_void, *mut *mut c_void) -> HRESULT,
    pub get_OffProfileId: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub get_ActiveQuietMomentProfile:
        unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub put_ActiveQuietMomentProfile:
        unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub get_ActiveProfile: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub get_QuietHoursPinnedContactManager:
        unsafe extern "system" fn(this: *mut c_void, *mut *mut c_void) -> HRESULT,
    pub put_QuietMomentsShowSummaryEnabled:
        unsafe extern "system" fn(this: *mut c_void, *mut BOOL) -> HRESULT,
    pub GetAlwaysAllowedApps:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut *mut PWSTR) -> HRESULT,
    pub StartProcessing: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub StopProcessing: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
}

// =============================
// ====IQuietHoursProfile====
// =============================

windows_core::imp::define_interface!(
    IQuietHoursProfile,
    IQuietHoursProfile_Vtbl,
    0xe813fe81_62b6_417d_b951_9d2e08486ac1
);

windows_core::imp::interface_hierarchy!(IQuietHoursProfile, windows_core::IUnknown);
#[allow(non_snake_case)]
impl IQuietHoursProfile {
    pub unsafe fn DisplayName(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_DisplayName)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn ProfileId(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_ProfileId)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn GetSetting(&self, setting: i32) -> Result<i32> {
        unsafe {
            let mut result__ = 0i32;
            (Interface::vtable(self).GetSetting)(Interface::as_raw(self), setting, &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn PutSetting(&self, setting: i32, value: i32) -> Result<()> {
        unsafe {
            (Interface::vtable(self).PutSetting)(Interface::as_raw(self), setting, value).ok()
        }
    }

    pub unsafe fn IsCustomizable(&self) -> Result<bool> {
        unsafe {
            let mut result__ = zeroed::<BOOL>();
            (Interface::vtable(self).get_IsCustomizable)(Interface::as_raw(self), &mut result__)
                .map(|| result__.as_bool())
        }
    }

    /// The IDL declares the out param as `LPWSTR*` (not `LPWSTR**` like `GetAllowedApps`),
    /// it is probably wrong, check it before use.
    pub unsafe fn GetAllowedContacts(&self) -> Result<(u32, PWSTR)> {
        unsafe {
            let mut count = 0u32;
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).GetAllowedContacts)(
                Interface::as_raw(self),
                &mut count,
                &mut result__,
            )
            .map(|| (count, result__))
        }
    }

    pub unsafe fn AddAllowedContact(&self, contact: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).AddAllowedContact)(
                Interface::as_raw(self),
                contact.param().abi(),
            )
            .ok()
        }
    }

    pub unsafe fn RemoveAllowedContact(&self, contact: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).RemoveAllowedContact)(
                Interface::as_raw(self),
                contact.param().abi(),
            )
            .ok()
        }
    }

    /// Returns `(count, array)`, both the array and each string must be freed.
    pub unsafe fn GetAllowedApps(&self) -> Result<(u32, *mut PWSTR)> {
        unsafe {
            let mut count = 0u32;
            let mut result__ = zeroed::<*mut PWSTR>();
            (Interface::vtable(self).GetAllowedApps)(
                Interface::as_raw(self),
                &mut count,
                &mut result__,
            )
            .map(|| (count, result__))
        }
    }

    pub unsafe fn AddAllowedApp(&self, app: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).AddAllowedApp)(Interface::as_raw(self), app.param().abi()).ok()
        }
    }

    pub unsafe fn RemoveAllowedApp(&self, app: impl Param<PCWSTR>) -> Result<()> {
        unsafe {
            (Interface::vtable(self).RemoveAllowedApp)(Interface::as_raw(self), app.param().abi())
                .ok()
        }
    }

    pub unsafe fn Description(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_Description)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn CustomizeLinkText(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_CustomizeLinkText)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }

    pub unsafe fn RestrictiveLevel(&self) -> Result<PWSTR> {
        unsafe {
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).get_RestrictiveLevel)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}

#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IQuietHoursProfile_Vtbl {
    pub base__: ::windows::core::IUnknown_Vtbl,
    pub get_DisplayName: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub get_ProfileId: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub GetSetting: unsafe extern "system" fn(this: *mut c_void, i32, *mut i32) -> HRESULT,
    pub PutSetting: unsafe extern "system" fn(this: *mut c_void, i32, i32) -> HRESULT,
    pub get_IsCustomizable: unsafe extern "system" fn(this: *mut c_void, *mut BOOL) -> HRESULT,
    pub GetAllowedContacts:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut PWSTR) -> HRESULT,
    pub AddAllowedContact: unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub RemoveAllowedContact: unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub GetAllowedApps:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut *mut PWSTR) -> HRESULT,
    pub AddAllowedApp: unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub RemoveAllowedApp: unsafe extern "system" fn(this: *mut c_void, PCWSTR) -> HRESULT,
    pub get_Description: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub get_CustomizeLinkText: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
    pub get_RestrictiveLevel: unsafe extern "system" fn(this: *mut c_void, *mut PWSTR) -> HRESULT,
}

// =============================
// ====IQuietMomentsManager====
// =============================

windows_core::imp::define_interface!(
    IQuietMomentsManager,
    IQuietMomentsManager_Vtbl,
    0xb0217783_87b7_422c_b902_5c148c14f150
);

windows_core::imp::interface_hierarchy!(IQuietMomentsManager, windows_core::IUnknown);
#[allow(non_snake_case)]
impl IQuietMomentsManager {
    /// Returns `(count, array)`, the array must be freed.
    pub unsafe fn GetAllQuietMomentModes(&self) -> Result<(u32, *mut u32)> {
        unsafe {
            let mut count = 0u32;
            let mut result__ = zeroed::<*mut u32>();
            (Interface::vtable(self).GetAllQuietMomentModes)(
                Interface::as_raw(self),
                &mut count,
                &mut result__,
            )
            .map(|| (count, result__))
        }
    }

    pub unsafe fn GetQuietMoment(&self, quiet_moment_id: u32) -> Result<IQuietMoment> {
        unsafe {
            let mut result__ = zeroed();
            (Interface::vtable(self).GetQuietMoment)(
                Interface::as_raw(self),
                quiet_moment_id,
                &mut result__,
            )
            .and_then(|| windows_core::Type::from_abi(result__))
        }
    }

    pub unsafe fn TurnOffCurrentlyActiveQuietMoment(&self) -> Result<()> {
        unsafe {
            (Interface::vtable(self).TurnOffCurrentlyActiveQuietMoment)(Interface::as_raw(self))
                .ok()
        }
    }

    pub unsafe fn GetActiveQuietMoment(&self) -> Result<u32> {
        unsafe {
            let mut result__ = 0u32;
            (Interface::vtable(self).GetActiveQuietMoment)(Interface::as_raw(self), &mut result__)
                .map(|| result__)
        }
    }
}

#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IQuietMomentsManager_Vtbl {
    pub base__: ::windows::core::IUnknown_Vtbl,
    pub GetAllQuietMomentModes:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut *mut u32) -> HRESULT,
    pub GetQuietMoment:
        unsafe extern "system" fn(this: *mut c_void, u32, *mut *mut c_void) -> HRESULT,
    pub TurnOffCurrentlyActiveQuietMoment: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub GetActiveQuietMoment: unsafe extern "system" fn(this: *mut c_void, *mut u32) -> HRESULT,
}

// =============================
// ====IQuietMoment====
// =============================

windows_core::imp::define_interface!(
    IQuietMoment,
    IQuietMoment_Vtbl,
    0xaf86e2e0_b12d_4c6a_9c5a_d7aa65101e90
);

windows_core::imp::interface_hierarchy!(IQuietMoment, windows_core::IUnknown);

/// Incomplete on the reverse engineered IDL, its methods are unknown.
#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IQuietMoment_Vtbl {
    pub base__: ::windows::core::IUnknown_Vtbl,
}

// =============================
// ====IQuietHoursPinnedContactManager====
// =============================

windows_core::imp::define_interface!(
    IQuietHoursPinnedContactManager,
    IQuietHoursPinnedContactManager_Vtbl,
    0xcd86a976_8ea9_404b_a197_42e73dbaa901
);

windows_core::imp::interface_hierarchy!(IQuietHoursPinnedContactManager, windows_core::IUnknown);
#[allow(non_snake_case)]
impl IQuietHoursPinnedContactManager {
    /// Same doubt as `IQuietHoursProfile::GetAllowedContacts` about the out param type.
    pub unsafe fn GetPinnedContactList(&self) -> Result<(u32, PWSTR)> {
        unsafe {
            let mut count = 0u32;
            let mut result__ = zeroed::<PWSTR>();
            (Interface::vtable(self).GetPinnedContactList)(
                Interface::as_raw(self),
                &mut count,
                &mut result__,
            )
            .map(|| (count, result__))
        }
    }
}

#[repr(C)]
#[doc(hidden)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct IQuietHoursPinnedContactManager_Vtbl {
    pub base__: ::windows::core::IUnknown_Vtbl,
    pub GetPinnedContactList:
        unsafe extern "system" fn(this: *mut c_void, *mut u32, *mut PWSTR) -> HRESULT,
}

// =============================
// ====QH_PROFILE_DATA====
// =============================

/// Incomplete on the reverse engineered IDL, its layout is unknown so it is only usable
/// behind a pointer.
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct QH_PROFILE_DATA {
    _opaque: [u8; 0],
}
