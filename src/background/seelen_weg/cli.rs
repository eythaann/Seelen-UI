use clap::Command;

use crate::{
    error_handler::Result,
    get_subcommands,
    windows_api::{monitor::Monitor, WindowsApi},
};

use super::SeelenWeg;

get_subcommands![
    /** Open Dev Tools (only works if the app is running in dev mode) */
    Debug,
    /** Set foreground to the application which is idx-nth on the weg. If it is not started, then starts it. */
    ForegroundOrRunApp(idx: usize => "Which index should be started on weg."),
];

impl SeelenWeg {
    pub const CLI_IDENTIFIER: &'static str = "weg";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Seelen's Weg")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn process(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
            SubCommand::Debug => {
                #[cfg(any(debug_assertions, feature = "devtools"))]
                self.window.open_devtools();
            }
            SubCommand::ForegroundOrRunApp(_) => {
                if Monitor::from(WindowsApi::monitor_from_window(self.window.hwnd()?)).index()? == 0
                {
                    // self.emit(SeelenEvent::WegFocusedAppByIndex, idx)?;
                }
            }
        };
        Ok(())
    }
}
