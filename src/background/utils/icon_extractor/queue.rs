use std::{path::PathBuf, sync::LazyLock};

use serde::{Deserialize, Serialize};
use slu_utils::{debounce, Debounce};

use crate::{
    error::Result,
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
                    let path = SEELEN_COMMON.app_cache_dir().join("icon_failures");
                    if let Ok(file) = std::fs::File::create(&path) {
                        let _ = serde_yaml::to_writer(file, &Self::instance().failures.to_vec());
                    }
                },
                std::time::Duration::from_secs(2),
            ),
        };

        let cached = SEELEN_COMMON.app_cache_dir().join("icon_failures");
        if cached.exists() {
            if let Ok(buff) = std::fs::read(cached) {
                extractor.failures = serde_yaml::from_slice::<Vec<IconExtractorRequest>>(&buff)
                    .unwrap_or_default()
                    .into();
            }
        }

        Self::subscribe(|request| {
            if let Err(err) = Self::process(&request) {
                log::error!("Failed to extract icon: {err}");
                Self::instance().failures.push(request);
                Self::instance().save_failures.call(());
            }
        });
        extractor
    }

    pub fn instance() -> &'static IconExtractor {
        static ICON_EXTRACTOR: LazyLock<IconExtractor> = LazyLock::new(IconExtractor::create);
        &ICON_EXTRACTOR
    }

    pub fn request(request: IconExtractorRequest) {
        if Self::instance().failures.contains(&request) {
            return;
        }
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
