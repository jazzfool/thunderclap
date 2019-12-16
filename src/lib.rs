//! Reui aims to be a large widget toolkit for Reclutch.
//! Beyond this, it also defines a framework to create widgets from.

#[macro_use]
pub extern crate reclutch;

pub mod base;
pub mod draw;
pub mod error;
pub mod themes;
pub mod ui;

#[cfg(feature = "app")]
pub mod app;

pub mod prelude {
    pub use crate::base::{Layout, Movable, Rectangular, Repaintable, Resizable, WidgetChildren};
}
