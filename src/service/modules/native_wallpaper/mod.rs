//! Native desktop wallpaper mirroring.
//!
//! While Seelen UI renders its own wallpaper, the native one is replaced by a static version of it
//! (the image itself or a thumbnail, black otherwise), so system surfaces that show the native
//! wallpaper (UAC prompt, session transitions, explorer restarts, etc.) match what the user sees.
//! The user's original wallpaper is saved to disk before the first change and restored when the
//! app asks for it or when the service stops, so it also survives an app/service crash.
//! A slideshow is restored as such. Windows Spotlight has no public API, so it is restored as its
//! last image.

mod application;
mod domain;

pub use self::application::NativeWallpaperManager;
