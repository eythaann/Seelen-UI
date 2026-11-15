use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

pub use slu_ipc::commands::WegCli;
use slu_ipc::commands::WegCommand;

use seelen_core::{
    state::{WegItem, WegItemData},
    system_state::UserAppWindow,
};

use crate::{
    error::{Result, ResultLogExt},
    platform::{
        shell,
        window::{Window, WindowState},
    },
    state::WEG_ITEMS_MANAGER,
    system_modules::apps::UserAppsManager,
};

static DOCK_REQUEST: AtomicUsize = AtomicUsize::new(0);

/// Mirrors `getWindowsForItem` from the frontend (`windows.ts`).
///
/// Grouping rules:
///   1. Window has a umid  → matched only by exact umid equality. Path is not used.
///      If no item has that umid, a new item will be created for it.
///   2. Window has no umid → matched by exact path (item.relaunch.command or item.path).
///
/// note: on update of this function check src\ui\react\weg\modules\shared\state\windows.ts both should work the same
fn get_windows_for_item<'a>(
    item: &WegItemData,
    interactables: &'a [UserAppWindow],
) -> Vec<&'a UserAppWindow> {
    let item_command = item.relaunch.as_ref().map(|r| r.command.to_lowercase());
    let item_path = item.path.to_string_lossy().to_lowercase();

    interactables
        .iter()
        .filter(|w| {
            if w.umid.is_some() {
                return item.umid == w.umid;
            }

            let win_path = w
                .process
                .path
                .as_ref()
                .map(|p| p.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if win_path.is_empty() {
                return false;
            }

            item_command.as_deref() == Some(win_path.as_str()) || item_path == win_path
        })
        .collect()
}

pub fn process(cmd: WegCli) -> Result<()> {
    #[allow(irrefutable_let_patterns)]
    if let WegCommand::ForegroundOrRunApp { index } = cmd.subcommand {
        let weg_items = WEG_ITEMS_MANAGER.get();

        let all_items: Vec<&WegItem> = weg_items
            .left
            .iter()
            .chain(weg_items.center.iter())
            .chain(weg_items.right.iter())
            .filter(|item| matches!(item, WegItem::AppOrFile(_)))
            .collect();

        if all_items.len() <= index {
            return Ok(());
        }

        let WegItem::AppOrFile(inner_data) = all_items[index] else {
            return Ok(());
        };

        let request = DOCK_REQUEST.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let interactables = UserAppsManager::instance().interactable_windows.to_vec();
        let windows = get_windows_for_item(inner_data, &interactables);

        if windows.is_empty() {
            let packaged_id = inner_data.umid.as_ref().filter(|id| {
                inner_data.relaunch.is_none()
                    && (inner_data.path.as_os_str().is_empty() || shell::app_executable(id).is_ok())
            });
            let command = inner_data
                .relaunch
                .as_ref()
                .map(|r| r.command.clone())
                .unwrap_or_else(|| match packaged_id {
                    Some(umid) => format!("shell:AppsFolder\\{umid}"),
                    None => inner_data.path.to_string_lossy().into_owned(),
                });
            let args = inner_data
                .relaunch
                .as_ref()
                .and_then(|r| r.args.as_ref())
                .map(|a| a.to_string());
            let working_dir = inner_data
                .relaunch
                .as_ref()
                .and_then(|r| r.working_dir.clone());
            let foreground = Window::get_foregrounded().address();
            shell::execute(&command, args.as_deref(), working_dir.as_deref(), false)?;
            if let Some(id) = packaged_id.cloned() {
                tokio::spawn(focus_launched_app(id, foreground, request));
            }
        } else {
            let focused = windows.iter().find(|w| Window::from(w.hwnd).is_focused());
            if let Some(w) = focused {
                Window::from(w.hwnd).set_state_async(WindowState::Minimized)?;
            } else if let Some(w) = windows.first() {
                let window = Window::from(w.hwnd);
                if window.is_window() {
                    window.unminimize()?;
                    window.focus()?;
                }
            }
        }
    }
    Ok(())
}

async fn focus_launched_app(id: String, foreground: isize, request: usize) {
    // Explorer launches asynchronously. Stop waiting if a newer shortcut or window takes over.
    let _ = tokio::time::timeout(Duration::from_millis(1500), async {
        let apps = &UserAppsManager::instance().interactable_windows;
        let still_current = || {
            DOCK_REQUEST.load(Ordering::Relaxed) == request && Window::from(foreground).is_focused()
        };
        while DOCK_REQUEST.load(Ordering::Relaxed) == request {
            let current = Window::get_foregrounded().address();
            if current != 0 && current != foreground {
                return;
            }

            if current != 0 {
                let mut candidates = Vec::new();
                apps.for_each(|app| {
                    if app.umid.as_deref() == Some(id.as_str()) {
                        candidates.push(Window::from(app.hwnd));
                    }
                });
                if let Some(window) = candidates.into_iter().find(|window| {
                    window.is_visible() && !window.is_minimized() && !window.is_cloaked()
                }) && still_current()
                {
                    log::trace!(
                        "Activating newly launched dock window: {}",
                        window.address()
                    );
                    window.focus().log_error();
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await;
}
