mod art;
mod resources;

use clap::Parser;
use slu_ipc::{
    commands::{AppCli, AppCommand, CommandExecutionMode, SluCliCommand},
    messages::AppMessage,
    AppIpc,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    let cli = AppCli::parse();

    if cli.verbose {
        println!("Received args: {:#?}", std::env::args().collect::<Vec<_>>());
        println!("Parsed CLI: {cli:#?}");
    }

    if let Err(err) = run(cli).await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run(cli: AppCli) -> Result<()> {
    let mode = cli.command.execution_mode();
    match mode {
        CommandExecutionMode::Direct => process_direct(cli).await,
        CommandExecutionMode::MainInstance => send_to_main_instance(cli).await,
    }
}

async fn process_direct(cli: AppCli) -> Result<()> {
    match cli.command {
        AppCommand::Art(cmd) => art::process(cmd),
        AppCommand::Resource(cmd) => resources::process(cmd).await?,
        _ => return Err("Command does not support direct execution".into()),
    }
    Ok(())
}

async fn send_to_main_instance(cli: AppCli) -> Result<()> {
    let working_dir = std::env::current_dir()?;
    let args: Vec<String> = std::env::args()
        .map(|arg| {
            if arg.starts_with("./")
                || arg.starts_with(".\\")
                || arg.starts_with("../")
                || arg.starts_with("..\\")
            {
                working_dir.join(&arg).to_string_lossy().to_string()
            } else {
                arg
            }
        })
        .collect();

    if cli.verbose {
        println!("Sending {args:#?}");
    }

    AppIpc::send(AppMessage::Cli(args))
        .await
        .map_err(|_| "Can't establish connection, ensure Seelen UI is running.")?;
    Ok(())
}
