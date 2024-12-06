pub mod cli;
pub mod input;
pub mod media;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod system_settings;
pub mod tray;
pub mod uwp;
pub mod virtual_desk;

#[macro_export]
macro_rules! event_manager {
    ($name:ident, $event:ty) => {
        static CHANNEL: std::sync::OnceLock<(
            crossbeam_channel::Sender<$event>,
            crossbeam_channel::Receiver<$event>,
        )> = std::sync::OnceLock::new();

        impl $name {
            pub fn channel() -> &'static (
                crossbeam_channel::Sender<$event>,
                crossbeam_channel::Receiver<$event>,
            ) {
                CHANNEL.get_or_init(crossbeam_channel::unbounded)
            }

            pub fn event_tx() -> crossbeam_channel::Sender<$event> {
                Self::channel().0.clone()
            }

            pub fn event_rx() -> crossbeam_channel::Receiver<$event> {
                Self::channel().1.clone()
            }
        }
    };
}
