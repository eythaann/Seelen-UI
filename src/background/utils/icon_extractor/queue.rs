use std::{path::PathBuf, sync::LazyLock};

use serde::{Deserialize, Serialize};
use slu_utils::{debounce, Debounce};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::{constants::SEELEN_COMMON, lock_free::SyncVec},
    windows_api::types::AppUserModelId,
};

use super::{_extract_and_save_icon_from_file, _extract_and_save_icon_umid};

pub struct IconExtractor {
    failures: SyncVec<IconExtractorRequest>,
    save_failures: Debounce<()>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconExtractorRequest {
    AppUMID(AppUserModelId),
    Path(PathBuf),
}

event_manager!(IconExtractor, IconExtractorRequest);

impl IconExtractor {
    fn create() -> Self {
        let mut extractor = Self {
            failures: SyncVec::new(),
            save_failures: debounce(
                |_| {
                    let path = SEELEN_COMMON.app_cache_dir().join("icon_failures.yml");
                    if let Ok(file) = std::fs::File::create(&path) {
                        let _ = serde_yaml::to_writer(file, &Self::instance().failures.to_vec());
                    }
                },
                std::time::Duration::from_secs(2),
            ),
        };

        extractor.init().log_error();

        Self::subscribe(|request| {
            let m = Self::instance();
            if m.failures.contains(&request) {
                return;
            }

            if let Err(err) = Self::process(&request) {
                log::error!("Failed to extract icon: {err}");
                m.failures.push(request);
                m.save_failures.call(());
            }
        });
        extractor
    }

    pub fn init(&mut self) -> Result<()> {
        let cached = SEELEN_COMMON.app_cache_dir().join("icon_failures.yml");
        if cached.exists() {
            let buff = std::fs::read(cached)?;
            let failures = serde_yaml::from_slice::<Vec<IconExtractorRequest>>(&buff)?;
            self.failures = failures.into();
        }
        Ok(())
    }

    pub fn instance() -> &'static IconExtractor {
        static ICON_EXTRACTOR: LazyLock<IconExtractor> = LazyLock::new(IconExtractor::create);
        &ICON_EXTRACTOR
    }

    pub fn request(&self, request: IconExtractorRequest) {
        Self::send(request);
    }

    fn process(request: &IconExtractorRequest) -> Result<()> {
        match request {
            IconExtractorRequest::AppUMID(umid) => {
                _extract_and_save_icon_umid(umid)?;
            }
            IconExtractorRequest::Path(path) => {
                _extract_and_save_icon_from_file(path, None)?;
            }
        }
        Ok(())
    }
}
