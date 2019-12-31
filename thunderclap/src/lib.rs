//! Reui aims to be a large widget toolkit for Reclutch.
//! Beyond this, it also defines a framework to create widgets from.

#[macro_use]
pub extern crate reclutch;

#[allow(unused_imports)]
#[macro_use]
extern crate thunderclap_macros;

pub use thunderclap_macros::{
    rooftop, DropNotifier, HasVisibility, LayableWidget, Movable, PipelineEvent, Repaintable,
    Resizable,
};

pub use paste;

#[macro_use]
pub mod base;
pub mod draw;
pub mod error;
#[macro_use]
pub mod pipe;
pub mod geom;
pub mod ui;

#[cfg(feature = "app")]
pub mod app;
#[cfg(feature = "default-themes")]
pub mod themes;

pub mod prelude {
    pub use crate::{
        base::{Layout, Movable, Rectangular, Repaintable, Resizable, WidgetChildren},
        geom::{ContextuallyMovable, ContextuallyRectangular},
    };
}
