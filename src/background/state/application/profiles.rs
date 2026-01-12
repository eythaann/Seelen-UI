use crate::error::Result;

use super::FullState;

impl FullState {
    pub(super) fn load_profiles(&mut self) -> Result<()> {
        /* let user_path = SEELEN_COMMON.user_profiles_path();
        for entry in std::fs::read_dir(user_path)?.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            match Profile::load(&path) {
                Ok(profile) => {
                    self.profiles.push(profile);
                }
                Err(e) => {
                    log::error!("Failed to load profile: {e}");
                }
            }
        } */
        Ok(())
    }
}
