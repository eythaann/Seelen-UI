use std::path::PathBuf;

use seelen_core::{
    resource::{ResourceKind, SluResource},
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
};
use serde::{Deserialize, Serialize};

use crate::{error::Result, resources::RESOURCES};

/// Manage the Seelen Resources.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct ResourceManagerCli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
enum SubCommand {
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
}

impl ResourceManagerCli {
    pub fn process(self) -> Result<()> {
        match self.subcommand {
            SubCommand::Load { kind, path } => {
                let kind = kind.into();
                RESOURCES.load(&kind, &path)?;
                let _ = RESOURCES.manual.insert(path);
                RESOURCES.emit_kind_changed(&kind)?;
            }
            SubCommand::Unload { kind, path } => {
                let kind = kind.into();
                RESOURCES.unload(&kind, &path);
                RESOURCES.manual.remove(&path);
                RESOURCES.emit_kind_changed(&kind)?;
            }
            SubCommand::Bundle { kind, path } => {
                let mut to_store_path = path.clone();

                let format = time::macros::format_description!(
                    "[year]-[month]-[day] [hour]-[minute]-[second]"
                );
                let date =
                    time::OffsetDateTime::now_local().map_err(time::Error::IndeterminateOffset)?;
                let date_str = date.format(&format).map_err(time::Error::Format)?;
                let filename = format!("bundle {date_str}.yml");

                if to_store_path.is_dir() {
                    to_store_path.push(filename);
                } else {
                    to_store_path.set_file_name(filename);
                }

                match kind {
                    ClapResourceKind::Theme => {
                        let mut theme = Theme::load(&path)?;
                        theme.metadata.internal.path = to_store_path.clone();
                        theme.save()?
                    }
                    ClapResourceKind::Plugin => {
                        let mut plugin = Plugin::load(&path)?;
                        plugin.metadata.internal.path = to_store_path.clone();
                        plugin.save()?
                    }
                    ClapResourceKind::Widget => {
                        let mut widget = Widget::load(&path)?;
                        widget.metadata.internal.path = to_store_path.clone();
                        widget.save()?
                    }
                    ClapResourceKind::IconPack => {
                        let mut icon_pack = IconPack::load(&path)?;
                        icon_pack.metadata.internal.path = to_store_path.clone();
                        icon_pack.save()?
                    }
                    ClapResourceKind::Wallpaper => {
                        let mut wallpaper = Wallpaper::load(&path)?;
                        wallpaper.metadata.internal.path = to_store_path.clone();
                        wallpaper.save()?
                    }
                    _ => {
                        return Err("Not implemented".into());
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum ClapResourceKind {
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
