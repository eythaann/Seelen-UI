pub fn is_windows_10() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if (&10240..&22000).contains(&x))
}

pub fn is_windows_11() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &22000)
}

/// this should be called before call any winvd function
pub fn is_virtual_desktop_supported() -> bool {
    // disable virtual desktop for 24h2
    // matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if (&10240..&26000).contains(&x))
    false
}
