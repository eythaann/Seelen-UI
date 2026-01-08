pub mod apps;
pub mod input;
pub mod media;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod radios;
pub mod start;
pub mod system_settings;
pub mod system_tray;
pub mod user;

#[macro_export]
macro_rules! event_manager {
    ($name:ident, $event:ty) => {
        static CHANNEL: std::sync::LazyLock<(
            crossbeam_channel::Sender<$event>,
            crossbeam_channel::Receiver<$event>,
        )> = std::sync::LazyLock::new(crossbeam_channel::unbounded);

        static SUBSCRIBERS: std::sync::LazyLock<
            parking_lot::RwLock<Vec<(String, Box<dyn Fn($event) + Sync + Send + 'static>, u32)>>,
        > = std::sync::LazyLock::new(|| parking_lot::RwLock::new(Vec::new()));

        static THREAD_INIT: std::sync::Once = std::sync::Once::new();

        #[allow(dead_code)]
        impl $name {
            fn _init_thread() {
                THREAD_INIT.call_once(|| {
                    let rx = CHANNEL.1.clone();
                    std::thread::spawn(move || {
                        for event in rx {
                            let subscribers = SUBSCRIBERS.read();
                            for (_id, callback, _) in subscribers.iter() {
                                callback(event.clone());
                            }
                        }
                    });
                });
            }

            pub fn event_tx() -> crossbeam_channel::Sender<$event> {
                Self::_init_thread();
                CHANNEL.0.clone()
            }

            pub fn send(event: $event) {
                Self::_init_thread();
                if let Err(e) = CHANNEL.0.send(event) {
                    log::error!("Failed to send event: {e}");
                }
            }

            pub fn subscribe<F>(callback: F) -> String
            where
                F: Fn($event) + Sync + Send + 'static,
            {
                let id = uuid::Uuid::new_v4().to_string();
                SUBSCRIBERS
                    .write()
                    .push((id.clone(), Box::new(callback), 0));
                id
            }

            pub fn set_event_handler_priority(id: &str, priority: u32) {
                let mut subscribers = SUBSCRIBERS.write();
                for s in subscribers.iter_mut() {
                    if s.0 == id {
                        s.2 = priority;
                        break;
                    }
                }
                // Higher priority subscribers will be called first
                subscribers.sort_by(|a, b| b.2.cmp(&a.2));
            }

            pub fn unsubscribe(id: &str) {
                let mut subscribers = SUBSCRIBERS.write();
                subscribers.retain(|(i, _, _)| i != id);
            }
        }
    };
}
