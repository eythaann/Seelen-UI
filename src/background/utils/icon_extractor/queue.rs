use std::{path::PathBuf, sync::LazyLock};

use crate::{error::Result, event_manager, log_error, windows_api::types::AppUserModelId};

use super::{_extract_and_save_icon_from_file, _extract_and_save_icon_umid};

pub static ICON_EXTRACTOR: LazyLock<IconExtractor> = LazyLock::new(IconExtractor::new);

pub struct IconExtractor {}

#[derive(Debug, Clone)]
pub enum IconExtractorRequest {
    AppUMID(AppUserModelId),
    Path(PathBuf),
}

event_manager!(IconExtractor, IconExtractorRequest);

impl IconExtractor {
    fn new() -> Self {
        let extractor = Self {};
        Self::subscribe(|request| {
            log_error!(Self::process(request));
        });
        extractor
    }

    pub fn request(request: IconExtractorRequest) {
        let _ = &*ICON_EXTRACTOR;
        log_error!(Self::event_tx().send(request));
    }

    fn process(request: IconExtractorRequest) -> Result<()> {
        match request {
            IconExtractorRequest::AppUMID(umid) => {
                _extract_and_save_icon_umid(&umid)?;
            }
            IconExtractorRequest::Path(path) => {
                _extract_and_save_icon_from_file(&path, None)?;
            }
        }
        Ok(())
    }
}
