use itertools::Itertools;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IGNORE_FOCUS: Vec<String> = vec![
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager", // For some reason th WM is focused on change of virtual desktop
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();
}
