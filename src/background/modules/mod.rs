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
pub mod tray;
pub mod user;
pub mod uwp;

#[macro_export]
macro_rules! event_manager {
    ($name:ident, $event:ty) => {
        static CHANNEL: std::sync::OnceLock<(
            crossbeam_channel::Sender<$event>,
            crossbeam_channel::Receiver<$event>,
        )> = std::sync::OnceLock::new();

        static CHANNEL_THREAD: std::sync::OnceLock<std::thread::JoinHandle<()>> =
            std::sync::OnceLock::new();

        static SUBSCRIBERS: std::sync::OnceLock<
            std::sync::Arc<parking_lot::Mutex<Vec<Box<dyn FnMut($event) + Send + 'static>>>>,
        > = std::sync::OnceLock::new();

        #[allow(dead_code)]
        impl $name {
            fn channel() -> &'static (
                crossbeam_channel::Sender<$event>,
                crossbeam_channel::Receiver<$event>,
            ) {
                CHANNEL.get_or_init(crossbeam_channel::unbounded)
            }

            pub fn event_tx() -> crossbeam_channel::Sender<$event> {
                Self::channel().0.clone()
            }

            pub fn send(event: $event) {
                if let Err(e) = Self::channel().0.send(event) {
                    log::error!("Failed to send event: {e}");
                }
            }

            fn subscribers() -> &'static std::sync::Arc<
                parking_lot::Mutex<Vec<Box<dyn FnMut($event) + Send + 'static>>>,
            > {
                SUBSCRIBERS.get_or_init(|| std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())))
            }

            /// Add a new subscriber to the event manager and start the event processing thread
            pub fn subscribe<F>(callback: F)
            where
                F: FnMut($event) + Send + 'static,
            {
                let mut subs = Self::subscribers().lock();
                subs.push(Box::new(callback));
                if subs.len() != 1 {
                    return;
                }
                // Start the event processing thread on the first subscriber
                CHANNEL_THREAD.get_or_init(|| {
                    let rx = Self::channel().1.clone();
                    let subscribers = Self::subscribers().clone();
                    std::thread::spawn(move || {
                        for event in rx {
                            let mut subs = subscribers.lock();
                            for subscriber in subs.iter_mut() {
                                subscriber(event.clone());
                            }
                        }
                    })
                });
            }
        }
    };
}
