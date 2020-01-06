//! A collection of various themes to quickly get up and running with Thunderclap.

use crate::draw::ThemeData;

mod dynamic;
mod primer;

/// GitHub's "Primer" theme, based off the CSS widgets.
pub struct Primer {
    data: ThemeData,
}

/// Theme generated from a RON (Rusty Object Notation) file.
pub struct Dynamic {
    //data: ThemeData,
}
