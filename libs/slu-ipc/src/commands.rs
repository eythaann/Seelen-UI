use std::path::PathBuf;

use seelen_core::resource::ResourceKind;
use serde::{Deserialize, Serialize};

// ===== Execution mode =====

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecutionMode {
    Direct,
    MainInstance,
}

pub trait SluCliCommand {
    fn execution_mode(&self) -> CommandExecutionMode {
        CommandExecutionMode::MainInstance
    }
}

// ===== Top-level =====

/// Seelen UI Command Line Interface
#[derive(Debug, clap::Parser)]
#[command(version, name = "Seelen UI")]
pub struct AppCli {
    /// Prints some extra information on the console.
    #[arg(long, default_value_t)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum AppCommand {
    /// Opens the Seelen settings gui.
    Settings,
    VirtualDesk(VirtualDesktopCli),
    Debugger(DebuggerCli),
    WindowManager(WindowManagerCli),
    Popup(PopupsCli),
    Weg(WegCli),
    Widget(WidgetCli),
    Resource(ResourceManagerCli),
    Art(ArtCli),
    TaskSwitcher(TaskSwitcherClient),
    Wallpaper(WallpaperCli),
}

impl SluCliCommand for AppCommand {
    fn execution_mode(&self) -> CommandExecutionMode {
        match self {
            AppCommand::Art(_) => CommandExecutionMode::Direct,
            AppCommand::Resource(r) => r.execution_mode(),
            _ => CommandExecutionMode::MainInstance,
        }
    }
}

// ===== Debugger =====

/// Debugger cli
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct DebuggerCli {
    #[command(subcommand)]
    pub subcommand: DebuggerSubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum DebuggerSubCommand {
    /// Toggles the tracing of window events
    ToggleWinEvents,
    /// Toggles the tracing of mutex lock
    ToggleTraceLock,
}

// ===== Art =====

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum)]
pub enum ArtVariant {
    SeelenLogo,
    SeelenLogoSmall,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::Args)]
pub struct ArtCli {
    pub variant: ArtVariant,
}

// ===== Resource =====

/// Manage the Seelen Resources.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct ResourceManagerCli {
    #[command(subcommand)]
    pub subcommand: ResourceSubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum ResourceSubCommand {
    /// loads a widget into the internal registry
    Load {
        kind: ClapResourceKind,
        path: PathBuf,
    },
    /// deletes the widget from internal registry
    Unload {
        kind: ClapResourceKind,
        path: PathBuf,
    },
    /// Bundles a widget into a single file to be shared.
    ///
    /// Exported file will be at the same location as the passed path
    /// with a filename `export_{date}.yml`.
    Bundle {
        kind: ClapResourceKind,
        path: PathBuf,
    },
    /// Translates a resource text file to all the supported languages by Seelen UI
    /// this file should contain the source language key and value in order to be translated.
    ///
    /// Example:
    /// ```yaml
    /// # The file will be completed with the rest of the supported languages
    /// en: Some text to be translated
    /// ```
    Translate {
        /// The file to be translated
        path: PathBuf,
        /// The source language of the file, by default `en`
        source_lang: Option<String>,
    },
}

impl SluCliCommand for ResourceSubCommand {
    fn execution_mode(&self) -> CommandExecutionMode {
        match self {
            ResourceSubCommand::Bundle { .. } => CommandExecutionMode::Direct,
            ResourceSubCommand::Translate { .. } => CommandExecutionMode::Direct,
            ResourceSubCommand::Load { .. } | ResourceSubCommand::Unload { .. } => {
                CommandExecutionMode::MainInstance
            }
        }
    }
}

impl SluCliCommand for ResourceManagerCli {
    fn execution_mode(&self) -> CommandExecutionMode {
        self.subcommand.execution_mode()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
pub enum ClapResourceKind {
    Theme,
    Widget,
    Plugin,
    IconPack,
    Wallpaper,
    SoundPack,
}

impl From<ClapResourceKind> for ResourceKind {
    fn from(value: ClapResourceKind) -> Self {
        match value {
            ClapResourceKind::Theme => ResourceKind::Theme,
            ClapResourceKind::IconPack => ResourceKind::IconPack,
            ClapResourceKind::Widget => ResourceKind::Widget,
            ClapResourceKind::Plugin => ResourceKind::Plugin,
            ClapResourceKind::Wallpaper => ResourceKind::Wallpaper,
            ClapResourceKind::SoundPack => ResourceKind::SoundPack,
        }
    }
}

// ===== VirtualDesktop =====

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "vd")]
pub struct VirtualDesktopCli {
    #[command(subcommand)]
    pub subcommand: VdCommand,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, clap::Subcommand)]
pub enum VdCommand {
    /// Send the window to the specified workspace
    SendToWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Switch to the specified workspace
    SwitchWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Send the window to the specified workspace and switch to it
    MoveToWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Switch to the next workspace
    SwitchNext,
    /// Switch to the previous workspace
    SwitchPrev,
    /// Create a new workspace
    CreateNewWorkspace,
    /// Destroy the current workspace (will do nothing if there's only one workspace)
    DestroyCurrentWorkspace,
}

// ===== Widget =====

#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct WidgetCli {
    #[command(subcommand)]
    pub command: WidgetCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WidgetCommand {
    /// Triggers a widget
    Trigger { widget_id: String },
}

// ===== Popups =====

/// Manage the Seelen Popups.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct PopupsCli {
    #[command(subcommand)]
    pub subcommand: PopupsCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum PopupsCommand {
    Create {
        /// json config
        config: String,
    },
    Update {
        /// id
        id: String,
        /// json config
        config: String,
    },
    Close {
        /// id
        id: String,
    },
    #[command(hide = true)]
    InternalSetShortcut { json: String },
}

// ===== Weg =====

/// Seelen's dock commands
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct WegCli {
    #[command(subcommand)]
    pub subcommand: WegCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WegCommand {
    /// Set foreground to the application which is idx-nth on the weg. If it is not started, then starts it.
    ForegroundOrRunApp {
        /// Which index should be started on weg.
        index: usize,
    },
}

// ===== WindowManager =====

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum)]
pub enum AllowedReservations {
    Left,
    Right,
    Top,
    Bottom,
    Stack,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum NodeSiblingSide {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum Sizing {
    Increase,
    Decrease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum StepWay {
    Next,
    Prev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum Axis {
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
}

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "wm")]
pub struct WindowManagerCli {
    #[command(subcommand)]
    pub subcommand: WmCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::Subcommand)]
pub enum WmCommand {
    /// Open Dev Tools (only works if the app is running in dev mode)
    Debug,
    /// Toggles the Seelen Window Manager.
    Toggle,
    /// Reserve space for a incoming window.
    Reserve {
        /// The position of the new window.
        side: AllowedReservations,
    },
    /// Cancels the current reservation
    CancelReservation,
    /// Increases or decreases the size of the window
    Width {
        /// What to do with the width.
        action: Sizing,
    },
    /// Increases or decreases the size of the window
    Height {
        /// What to do with the height.
        action: Sizing,
    },
    /// Resets the size of the containers in current workspace to the default size.
    ResetWorkspaceSize,
    /// Toggles the floating state of the window
    ToggleFloat,
    /// Toggles workspace layout mode to monocle (single stack)
    ToggleMonocle,
    /// Cycles the foregrounf node if it is a stack
    CycleStack { way: StepWay },
    /// Focuses the window in the specified position.
    Focus {
        /// The position of the window to focus.
        side: NodeSiblingSide,
    },
    /// Moves the window to the specified position
    Move {
        /// Direction to move
        side: NodeSiblingSide,
    },
    /// Moves the window to another monitor in the specified side
    MoveToMonitor {
        /// Direction to move
        side: NodeSiblingSide,
    },
}

// ===== TaskSwitcher =====

#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct TaskSwitcherClient {
    #[command(subcommand)]
    pub command: TaskSwitcherCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum TaskSwitcherCommand {
    SelectNextTask {
        #[clap(long)]
        auto_confirm: bool,
    },
    SelectPreviousTask {
        #[clap(long)]
        auto_confirm: bool,
    },
}

// ===== Wallpaper =====

/// Wallpaper manager commands
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct WallpaperCli {
    #[command(subcommand)]
    pub command: WallpaperCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WallpaperCommand {
    /// Cycle to the next wallpaper
    Next,
    /// Cycle to the previous wallpaper
    Prev,
}
