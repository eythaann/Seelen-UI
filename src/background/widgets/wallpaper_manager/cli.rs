use crate::{error::Result, virtual_desktops::wallpapers::WorkspaceWallpapersManager};

/// Wallpaper manager commands
#[derive(Debug, clap::Args)]
pub struct WallpaperCli {
    #[command(subcommand)]
    command: WallpaperCommand,
}

#[derive(Debug, clap::Subcommand)]
enum WallpaperCommand {
    /// Cycle to the next wallpaper
    Next,
    /// Cycle to the previous wallpaper
    Prev,
}

impl WallpaperCli {
    pub fn process(self) -> Result<()> {
        match self.command {
            WallpaperCommand::Next => WorkspaceWallpapersManager::next(),
            WallpaperCommand::Prev => WorkspaceWallpapersManager::previous(),
        }
        Ok(())
    }
}
