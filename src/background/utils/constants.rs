use itertools::Itertools;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IGNORE_FOCUS: Vec<String> = [
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager",
        "SeelenWeg",
        "SeelenWeg Hitbox",
        "Seelen Fancy Toolbar",
        "Seelen Fancy Toolbar Hitbox"
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();

    pub static ref IGNORE_FULLSCREEN: Vec<String> = [
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager",
        "Seelen Fancy Toolbar",
        "SeelenWeg"
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
