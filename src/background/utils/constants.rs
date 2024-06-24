use itertools::Itertools;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IGNORE_FOCUS_AND_FULLSCREEN: Vec<String> = vec![
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager", // For some reason th WM is focused on change of virtual desktop
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();

    /**
     * Some UWP apps like WhatsApp are resized after be opened,
     * this list will be used to resize them back after a delay.
     */
    pub static ref FORCE_RETILING_AFTER_ADD: Vec<String> = vec![
        "WhatsApp",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();
}
