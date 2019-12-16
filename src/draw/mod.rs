//! Traits outlining the theme and drawing API.

pub mod state;

use {
    crate::base,
    reclutch::display::{DisplayCommand, Size, StyleColor},
};

/// Implemented by types which are capable of changing themes.
pub trait Themed {
    /// Updates `self` from `theme`.
    fn load_theme(&mut self, theme: &dyn Theme, aux: &dyn base::GraphicalAuxiliary);
}

/// Empty `Themed` type to assist in satisfying `HasTheme` required by `WidgetChildren`
/// for widgets which don't have a visual appearance (e.g. layout widgets).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhantomThemed;

impl Themed for PhantomThemed {
    fn load_theme(&mut self, _theme: &dyn Theme, _aux: &dyn base::GraphicalAuxiliary) {}
}

/// Object of a theme which paints a single state (which typically represents a single widget).
pub trait Painter<T> {
    /// Invokes the corresponding method from a given `Theme` to retrieve the same
    /// `Painter` which `self` was built from previously.
    fn invoke(&self, theme: &dyn Theme) -> Box<dyn Painter<T>>;
    /// Returns a stylistic size based on the state.
    fn size_hint(&self, state: T, aux: &dyn base::GraphicalAuxiliary) -> Size;
    /// Returns a list of display commands which visualize `state`.
    fn draw(&mut self, state: T, aux: &dyn base::GraphicalAuxiliary) -> Vec<DisplayCommand>;
}

/// Factory to create colors or `Painter`s which paint widgets with a specific visual theme.
pub trait Theme {
    fn button(&self) -> Box<dyn Painter<state::ButtonState>>;
    fn label_color(&self) -> StyleColor;
    fn default_text_size(&self) -> f32;
}

/// Implemented by types which have an inner `Themed` (but usually widgets with
/// an inner `Box<Painter<_>>`, which implements `Themed`).
pub trait HasTheme {
    /// Returns the inner `Themed`.
    fn theme(&mut self) -> &mut dyn Themed;
    /// *Possibly* invokes `size_hint` on the inner `Painter` and applies it.
    fn resize_from_theme(&mut self, aux: &dyn base::GraphicalAuxiliary);
}

impl<T> Themed for Box<dyn Painter<T>> {
    fn load_theme(&mut self, theme: &dyn Theme, _aux: &dyn base::GraphicalAuxiliary) {
        *self = self.invoke(theme);
    }
}

impl<T> Themed for T
where
    T: HasTheme,
{
    fn load_theme(&mut self, theme: &dyn Theme, aux: &dyn base::GraphicalAuxiliary) {
        self.theme().load_theme(theme, aux);
        self.resize_from_theme(aux);
    }
}
