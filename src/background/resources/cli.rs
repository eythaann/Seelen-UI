pub use slu_ipc::commands::ResourceManagerCli;
use slu_ipc::commands::ResourceSubCommand;

use crate::{error::Result, resources::RESOURCES};

pub async fn process(cmd: ResourceManagerCli) -> Result<()> {
    match cmd.subcommand {
        ResourceSubCommand::Load { kind, path } => {
            let kind = kind.into();
            let loaded_id = RESOURCES.load(&kind, &path).await?;
            let _ = RESOURCES.manual.insert(path);
            // emit the updated resource list before enabling it, so the frontend
            // already knows about the resource by the time it receives the enable event
            RESOURCES.emit_kind_changed(&kind)?;
            if let Some(id) = loaded_id {
                RESOURCES.enable_resource(kind, id);
            }
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
