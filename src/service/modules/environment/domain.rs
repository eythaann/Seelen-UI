use crate::error::Result;

/// OS contract of the environment module.
pub trait EnvironmentSystem: Sized + Send + Sync {
    /// Separator of the entries of the `PATH` variable.
    const PATH_SEPARATOR: char;

    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Whether the installation is already reachable through the `PATH` by OS means (e.g. an
    /// MSIX app execution alias), so the system `PATH` must not be touched.
    fn is_path_managed_by_os(&self) -> bool;

    /// Raw value of the system wide (all users) `PATH` variable.
    fn system_path(&self) -> Result<String>;

    /// Replaces the system wide (all users) `PATH` variable.
    fn set_system_path(&self, value: &str) -> Result<()>;
}

/// `paths` with `entry` appended, `None` if it already contains it.
pub fn path_with_entry(paths: &str, separator: char, entry: &str) -> Option<String> {
    let mut entries: Vec<&str> = paths.split(separator).collect();
    if entries.contains(&entry) {
        return None;
    }
    entries.push(entry);
    Some(entries.join(&separator.to_string()))
}

/// `paths` without `entry`, `None` if it doesn't contain it.
pub fn path_without_entry(paths: &str, separator: char, entry: &str) -> Option<String> {
    let entries: Vec<&str> = paths.split(separator).collect();
    if !entries.contains(&entry) {
        return None;
    }
    let entries: Vec<&str> = entries.into_iter().filter(|e| *e != entry).collect();
    Some(entries.join(&separator.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_entry_only_once() {
        assert_eq!(
            path_with_entry(r"C:\a;C:\b", ';', r"C:\slu"),
            Some(r"C:\a;C:\b;C:\slu".to_owned())
        );
        assert_eq!(path_with_entry(r"C:\a;C:\slu", ';', r"C:\slu"), None);
    }

    #[test]
    fn removes_every_occurrence() {
        assert_eq!(
            path_without_entry(r"C:\slu;C:\a;C:\slu", ';', r"C:\slu"),
            Some(r"C:\a".to_owned())
        );
        assert_eq!(path_without_entry(r"C:\a", ';', r"C:\slu"), None);
    }
}
