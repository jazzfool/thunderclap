//! A collection of various themes to quickly get up and running with Reui.

use crate::draw::ThemeData;

mod primer;

/// GitHub's "Primer" theme, based off the CSS widgets.
pub struct Primer {
    data: ThemeData,
}
