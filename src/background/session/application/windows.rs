use windows::{
    Security::Credentials::{PasswordCredential, PasswordVault},
    core::HSTRING,
};

use crate::{
    error::{Result, ResultLogExt},
    session::domain::CredentialStore,
};

/// Windows Credential Manager (`PasswordVault`).
pub struct WindowsCredentials;

impl WindowsCredentials {
    fn open_vault() -> Result<PasswordVault> {
        PasswordVault::new().map_err(|e| {
            format!(
                "Windows Credential Manager is unavailable (VaultSvc may be stopped or disabled). \
                 Enable the 'Credential Manager' service and try again. \
                 Original error: {e}"
            )
            .into()
        })
    }
}

impl CredentialStore for WindowsCredentials {
    fn store(resource: &str, key: &str, value: &str) -> Result<()> {
        let resource = HSTRING::from(resource);
        let user_name = HSTRING::from(key);
        let password = HSTRING::from(value);
        let vault = Self::open_vault()?;
        // Remove stale entry first so Add always succeeds.
        if let Ok(existing) = vault.Retrieve(&resource, &user_name) {
            vault.Remove(&existing).ok();
        }
        let cred = PasswordCredential::CreatePasswordCredential(&resource, &user_name, &password)?;
        vault.Add(&cred)?;
        Ok(())
    }

    fn read(resource: &str, key: &str) -> Result<String> {
        let resource = HSTRING::from(resource);
        let user_name = HSTRING::from(key);
        let vault = Self::open_vault()?;
        let cred = vault.Retrieve(&resource, &user_name)?;
        cred.RetrievePassword()?;
        Ok(cred.Password()?.to_string())
    }

    fn delete(resource: &str, key: &str) -> Result<()> {
        let vault = Self::open_vault()?;
        if let Ok(cred) = vault.Retrieve(&HSTRING::from(resource), &HSTRING::from(key)) {
            vault.Remove(&cred).log_error();
        }
        Ok(())
    }
}
