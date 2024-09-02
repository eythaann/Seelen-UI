use itertools::Itertools;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IGNORE_FOCUS: Vec<String> = [
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager", // for some reason this sometimes is focused, maybe could be deleted
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();

    pub static ref IGNORE_FULLSCREEN: Vec<String> = [
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();

    /**
     * Some UWP apps like WhatsApp are resized after be opened,
     * this list will be used to resize them back after a delay.
     */
    pub static ref FORCE_RETILING_AFTER_ADD: Vec<String> = ["WhatsApp"]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();
}

pub static OVERLAP_BLACK_LIST_BY_TITLE: [&str; 2] = ["", "Program Manager"];

pub static OVERLAP_BLACK_LIST_BY_EXE: [&str; 4] = [
    "msedgewebview2.exe",
    "SearchHost.exe",
    "StartMenuExperienceHost.exe",
    "ShellExperienceHost.exe",
];
