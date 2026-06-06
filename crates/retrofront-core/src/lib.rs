//! Rust libretro host used by the Swift frontends.
//!
//! This crate intentionally keeps platform policy out of the emulation loop: Linux
//! and iOS frontends provide windows, input and audio devices, while Rust owns the
//! libretro ABI, callbacks, configuration, playlists, save/runtime paths and the
//! frame scheduler.

pub mod config;
pub mod dynlib;
pub mod ffi;
pub mod host;
pub mod libretro;
pub mod menu;
pub mod overlay;

pub use config::{FrontendConfig, PathConfig};
pub use host::{CoreMetadata, FrameResult, RetroHost};
pub use menu::{MenuItem, MenuModel, SkinTheme};
pub use overlay::{ControllerSkin, ControllerSkinButton, OverlayHitShape};
