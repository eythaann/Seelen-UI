pub mod apps;
pub mod clipboard;
pub mod focus_assist;
pub mod fonts;
pub mod media;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod radios;
pub mod start;
pub mod system;
pub mod system_settings;
pub mod system_tray;
pub mod trash_bin;
pub mod user;

#[macro_export]
macro_rules! event_manager {
    ($name:ident, $event:ty) => {
        static CHANNEL: std::sync::LazyLock<(
            crossbeam_channel::Sender<$event>,
            crossbeam_channel::Receiver<$event>,
        )> = std::sync::LazyLock::new(crossbeam_channel::unbounded);

        /// Copy-on-write list: the dispatcher takes a snapshot and runs the callbacks
        /// without holding any lock, so a callback blocking (e.g. waiting on a LazyLock
        /// being initialized) never prevents others from (un)subscribing.
        static SUBSCRIBERS: std::sync::LazyLock<
            arc_swap::ArcSwap<
                Vec<(
                    String,
                    std::sync::Arc<dyn Fn($event) + Sync + Send + 'static>,
                    u32,
                )>,
            >,
        > = std::sync::LazyLock::new(|| arc_swap::ArcSwap::from_pointee(Vec::new()));

        static THREAD_INIT: std::sync::Once = std::sync::Once::new();

        #[allow(dead_code)]
        impl $name {
            fn _init_thread() {
                THREAD_INIT.call_once(|| {
                    let rx = CHANNEL.1.clone();
                    std::thread::spawn(move || {
                        for event in rx {
                            let subscribers = SUBSCRIBERS.load_full();
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
                let callback: std::sync::Arc<dyn Fn($event) + Sync + Send + 'static> =
                    std::sync::Arc::new(callback);
                SUBSCRIBERS.rcu(|subscribers| {
                    let mut subscribers = Vec::clone(subscribers);
                    subscribers.push((id.clone(), callback.clone(), 0));
                    subscribers
                });
                id
            }

            pub fn set_event_handler_priority(id: &str, priority: u32) {
                SUBSCRIBERS.rcu(|subscribers| {
                    let mut subscribers = Vec::clone(subscribers);
                    if let Some(s) = subscribers.iter_mut().find(|s| s.0 == id) {
                        s.2 = priority;
                    }
                    // Higher priority subscribers will be called first
                    subscribers.sort_by(|a, b| b.2.cmp(&a.2));
                    subscribers
                });
            }

            pub fn unsubscribe(id: &str) {
                SUBSCRIBERS.rcu(|subscribers| {
                    let mut subscribers = Vec::clone(subscribers);
                    subscribers.retain(|(i, _, _)| i != id);
                    subscribers
                });
            }
        }
    };
}
