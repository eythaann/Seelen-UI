use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use seelen_core::system_state::FolderType;

use crate::error::Result;

/// OS contract of the user folders module.
pub trait UserFoldersSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts watching the given folders, reporting their changes through `on_event`.
    /// Watching is best effort: a folder that can't be watched is logged and skipped.
    /// Called once; the watchers live as long as `self`.
    fn init<F>(&mut self, folders: &[(FolderType, PathBuf)], on_event: F) -> Result<()>
    where
        F: Fn(UserFoldersEvent) + Send + Sync + 'static;

    /// Where the folder lives on this system, if it has one.
    fn folder_path(&self, folder: FolderType) -> Option<PathBuf>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserFoldersEvent {
    FolderChanged { folder: FolderType },
}

/// How deep a folder is scanned: Recent is a flat folder of shortcuts, others are scanned up to
/// 5 levels deep to avoid exploding on large/deep folder trees.
pub fn max_scan_depth(folder: FolderType) -> usize {
    match folder {
        FolderType::Recent => 1,
        _ => 5,
    }
}

/// Returns true if the entry should be excluded from the file list.
/// When applied to a directory, returning true prunes its entire subtree.
pub fn is_ignored_entry(path: &Path) -> bool {
    static IGNORED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
        HashSet::from([
            // Dev artifact directories
            "node_modules",
            "target",
            "dist",
            "build",
            "out",
            "__pycache__",
            "venv",
            ".venv",
            "env",
            "vendor",
            // VCS
            ".git",
            ".svn",
            ".hg",
            // IDE/editor
            ".idea",
            ".vscode",
            ".next",
            // Cache
            ".cache",
            "cache",
            "Cache",
            // Windows system
            "$RECYCLE.BIN",
            "System Volume Information",
            "WindowsApps",
            "MicrosoftEdgeBackups",
        ])
    });

    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| IGNORED.contains(name))
}

pub fn is_ignored_file(path: &Path) -> bool {
    static IGNORED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
        HashSet::from([
            // Temp / backup
            "ini",
            "dat",
            "bak",
            "tmp",
            "temp",
            "old",
            "swp",
            "save",
            // Crash / memory dumps
            "dmp",
            "blk",
            // In-progress downloads
            "download",
            "crdownload",
            "part",
            // Lock / pid files
            "lock",
            "pid",
            // Log files
            "log",
            // Database / cache files
            "db",
            "sqlite",
            "sqlite3",
            "ldb",
            // Build artifacts
            "obj",
            "pdb",
            "ilk",
            "exp",
            "iobj",
            "ipdb",
            // Windows shortcut noise (internet shortcuts, not file shortcuts)
            "url",
            // System / driver files
            "dll",
            "sys",
            "lib",
            "cat",
            "inf",
            "winmd",
            // Certificates & keys
            "pfx",
            "pem",
            "crl",
            "p7b",
            // Checksums
            "sha512",
            "sha256",
            "sha1",
            "md5",
            // Game / engine packages
            "pak",
            "pck",
            "vdf",
            // iTunes library metadata
            "itdb",
            "itl",
            // Compiled shader cache
            "fxo",
            // Numbered / no-semantic extension (autotools, Python dist)
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            // Obscure image formats (raw scientific / legacy)
            "pbm",
            "pgm",
            "ppm",
            "ras",
            "sgi",
            "xbm",
            "xpm",
            // Obscure / legacy audio formats
            "8svx",
            "hcom",
            "sndt",
            "voc",
            "spx",
            "aifc",
        ])
    });

    let Some(extension) = path.extension() else {
        return true; // filter files without extension
    };

    let ext = extension.to_string_lossy().to_lowercase();

    // Backup files can have numbers in their name (e.g. 1, 2, 3, etc.)
    if ext.starts_with("bak") || ext.starts_with("backup") {
        return true;
    }

    IGNORED.contains(ext.as_str())
}
