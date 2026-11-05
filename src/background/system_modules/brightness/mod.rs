//! Brightness of the monitors. Its changes are reported to the webviews by the `monitors`
//! module, as part of the monitor data.

mod application;
mod domain;

pub use self::{application::BrightnessManager, domain::BrightnessEvent};
