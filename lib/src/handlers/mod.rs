pub struct SeelenEvent;

#[allow(non_upper_case_globals)]
impl SeelenEvent {
    pub const WorkspacesChanged: &str = "workspaces-changed";
    pub const ActiveWorkspaceChanged: &str = "active-workspace-changed";

    pub const GlobalFocusChanged: &str = "global-focus-changed";
    pub const GlobalMouseMove: &str = "global-mouse-move";

    pub const HandleLayeredHitboxes: &str = "handle-layered";

    pub const MediaSessions: &str = "media-sessions";
    pub const MediaInputs: &str = "media-inputs";
    pub const MediaOutputs: &str = "media-outputs";

    pub const NetworkDefaultLocalIp: &str = "network-default-local-ip";
    pub const NetworkAdapters: &str = "network-adapters";
    pub const NetworkInternetConnection: &str = "network-internet-connection";
    pub const NetworkWlanScanned: &str = "wlan-scanned";

    pub const Notifications: &str = "notifications";

    pub const PowerStatus: &str = "power-status";
    pub const BatteriesStatus: &str = "batteries-status";

    pub const ColorsChanged: &str = "colors-changed";

    pub const TrayInfo: &str = "tray-info";

    pub const ToolbarOverlaped: &str = "set-auto-hide";

    pub const WegOverlaped: &str = "set-auto-hide";
    pub const WegSetFocusedHandle: &str = "set-focused-handle";
    pub const WegSetFocusedExecutable: &str = "set-focused-executable";
    pub const WegUpdateOpenAppInfo: &str = "update-open-app-info";
    pub const WegAddOpenApp: &str = "add-open-app";
    pub const WegRemoveOpenApp: &str = "remove-open-app";

    pub const WMSetReservation: &str = "set-reservation";
    pub const WMUpdateHeight: &str = "update-height";
    pub const WMUpdateWidth: &str = "update-width";
    pub const WMResetWorkspaceSize: &str = "reset-workspace-size";
    pub const WMFocus: &str = "focus";
    pub const WMSetActiveWorkspace: &str = "set-active-workspace";
    pub const WMAddWindow: &str = "add-window";
    pub const WMUpdateWindow: &str = "update-window";
    pub const WMRemoveWindow: &str = "remove-window";
    
    pub const WMForceRetiling: &str = "wm-force-retiling";
    pub const WMSetLayout: &str = "wm-set-layout";
    pub const WMSetOverlayVisibility: &str = "wm-set-overlay-visibility";
    pub const WMSetActiveWindow: &str = "wm-set-active-window";

    pub const StateSettingsChanged: &str = "settings-changed";
    pub const StateWegItemsChanged: &str = "weg-items";
    pub const StateThemesChanged: &str = "themes";
    pub const StatePlaceholdersChanged: &str = "placeholders";
    pub const StateLayoutsChanged: &str = "layouts";
    pub const StateSettingsByAppChanged: &str = "settings-by-app";
    pub const StateHistoryChanged: &str = "history";
    pub const StateIconPacksChanged: &str = "icon-packs";
}
