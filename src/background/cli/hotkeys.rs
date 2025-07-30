use seelen_core::state::shortcuts::SluHotkeyAction;

use crate::{
    cli::application::AppCliCommand,
    modules::virtual_desk::cli::{VdCommand, VirtualDesktopCli},
    seelen_rofi::cli::{AppLauncherCli, LauncherSubCommand},
    seelen_weg::cli::{WegCli, WegCommand},
    seelen_wm_v2::cli::{Sizing, WindowManagerCli, WmCommand},
};

impl AppCliCommand {
    fn weg(subcommand: WegCommand) -> Self {
        Self::Weg(WegCli { subcommand })
    }

    fn wm(subcommand: WmCommand) -> Self {
        Self::WindowManager(WindowManagerCli { subcommand })
    }

    fn vd(subcommand: VdCommand) -> Self {
        Self::VirtualDesk(VirtualDesktopCli { subcommand })
    }

    fn start(subcommand: LauncherSubCommand) -> Self {
        Self::Launcher(AppLauncherCli { subcommand })
    }
}

pub fn hotkey_action_to_cli_command(action: SluHotkeyAction) -> AppCliCommand {
    use SluHotkeyAction::*;

    match action {
        // Virtual Desktop
        SwitchToNextWorkspace => AppCliCommand::vd(VdCommand::SwitchNext),
        SwitchToPreviousWorkspace => AppCliCommand::vd(VdCommand::SwitchPrev),
        SwitchWorkspace(index) => AppCliCommand::vd(VdCommand::SwitchWorkspace { index }),
        MoveToWorkspace(index) => AppCliCommand::vd(VdCommand::MoveToWorkspace { index }),
        SendToWorkspace(index) => AppCliCommand::vd(VdCommand::SendToWorkspace { index }),
        DestroyCurrentWorkspace => AppCliCommand::vd(VdCommand::DestroyCurrentWorkspace),
        // Weg
        StartWegApp(index) => AppCliCommand::weg(WegCommand::ForegroundOrRunApp { index }),
        // App Launcher / Start Menu
        ToggleLauncher => AppCliCommand::start(LauncherSubCommand::Toggle),
        // Window Manager
        IncreaseWidth => AppCliCommand::wm(WmCommand::Width {
            action: Sizing::Increase,
        }),
        DecreaseWidth => AppCliCommand::wm(WmCommand::Width {
            action: Sizing::Decrease,
        }),
        IncreaseHeight => AppCliCommand::wm(WmCommand::Height {
            action: Sizing::Increase,
        }),
        DecreaseHeight => AppCliCommand::wm(WmCommand::Height {
            action: Sizing::Decrease,
        }),
        RestoreSizes => AppCliCommand::wm(WmCommand::ResetWorkspaceSize),
        // Window Manger focused window sizing
        FocusTop => unimplemented!(),
        FocusBottom => unimplemented!(),
        FocusLeft => unimplemented!(),
        FocusRight => unimplemented!(),
        // Window Manager focused window positioning
        MoveWindowUp => unimplemented!(),
        MoveWindowDown => unimplemented!(),
        MoveWindowLeft => unimplemented!(),
        MoveWindowRight => unimplemented!(),
        // Tiling window manager reservation
        ReserveTop => unimplemented!(),
        ReserveBottom => unimplemented!(),
        ReserveLeft => unimplemented!(),
        ReserveRight => unimplemented!(),
        ReserveFloat => unimplemented!(),
        ReserveStack => unimplemented!(),
        // others
        MiscOpenSettings => unimplemented!(),
        MiscToggleLockTracing => unimplemented!(),
        MiscToggleWinEventTracing => unimplemented!(),
    }
}
