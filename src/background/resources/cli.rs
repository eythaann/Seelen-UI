pub use slu_ipc::commands::ResourceManagerCli;
use slu_ipc::commands::ResourceSubCommand;

use crate::{error::Result, resources::RESOURCES};

pub fn process(cmd: ResourceManagerCli) -> Result<()> {
    match cmd.subcommand {
        ResourceSubCommand::Load { kind, path } => {
            let kind = kind.into();
            RESOURCES.load(&kind, &path)?;
            let _ = RESOURCES.manual.insert(path);
            RESOURCES.emit_kind_changed(&kind)?;
        }
        ResourceSubCommand::Unload { kind, path } => {
            let kind = kind.into();
            RESOURCES.unload(&kind, &path);
            RESOURCES.manual.remove(&path);
            RESOURCES.emit_kind_changed(&kind)?;
        }
        _ => {
            return Err("This command should be executed directly in console".into());
        }
    }
    Ok(())
}
