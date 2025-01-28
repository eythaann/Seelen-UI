use std::ops::Index;

use clap::Command;
use regex::Regex;
use seelen_core::state::WegItem;
use tauri_plugin_shell::ShellExt;
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

use crate::{
    error_handler::Result,
    get_subcommands,
    seelen::get_app_handle,
    seelen_weg::weg_items_impl::WEG_ITEMS_IMPL,
    trace_lock,
    windows_api::{monitor::Monitor, window::Window, WindowsApi},
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

    pub fn process(matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        if let SubCommand::ForegroundOrRunApp(index) = subcommand {
            let id = Monitor::from(WindowsApi::monitor_from_cursor_point()).device_id()?;
            //This shift the user awaited number to the index (when you press 1 you want to open the 0 index, if you press 0, then it is then 10. item which 9 in index)
            let idx = if index == 0 { 9 } else { index - 1 };

            let items = trace_lock!(WEG_ITEMS_IMPL).get_filtered_by_monitor()?;
            if let Some(wegitems) = items.get(&id) {
                let all_items: Vec<&WegItem> = wegitems
                    .left
                    .iter()
                    .chain(wegitems.center.iter())
                    .chain(wegitems.right.iter())
                    .filter(|item| matches!(item, WegItem::Pinned(_) | WegItem::Temporal(_)))
                    .collect();

                if all_items.len() <= idx {
                    return Ok(());
                }

                let item = all_items.index(idx);

                if let WegItem::Pinned(inner_data) | WegItem::Temporal(inner_data) = item {
                    if let Some(item) = inner_data.windows.first() {
                        let window = Window::from(item.handle);
                        if !window.is_window() {
                            Self::remove_hwnd(&window)?;
                            return Ok(());
                        }
                        if window.is_focused() {
                            window.show_window_async(SW_MINIMIZE)?;
                        } else {
                            window.focus()?;
                        }
                    } else {
                        // TODO: move all this block to a reuseable function
                        let regex = Regex::new("\"(.*?)\"|'(.*?)'|(\\S+)").unwrap();
                        let mut matches = vec![];
                        for (_, [finds]) in regex
                            .captures_iter(&inner_data.relaunch_command)
                            .map(|c| c.extract())
                        {
                            matches.push(finds);
                        }

                        let special_regex = Regex::new("/^[a-zA-Z]:\\/").unwrap();
                        let mut program = "".to_owned();
                        if matches.len() > 1 && special_regex.is_match(matches[0]) {
                            for item in matches.clone() {
                                if !item.starts_with("-") && !item.contains("=") {
                                    program.push_str(item);
                                    _ = matches.pop();
                                } else {
                                    break;
                                }
                            }
                        } else {
                            program = matches.remove(0).to_owned();
                        }

                        // we create a link file to trick with explorer into a separated process
                        // and without elevation in case Seelen UI was running as admin
                        // this could take some delay like is creating a file but just are some milliseconds
                        // and this exposed funtion is intended to just run certain times
                        let lnk_file =
                            WindowsApi::create_temp_shortcut(&program, &matches.join(" "))?;
                        let path = lnk_file.clone();
                        tauri::async_runtime::block_on(async move {
                            get_app_handle()
                                .shell()
                                .command("explorer")
                                .arg(&path)
                                .status()
                                .await
                                .unwrap();
                        });
                        std::fs::remove_file(&lnk_file)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn process_by_instance(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        if let SubCommand::Debug = subcommand {
            #[cfg(any(debug_assertions, feature = "devtools"))]
            self.window.open_devtools();
        };
        Ok(())
    }
}
