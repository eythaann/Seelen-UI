use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use slu_utils::{debounce, Debounce};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::start::application::{StartMenuEvent, StartMenuManager},
    utils::{constants::SEELEN_COMMON, lock_free::SyncVec},
    windows_api::types::AppUserModelId,
};

use super::{_extract_and_save_icon_from_file, _extract_and_save_icon_umid};

/// File extensions that are per-file (each has its own unique icon).
/// Everything else is per-extension (all files of that type share the same shell icon).
const PER_FILE_EXTENSIONS: &[&str] = &["exe", "lnk", "url"];

/// Returns the real file path for Windows `path,index` icon notation (e.g. `"file.ico,0"` → `"file.ico"`).
/// For normal paths the input is returned unchanged.
fn real_path_for_ext(path: &Path) -> std::borrow::Cow<'_, std::path::Path> {
    if let Some(s) = path.to_str() {
        if let Some(comma) = s.rfind(',') {
            if s[comma + 1..].trim().parse::<i32>().is_ok() {
                return std::borrow::Cow::Owned(PathBuf::from(&s[..comma]));
            }
        }
    }
    std::borrow::Cow::Borrowed(path)
}

pub struct IconExtractor {
    failures: SyncVec<IconExtractorRequest>,
    save_failures: Debounce<()>,
}

/// Represents a request to extract an icon.
///
/// - `AppUMID`: extract by application user model ID.
/// - `Path`: extract from a specific file path (used for exe/lnk/url).
/// - `Extension`: marks an entire file extension as failed (e.g. `"js"` covers all `*.js` files).
///   This avoids storing hundreds of individual path failures for non-executable file types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconExtractorRequest {
    AppUMID(AppUserModelId),
    Path(PathBuf),
    /// Glob-like failure record: any file with this extension should be skipped.
    Extension(String),
}

event_manager!(IconExtractor, IconExtractorRequest);

impl IconExtractor {
    fn create() -> Self {
        let mut extractor = Self {
            failures: SyncVec::new(),
            save_failures: debounce(
                |_| {
                    let path = SEELEN_COMMON.app_cache_dir().join("icon_failures2.yml");
                    if let Ok(file) = std::fs::File::create(&path) {
                        serde_yaml::to_writer(file, &Self::instance().failures.to_vec())
                            .log_error();
                    }
                },
                std::time::Duration::from_secs(2),
            ),
        };

        extractor.init().log_error();

        Self::subscribe(|request| {
            let m = Self::instance();
            if m.is_failed(&request) {
                return;
            }

            if let Err(err) = Self::process(&request) {
                log::error!("Failed to extract icon: {err}");
                m.record_failure(request);
                m.save_failures.call(());
            }
        });

        StartMenuManager::subscribe(|event| {
            if matches!(event, StartMenuEvent::ItemsRefreshed) {
                Self::instance().revalidate_property_store_failures();
            }
        });

        extractor
    }

    pub fn init(&mut self) -> Result<()> {
        let cached = SEELEN_COMMON.app_cache_dir().join("icon_failures2.yml");
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

    /// Returns true if this request is already known to fail and should be skipped.
    fn is_failed(&self, request: &IconExtractorRequest) -> bool {
        if self.failures.contains(request) {
            return true;
        }
        // For generic file paths, also check if the whole extension has been marked as failed.
        if let IconExtractorRequest::Path(path) = request {
            let real = real_path_for_ext(path);
            if let Some(ext) = real.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if !PER_FILE_EXTENSIONS.contains(&ext_lower.as_str()) {
                    return self
                        .failures
                        .contains(&IconExtractorRequest::Extension(ext_lower));
                }
            }
        }
        false
    }

    /// Records a failure. For non-executable file types the whole extension is recorded
    /// as a glob pattern (`Extension`), collapsing any previously stored individual paths
    /// with that extension to keep the failure list compact.
    fn record_failure(&self, request: IconExtractorRequest) {
        if let IconExtractorRequest::Path(ref path) = request {
            let real = real_path_for_ext(path);
            if let Some(ext) = real.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if !PER_FILE_EXTENSIONS.contains(&ext_lower.as_str()) {
                    // Remove any individual path entries already stored for this extension.
                    self.failures.retain(|f| match f {
                        IconExtractorRequest::Path(p) => {
                            p.extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .as_deref()
                                != Some(&ext_lower)
                        }
                        _ => true,
                    });
                    let ext_entry = IconExtractorRequest::Extension(ext_lower);
                    if !self.failures.contains(&ext_entry) {
                        self.failures.push(ext_entry);
                    }
                    return;
                }
            }
        }
        self.failures.push(request);
    }

    /// Re-queues all `AppUMID(PropertyStore)` failures so they are retried after the
    /// start menu has been refreshed. The entries are removed from the failures list
    /// first so `is_failed` does not skip them immediately.
    fn revalidate_property_store_failures(&self) {
        let to_retry: Vec<IconExtractorRequest> = self
            .failures
            .to_vec()
            .into_iter()
            .filter(|r| matches!(r, IconExtractorRequest::AppUMID(id) if id.is_property_store()))
            .collect();

        if to_retry.is_empty() {
            return;
        }

        log::debug!(
            "Revalidating {} PropertyStore UMID icon failure(s) after start menu refresh",
            to_retry.len()
        );

        self.failures.retain(|r| !to_retry.contains(r));
        self.save_failures.call(());
        for request in to_retry {
            Self::send(request);
        }
    }

    fn process(request: &IconExtractorRequest) -> Result<()> {
        match request {
            IconExtractorRequest::AppUMID(umid) => {
                _extract_and_save_icon_umid(umid)?;
            }
            IconExtractorRequest::Path(path) => {
                _extract_and_save_icon_from_file(path)?;
            }
            IconExtractorRequest::Extension(_) => {
                // Extension entries are failure markers, not actionable requests.
            }
        }
        Ok(())
    }
}
