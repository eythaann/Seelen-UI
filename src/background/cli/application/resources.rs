use std::path::PathBuf;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::{error::Result, utils::constants::SEELEN_COMMON};

#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct WidgetCli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
enum SubCommand {
    /// load a widget folder into the resources files
    Load {
        path: PathBuf,
        /// the name of the widget
        #[arg(long)]
        r#as: String,
    },
    /// delete a widget from the resources files
    Remove { name: String },
}

impl WidgetCli {
    pub fn process(self) -> Result<()> {
        match self.subcommand {
            SubCommand::Load { path: origin, r#as } => {
                let foldername = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&r#as);
                let folder = SEELEN_COMMON.user_widgets_path().join(foldername);

                if folder.exists() {
                    std::fs::remove_dir_all(&folder)?;
                }

                std::fs::create_dir_all(&folder)?;
                let _ = std::fs::copy(origin.join("metadata.yml"), folder.join("metadata.yml"));
                let _ = std::fs::copy(origin.join("index.js"), folder.join("index.js"));
                let _ = std::fs::copy(origin.join("index.html"), folder.join("index.html"));
                let _ = std::fs::copy(origin.join("index.css"), folder.join("index.css"));
            }
            SubCommand::Remove { name } => {
                let foldername = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(name);
                let folder = SEELEN_COMMON.user_widgets_path().join(foldername);
                std::fs::remove_dir_all(folder)?;
            }
        }
        Ok(())
    }
}
