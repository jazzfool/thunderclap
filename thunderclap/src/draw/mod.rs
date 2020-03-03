//! Simple theme framework based on Flutter.

pub mod state;

use {
    crate::{base, geom::*},
    reclutch::display::{Color, DisplayCommand, FontInfo, ResourceReference, Size},
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
    fn size_hint(&self, state: T) -> Size;
    /// Returns the paint boundaries based on the inner bounds.
    fn paint_hint(&self, rect: RelativeRect) -> RelativeRect;
    /// Returns the mouse boundaries based on the inner bounds.
    fn mouse_hint(&self, rect: RelativeRect) -> RelativeRect;
    /// Returns a list of display commands which visualize `state`.
    fn draw(&mut self, state: T) -> Vec<DisplayCommand>;
}

/// Lightens a color by a specified amount
pub fn lighten(color: Color, amount: f32) -> Color {
    use reclutch::palette::Shade;
    Color::from_linear(color.into_linear().lighten(amount))
}

/// Darkens a color by a specified amount
pub fn darken(color: Color, amount: f32) -> Color {
    use reclutch::palette::Shade;
    Color::from_linear(color.into_linear().darken(amount))
}

/// Darkens or lightens a color to contrast the theme.
pub fn strengthen(color: Color, amount: f32, contrast: ThemeContrast) -> Color {
    match contrast {
        ThemeContrast::Light => darken(color, amount),
        ThemeContrast::Dark => lighten(color, amount),
    }
}

/// Darkens or lightens a color to get closer with the theme.
pub fn weaken(color: Color, amount: f32, contrast: ThemeContrast) -> Color {
    match contrast {
        ThemeContrast::Light => lighten(color, amount),
        ThemeContrast::Dark => darken(color, amount),
    }
}

/// Returns the color with a different opacity.
pub fn with_opacity(color: Color, opacity: f32) -> Color {
    Color::new(color.red, color.green, color.blue, opacity)
}

/// A consistent palette of colors used throughout the UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    /// Background color.
    pub background: Color,
    /// A color which indicates an error.
    pub error: Color,
    /// A color which indicates component focus.
    pub focus: Color,
    /// A primary color used often.
    pub primary: Color,
    /// A control which is "outset", like a button.
    pub control_outset: Color,
    /// A control which is "inset", such as a text box.
    pub control_inset: Color,
    /// A color which appears clearly over `error`.
    pub over_error: Color,
    /// A color which appears clearly over `focus`.
    pub over_focus: Color,
    /// A color which appears clearly over `primary`.
    pub over_primary: Color,
    /// A color which appears clearly over `control_outset`.
    pub over_control_outset: Color,
    /// A color which appears clearly over `control_inset`.
    pub over_control_inset: Color,
}

/// A single typeface in 2 weights and italics.
#[derive(Debug, Clone)]
pub struct Typeface {
    pub regular: (ResourceReference, FontInfo),
    pub italic: (ResourceReference, FontInfo),
    pub bold: (ResourceReference, FontInfo),
    pub bold_italic: (ResourceReference, FontInfo),
}

impl PartialEq for Typeface {
    fn eq(&self, other: &Typeface) -> bool {
        self.regular.0 == other.regular.0
            && self.italic.0 == other.italic.0
            && self.bold.0 == other.bold.0
            && self.bold_italic.0 == other.bold_italic.0
    }
}

impl Eq for Typeface {}

impl Typeface {
    pub fn pick(&self, style: TextStyle) -> (ResourceReference, FontInfo) {
        match style {
            TextStyle::Regular => self.regular.clone(),
            TextStyle::RegularItalic => self.italic.clone(),
            TextStyle::Bold => self.bold.clone(),
            TextStyle::BoldItalic => self.bold_italic.clone(),
        }
    }
}

/// Text weights and italics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    /// "Baseline" font weight.
    Regular,
    /// Italicized variant of `Regular`.
    RegularItalic,
    /// Bold font weight.
    Bold,
    /// Italicized variant of `Bold`.
    BoldItalic,
}

/// A typeface with text size and text style.
#[derive(Debug, Clone, PartialEq)]
pub struct TypefaceStyle {
    pub typeface: Typeface,
    /// Text size in pixels.
    pub size: f32,
    /// Text style (regular, italic, etc).
    pub style: TextStyle,
}

/// List of typefaces used throughout the UI.
#[derive(Debug, Clone)]
pub struct Typography {
    /// Typeface used in headers, e.g. titles.
    pub header: TypefaceStyle,
    /// Typeface used in sub-headers, which typically appear underneath `header`s.
    pub sub_header: TypefaceStyle,
    /// Typeface used in regular text.
    pub body: TypefaceStyle,
    /// Typeface used in control widgets.
    pub button: TypefaceStyle,
}

/// The "contrast" mode of a theme, i.e. light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeContrast {
    Light,
    Dark,
}

/// Various information about a theme, including color scheme and fonts.
#[derive(Debug, Clone)]
pub struct ThemeData {
    /// Color scheme of the theme.
    pub scheme: ColorScheme,
    /// A list of typefaces used stylistically within the theme.
    pub typography: Typography,
    /// Contras mode of the theme.
    pub contrast: ThemeContrast,
}

/// Factory to create colors or `Painter`s which paint widgets with a specific visual theme.
pub trait Theme {
    /// Constructs a painter for a button.
    fn button(&self) -> Box<dyn Painter<state::ButtonState>>;
    /// Constructs a painter for a checkbox.
    fn checkbox(&self) -> Box<dyn Painter<state::CheckboxState>>;
    /// Constructs a painter for a text area.
    fn text_area(&self) -> Box<dyn Painter<state::TextAreaState>>;
    /// Constructs a painter for a scroll bar.
    fn scroll_bar(&self) -> Box<dyn Painter<state::ScrollBarState>>;

    fn data(&self) -> &ThemeData;
}

/// Implemented by types which have an inner `Themed` (but usually widgets with
/// an inner `Box<Painter<_>>`, which implements `Themed`).
pub trait HasTheme {
    /// Returns the inner `Themed`.
    fn theme(&mut self) -> &mut dyn Themed;
    /// *Possibly* invokes `size_hint` on the inner `Painter` and applies it.
    fn resize_from_theme(&mut self);
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
        self.resize_from_theme();
    }
}
