use serde::{Deserialize, Serialize};

use crate::{error_handler::Result, windows_api::WindowsApi};

#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct Win32Cli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
enum SubCommand {
    SetDefaultAudioDevice { id: String, role: String },
}

impl Win32Cli {
    pub fn process(&self) -> Result<()> {
        match &self.subcommand {
            SubCommand::SetDefaultAudioDevice { id, role } => {
                WindowsApi::set_default_audio_device(id, role)?;
            }
        };
        Ok(())
    }
}
