use seelen_core::system_state::ClipboardEntry;
use serde::{Deserialize, Serialize};
use windows::{
    Security::Cryptography::{CryptographicBuffer, DataProtection::DataProtectionProvider},
    core::HSTRING,
};

use crate::{error::Result, utils::constants::SEELEN_COMMON};

/// On-disk shape of our clipboard history store. Wrapped in a struct (instead
/// of persisting a bare `Vec<ClipboardEntry>`) so new top-level fields (like
/// `pinned_ids`) can be added later without changing the storage format.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClipboardStore {
    pub items: Vec<ClipboardEntry>,
    /// Ids of entries pinned by the user. Pinned entries are kept when the
    /// history is cleared via [`crate::modules::clipboard::application::ClipboardManager::clear_history`].
    pub pinned_ids: Vec<String>,
}

/// Descriptor scoping protected data to the current Windows user account,
/// on this machine only (equivalent to DPAPI's per-user scope).
const PROTECTION_DESCRIPTOR: &str = "LOCAL=user";

/// Encrypts `plaintext` so that only the current user, on this machine, can
/// decrypt it back (via [`decrypt`]). Must be called from an STA thread.
fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    let provider =
        DataProtectionProvider::CreateOverloadExplicit(&HSTRING::from(PROTECTION_DESCRIPTOR))?;
    let input = CryptographicBuffer::CreateFromByteArray(plaintext)?;
    let protected = provider.ProtectAsync(&input)?.join()?;

    let mut output = windows_core::Array::<u8>::new();
    CryptographicBuffer::CopyToByteArray(&protected, &mut output)?;
    Ok(output.to_vec())
}

/// Decrypts data previously produced by [`encrypt`]. Must be called from an
/// STA thread.
fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>> {
    let provider = DataProtectionProvider::new()?;
    let input = CryptographicBuffer::CreateFromByteArray(ciphertext)?;
    let plain = provider.UnprotectAsync(&input)?.join()?;

    let mut output = windows_core::Array::<u8>::new();
    CryptographicBuffer::CopyToByteArray(&plain, &mut output)?;
    Ok(output.to_vec())
}

/// Encrypts and writes the given store to disk. Must be called from an STA
/// thread (uses WinRT `DataProtectionProvider`).
pub fn save_store(store: &ClipboardStore) -> Result<()> {
    let json = serde_json::to_vec(store)?;
    let encrypted = encrypt(&json)?;
    std::fs::write(SEELEN_COMMON.clipboard_history_path(), encrypted)?;
    Ok(())
}

/// Reads and decrypts the stored data from disk. Returns an empty store if
/// there is no store yet, or if it fails to read/decrypt (corrupted file,
/// moved from another user/machine, etc). Must be called from an STA thread.
pub fn load_store() -> ClipboardStore {
    let path = SEELEN_COMMON.clipboard_history_path();
    if !path.exists() {
        return ClipboardStore::default();
    }

    let load = || -> Result<ClipboardStore> {
        let encrypted = std::fs::read(path)?;
        let json = decrypt(&encrypted)?;
        Ok(serde_json::from_slice(&json)?)
    };

    match load() {
        Ok(store) => store,
        Err(e) => {
            log::error!("Failed to load clipboard history store, starting empty: {e}");
            ClipboardStore::default()
        }
    }
}
