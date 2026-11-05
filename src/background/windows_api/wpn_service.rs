//! Read access to the database of the Windows Push Notifications User Service
//! (`WpnUserService_*`), where Windows stores the toast, badge and tile notifications of every app.
//!
//! The database is undocumented, but its schema is the same since Windows 10 1607.
//! It is in WAL mode: recent writes live in `wpndatabase.db-wal` until the next checkpoint,
//! SQLite applies them transparently as long as the original database is opened (never a copy
//! of the `.db` file alone).

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

use arc_swap::ArcSwap;
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use rusqlite::{Connection, OpenFlags, Row, types::ValueRef};
use seelen_core::system_state::{Badge, BadgeValue, Toast};
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{error::Result, event_manager, windows_api::WindowsApi};

const DATABASE_FILE_NAME: &str = "wpndatabase.db";

/// Files written by the service. `wpndatabase.db-shm` is excluded on purpose: our own reads
/// touch it, so watching it would make every reload trigger the next one.
const WATCHED_FILES: [&str; 2] = ["wpndatabase.db", "wpndatabase.db-wal"];

/// Offset between the FILETIME epoch (1601-01-01) and the unix epoch, in 100ns intervals.
const FILETIME_UNIX_EPOCH_DIFF: i64 = 116_444_736_000_000_000;

#[derive(Debug, Clone)]
pub enum WpnServiceEvent {
    /// The database was written and the notifications were reloaded,
    /// read them with [`WpnService::notifications`].
    NotificationsChanged,
}

event_manager!(WpnService, WpnServiceEvent);

pub struct WpnService {
    database_path: PathBuf,
    /// all the notifications stored in the database, expired ones included
    notifications: ArcSwap<Vec<WpnNotification>>,
    _watcher: Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
}

impl WpnService {
    pub fn instance() -> Result<&'static Self> {
        static SERVICE: LazyLock<Result<WpnService>> = LazyLock::new(WpnService::create);
        SERVICE.as_ref().map_err(|err| format!("{err:?}").into())
    }

    /// Snapshot of the notifications stored in the database, ordered by arrival time.
    /// Expired notifications are included, see [`WpnNotification::is_expired`].
    pub fn notifications(&self) -> Arc<Vec<WpnNotification>> {
        self.notifications.load_full()
    }

    /// Current badge of the app, None if it has no badge or it was cleared.
    pub fn get_badge_value(&self, umid: &str) -> Option<BadgeValue> {
        // an app has a single badge, the latest one wins
        self.notifications
            .load()
            .iter()
            .rev()
            .filter(|n| n.app_umid.eq_ignore_ascii_case(umid) && !n.is_expired())
            .find_map(|n| match &n.data {
                WpnNotificationData::Badge(badge) => Some(badge),
                _ => None,
            })
            .filter(|badge| !badge.is_cleared())
            .map(|badge| badge.value.clone())
    }

    fn create() -> Result<Self> {
        let database_path = WindowsApi::known_folder(FOLDERID_LocalAppData)?
            .join(r"Microsoft\Windows\Notifications")
            .join(DATABASE_FILE_NAME);

        if !database_path.exists() {
            return Err(format!("{} not found", database_path.display()).into());
        }

        let notifications = Self::read_notifications(&database_path)?;

        // reading still works without the watcher, only updates are lost
        let watcher = match Self::create_watcher(&database_path) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                log::error!("Failed to watch the notifications database: {err:?}");
                None
            }
        };

        Ok(Self {
            database_path,
            notifications: ArcSwap::from_pointee(notifications),
            _watcher: watcher,
        })
    }

    fn create_watcher(
        database_path: &Path,
    ) -> Result<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>> {
        let folder = database_path
            .parent()
            .ok_or("Invalid notifications database path")?;

        let mut debouncer = new_debouncer(
            Duration::from_millis(300),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    let database_changed = events.iter().any(|event| {
                        event.paths.iter().any(|path| {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| WATCHED_FILES.contains(&name))
                        })
                    });
                    if database_changed && let Ok(service) = Self::instance() {
                        service.reload();
                    }
                }
                Err(errors) => {
                    log::error!("Notifications database watcher error: {errors:?}");
                }
            },
        )?;

        debouncer.watch(folder, RecursiveMode::NonRecursive)?;
        Ok(debouncer)
    }

    /// Stores the new state before notifying, so subscribers read already parsed data.
    fn reload(&self) {
        match Self::read_notifications(&self.database_path) {
            Ok(notifications) => {
                self.notifications.store(Arc::new(notifications));
                Self::send(WpnServiceEvent::NotificationsChanged);
            }
            Err(err) => log::error!("Failed to read the notifications database: {err:?}"),
        }
    }

    /// Opens a new read only connection, it is cheap and avoids keeping the database open.
    fn connect(database_path: &Path) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            database_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // the service could be writing or checkpointing at the same time
        conn.busy_timeout(Duration::from_millis(500))?;
        Ok(conn)
    }

    fn read_notifications(database_path: &Path) -> Result<Vec<WpnNotification>> {
        let conn = Self::connect(database_path)?;
        let mut stmt = conn.prepare(
            r#"SELECT n.Id, n.HandlerId, h.PrimaryId, n.Type, n.Payload, n.PayloadType, n.Tag, n."Group",
                n.ExpiryTime, n.ArrivalTime, n.ExpiresOnReboot
            FROM Notification n
            JOIN NotificationHandler h ON h.RecordId = n.HandlerId
            ORDER BY n.ArrivalTime ASC"#,
        )?;

        let rows = stmt.query_map([], WpnNotification::from_row)?;
        let mut notifications = Vec::new();
        for row in rows {
            match row {
                Ok(notification) => notifications.push(notification),
                Err(err) => log::warn!("Skipping invalid notification row: {err}"),
            }
        }
        Ok(notifications)
    }
}

/// A row of the `Notification` table joined with its handler (the app).
#[derive(Debug, Clone)]
pub struct WpnNotification {
    pub id: i64,
    pub handler_id: i64,
    /// `NotificationHandler.PrimaryId`, the AppUserModelId of the app
    pub app_umid: String,
    pub data: WpnNotificationData,
    pub tag: Option<String>,
    pub group: Option<String>,
    /// unix timestamp (ms)
    pub arrival_time: Option<i64>,
    /// unix timestamp (ms), None if the notification never expires
    pub expiry_time: Option<i64>,
    pub expires_on_reboot: bool,
}

impl WpnNotification {
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        self.expiry_time.is_some_and(|expiry| expiry <= now)
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("Id")?,
            handler_id: row.get("HandlerId")?,
            app_umid: row.get("PrimaryId")?,
            data: WpnNotificationData::from_row(row)?,
            tag: row.get("Tag")?,
            group: row.get("Group")?,
            arrival_time: row
                .get::<_, Option<i64>>("ArrivalTime")?
                .filter(|t| *t > 0)
                .map(filetime_to_unix_ms),
            expiry_time: row
                .get::<_, Option<i64>>("ExpiryTime")?
                .filter(|t| *t > 0)
                .map(filetime_to_unix_ms),
            expires_on_reboot: read_bool(row, "ExpiresOnReboot")?,
        })
    }
}

/// Content of the notification, parsed according to its type.
#[derive(Debug, Clone)]
pub enum WpnNotificationData {
    Badge(Badge),
    Toast(Toast),
    /// unsupported types (e.g. tiles), or payloads that could not be parsed
    Other {
        /// value of the `Type` column
        kind: String,
        payload: Option<Vec<u8>>,
        /// format of the payload, e.g. `Xml`
        payload_type: String,
    },
}

impl WpnNotificationData {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let kind: String = row.get("Type")?;
        let payload: Option<Vec<u8>> = row.get("Payload")?;
        let payload_type: String = row.get("PayloadType")?;

        if payload_type.eq_ignore_ascii_case("xml")
            && let Some(bytes) = payload.as_deref()
        {
            let parsed = match kind.as_str() {
                "badge" => Some(quick_xml::de::from_reader(bytes).map(Self::Badge)),
                "toast" => Some(quick_xml::de::from_reader(bytes).map(Self::Toast)),
                _ => None,
            };
            match parsed {
                Some(Ok(data)) => return Ok(data),
                Some(Err(err)) => log::warn!("Failed to parse {kind} notification payload: {err}"),
                None => {}
            }
        }

        Ok(Self::Other {
            kind,
            payload,
            payload_type,
        })
    }
}

/// `BOOLEAN DEFAULT('FALSE')` columns can hold integers or the literal text `TRUE`/`FALSE`.
fn read_bool(row: &Row, column: &str) -> rusqlite::Result<bool> {
    Ok(match row.get_ref(column)? {
        ValueRef::Integer(value) => value != 0,
        ValueRef::Text(text) => text.eq_ignore_ascii_case(b"TRUE") || text == b"1",
        _ => false,
    })
}

fn filetime_to_unix_ms(filetime: i64) -> i64 {
    (filetime - FILETIME_UNIX_EPOCH_DIFF) / 10_000
}

#[cfg(test)]
mod tests {
    use seelen_core::system_state::BadgeGlyph;

    use super::*;

    fn badge(xml: &str) -> Badge {
        quick_xml::de::from_str(xml).unwrap()
    }

    #[test]
    fn numeric_badge() {
        assert_eq!(badge(r#"<badge value="19"/>"#).value, BadgeValue::Count(19));
    }

    #[test]
    fn glyph_badge() {
        assert_eq!(
            badge(r#"<badge value="newMessage"/>"#).value,
            BadgeValue::Glyph(BadgeGlyph::NewMessage)
        );
    }

    #[test]
    fn cleared_badge() {
        assert!(badge(r#"<badge value="none"/>"#).is_cleared());
        assert!(badge(r#"<badge value="0"/>"#).is_cleared());
    }

    #[test]
    fn badge_value_json_roundtrip() {
        for value in [
            BadgeValue::Count(19),
            BadgeValue::Glyph(BadgeGlyph::NewMessage),
        ] {
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(serde_json::from_str::<BadgeValue>(&json).unwrap(), value);
        }
    }

    #[test]
    fn filetime_conversion() {
        // ArrivalTime of the exported sample: 2026-09-29
        let ms = filetime_to_unix_ms(134_351_910_676_138_135);
        assert_eq!(ms, 1_790_717_467_613);
    }
}
