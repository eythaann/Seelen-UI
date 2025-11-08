pub mod apps;
pub mod bluetooth;
pub mod input;
pub mod language;
pub mod media;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod shared;
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
        )> = std::sync::LazyLock::new(|| {
            std::thread::spawn(move || {
                let rx = CHANNEL.1.clone();
                for event in rx {
                    SUBSCRIBERS.scan(|_k, v| {
                        v(event.clone());
                    })
                }
            });
            crossbeam_channel::unbounded()
        });

        static SUBSCRIBERS: std::sync::LazyLock<
            scc::HashMap<String, Box<dyn Fn($event) + Sync + Send + 'static>>,
        > = std::sync::LazyLock::new(scc::HashMap::new);

        static THREAD_INIT: std::sync::Once = std::sync::Once::new();

        #[allow(dead_code)]
        impl $name {
            pub fn event_tx() -> crossbeam_channel::Sender<$event> {
                CHANNEL.0.clone()
            }

            pub fn send(event: $event) {
                THREAD_INIT.call_once(|| {
                    let rx = CHANNEL.1.clone();
                    std::thread::spawn(move || {
                        for event in rx {
                            SUBSCRIBERS.scan(|_k, v| {
                                v(event.clone());
                            });
                        }
                    });
                });

                if let Err(e) = CHANNEL.0.send(event) {
                    log::error!("Failed to send event: {e}");
                }
            }

            pub fn subscribe<F>(callback: F) -> String
            where
                F: Fn($event) + Sync + Send + 'static,
            {
                let id = uuid::Uuid::new_v4().to_string();
                let _ = SUBSCRIBERS.insert(id.clone(), Box::new(callback));
                id
            }

            pub fn unsubscribe(id: &str) {
                SUBSCRIBERS.remove(id);
            }
        }
    };
}
