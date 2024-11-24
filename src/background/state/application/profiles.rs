use seelen_core::state::Profile;

use crate::error_handler::Result;

use super::FullState;

impl FullState {
    pub(super) fn load_profiles(&mut self) -> Result<()> {
        let user_path = self.data_dir.join("profiles");
        for entry in std::fs::read_dir(&user_path)?.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            match Profile::load(&path) {
                Ok(profile) => {
                    self.profiles.push(profile);
                }
                Err(e) => {
                    log::error!("Failed to load profile: {}", e);
                }
            }
        }
        Ok(())
    }
}
