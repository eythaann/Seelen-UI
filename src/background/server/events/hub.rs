use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};

use parking_lot::{Mutex, RwLock};
use seelen_core::handlers::SeelenEvent;
use tokio::sync::Notify;

use super::protocol::event_frame;

/// Upper bound of undelivered frames per connection. Snapshot events take at most one slot each,
/// so this only caps queued events of a webview that stopped reading.
const MAX_PENDING_FRAMES: usize = 256;

pub static EVENTS_HUB: LazyLock<EventsHub> = LazyLock::new(EventsHub::default);

/// Routes global events to the websocket connections subscribed to them.
///
/// Lock order: `connections` > `Connection::subscriptions` > `listeners` | `Connection::pending`.
#[derive(Default)]
pub struct EventsHub {
    /// event -> number of connections subscribed to it
    listeners: RwLock<HashMap<String, usize>>,
    /// webview raw label -> its active connection
    connections: RwLock<HashMap<String, Arc<Connection>>>,
}

impl EventsHub {
    pub fn has_listeners(&self, event: &str) -> bool {
        self.listeners
            .read()
            .get(event)
            .is_some_and(|count| *count > 0)
    }

    /// Hands the already serialized payload to every subscribed connection.
    pub fn publish(&self, event: &str, payload_json: &str) {
        if !self.has_listeners(event) {
            return;
        }

        let frame = match event_frame(event, payload_json) {
            Ok(frame) => Arc::<str>::from(frame),
            Err(err) => {
                log::error!("Failed to serialize event {event}: {err}");
                return;
            }
        };

        let coalesce = !SeelenEvent::is_queued(event);
        for connection in self.connections.read().values() {
            connection.push(event, &frame, coalesce);
        }
    }

    /// Registers a new connection for the webview, closing the previous one if any
    /// (e.g. the webview was reloaded before the old socket was dropped).
    pub fn connect(&self, label: &str) -> Arc<Connection> {
        let connection = Arc::new(Connection::default());
        let previous = self
            .connections
            .write()
            .insert(label.to_owned(), connection.clone());

        if let Some(previous) = previous {
            self.release(&previous);
            previous.close();
        }
        connection
    }

    pub fn disconnect(&self, label: &str, connection: &Arc<Connection>) {
        {
            let mut connections = self.connections.write();
            // the connection could have been already replaced by a newer one
            if connections
                .get(label)
                .is_some_and(|current| Arc::ptr_eq(current, connection))
            {
                connections.remove(label);
            }
        }
        self.release(connection);
    }

    /// Returns false if the event doesn't exist.
    pub fn subscribe(&self, connection: &Connection, event: &str) -> bool {
        if !SeelenEvent::exists(event) {
            return false;
        }
        if connection.subscriptions.lock().insert(event.to_owned()) {
            *self.listeners.write().entry(event.to_owned()).or_default() += 1;
        }
        true
    }

    pub fn unsubscribe(&self, connection: &Connection, event: &str) {
        if connection.subscriptions.lock().remove(event) {
            self.decrement_listeners([event]);
        }
    }

    /// Drops all the subscriptions of the connection. Idempotent.
    fn release(&self, connection: &Connection) {
        let subscriptions = std::mem::take(&mut *connection.subscriptions.lock());
        self.decrement_listeners(subscriptions.iter().map(String::as_str));
    }

    fn decrement_listeners<'a>(&self, events: impl IntoIterator<Item = &'a str>) {
        let mut listeners = self.listeners.write();
        for event in events {
            if let Some(count) = listeners.get_mut(event) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    listeners.remove(event);
                }
            }
        }
    }
}

/// State of a single websocket connection, shared between the hub (producer)
/// and the connection task (consumer).
#[derive(Default)]
pub struct Connection {
    subscriptions: Mutex<HashSet<String>>,
    pending: Mutex<VecDeque<PendingFrame>>,
    notify: Notify,
    closed: AtomicBool,
}

impl Connection {
    /// Waits until there are frames to send or the connection was closed by the hub.
    pub async fn notified(&self) {
        self.notify.notified().await
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    /// Takes all the frames pending to be sent, in emission order.
    pub fn drain(&self) -> VecDeque<PendingFrame> {
        std::mem::take(&mut *self.pending.lock())
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.notify.notify_one();
    }

    fn push(&self, event: &str, frame: &Arc<str>, coalesce: bool) {
        if !self.subscriptions.lock().contains(event) {
            return;
        }

        {
            let mut pending = self.pending.lock();
            if coalesce {
                // latest wins: the previous undelivered snapshot is obsolete
                pending.retain(|p| p.event != event);
            }
            if pending.len() >= MAX_PENDING_FRAMES {
                log::warn!("Events connection is not reading, dropping oldest pending event");
                pending.pop_front();
            }
            pending.push_back(PendingFrame {
                event: event.to_owned(),
                frame: frame.clone(),
            });
        }
        // stores a permit if the task is busy sending, so no wake up is lost
        self.notify.notify_one();
    }
}

pub struct PendingFrame {
    event: String,
    pub frame: Arc<str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(connection: &Connection) -> Vec<String> {
        connection
            .drain()
            .into_iter()
            .map(|p| p.frame.to_string())
            .collect()
    }

    #[test]
    fn publish_without_listeners_does_nothing() {
        let hub = EventsHub::default();
        let connection = hub.connect("hub-test");
        hub.publish(SeelenEvent::PowerMode, "1");
        assert!(frames(&connection).is_empty());
        assert!(!hub.has_listeners(SeelenEvent::PowerMode));
    }

    #[test]
    fn unknown_events_are_rejected() {
        let hub = EventsHub::default();
        let connection = hub.connect("hub-test");
        assert!(!hub.subscribe(&connection, "not-an-event"));
        assert!(!hub.has_listeners("not-an-event"));
    }

    #[test]
    fn snapshots_are_coalesced_and_queued_are_not() {
        let hub = EventsHub::default();
        let connection = hub.connect("hub-test");
        hub.subscribe(&connection, SeelenEvent::PowerMode);
        hub.subscribe(&connection, SeelenEvent::WegAddItem);

        hub.publish(SeelenEvent::PowerMode, "1");
        hub.publish(SeelenEvent::WegAddItem, r#""a""#);
        hub.publish(SeelenEvent::PowerMode, "2");
        hub.publish(SeelenEvent::WegAddItem, r#""b""#);

        let frames = frames(&connection);
        assert_eq!(frames.len(), 3);
        assert!(frames[0].contains(r#""payload":"a""#));
        assert!(frames[1].contains(r#""payload":2"#));
        assert!(frames[2].contains(r#""payload":"b""#));
    }

    #[test]
    fn listeners_are_counted_per_connection() {
        let hub = EventsHub::default();
        let a = hub.connect("hub-test-a");
        let b = hub.connect("hub-test-b");

        hub.subscribe(&a, SeelenEvent::PowerMode);
        hub.subscribe(&a, SeelenEvent::PowerMode);
        hub.subscribe(&b, SeelenEvent::PowerMode);

        hub.unsubscribe(&a, SeelenEvent::PowerMode);
        assert!(hub.has_listeners(SeelenEvent::PowerMode));
        hub.disconnect("hub-test-b", &b);
        assert!(!hub.has_listeners(SeelenEvent::PowerMode));
    }

    #[test]
    fn replaced_connection_does_not_remove_the_new_one() {
        let hub = EventsHub::default();
        let old = hub.connect("hub-test");
        hub.subscribe(&old, SeelenEvent::PowerMode);

        let new = hub.connect("hub-test");
        assert!(old.is_closed());
        assert!(!hub.has_listeners(SeelenEvent::PowerMode));

        hub.subscribe(&new, SeelenEvent::PowerMode);
        hub.disconnect("hub-test", &old);
        assert!(hub.has_listeners(SeelenEvent::PowerMode));

        hub.publish(SeelenEvent::PowerMode, "1");
        assert_eq!(frames(&new).len(), 1);
    }
}
