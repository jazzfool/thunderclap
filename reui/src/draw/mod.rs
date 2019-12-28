//! Simple theme framework based on Flutter.

pub mod state;

use {
    crate::base,
    reclutch::display::{Color, DisplayCommand, FontInfo, Rect, ResourceReference, Size},
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
    fn paint_hint(&self, rect: Rect) -> Rect;
    /// Returns the mouse boundaries based on the inner bounds.
    fn mouse_hint(&self, rect: Rect) -> Rect;
    /// Returns a list of display commands which visualize `state`.
    fn draw(&mut self, state: T) -> Vec<DisplayCommand>;
}

#[inline]
fn map_range(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    (x - a) / (b - a) * (d - c) + c
}

#[inline]
fn interp_shadepoint(x: f32) -> f32 {
    map_range(x, 0.05, 0.9, 0.0, 1.0)
}

/// 10 different shades of colors.
/// Lower shades are lighter and less saturated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorSwatch {
    pub shade_50: Color,
    pub shade_100: Color,
    pub shade_200: Color,
    pub shade_300: Color,
    pub shade_400: Color,
    pub shade_500: Color,
    pub shade_600: Color,
    pub shade_700: Color,
    pub shade_800: Color,
    pub shade_900: Color,
}

impl ColorSwatch {
    /// Generates a swatch from a mid-tone color (`shade_500`).
    /// Deviation is the value and relative inverse saturation deviation of the extremities (`shade_50` and `shade_900`)
    /// from the mid-tone (`shade_500`).
    /// If you are unsure of what `deviation` value to pass in, around `0.3` is fine.
    /// As deviation approaches 1, `shade_50` will approach white and `shade_900` will approach black.
    pub fn generate(shade_500: Color, deviation: f32) -> Self {
        use reclutch::palette as pal;

        let shade_500: pal::Hsva = shade_500.into();
        let (shade_50, shade_900) = {
            let mut shade_50 = shade_500;
            let mut shade_900 = shade_500;

            shade_50.value += deviation;
            shade_50.saturation -= shade_50.saturation * deviation;
            shade_900.value -= deviation;
            shade_900.saturation += shade_900.saturation * deviation;

            (shade_50, shade_900)
        };

        let grad = pal::gradient::Gradient::new([shade_50, shade_500, shade_900].iter().cloned());

        ColorSwatch {
            shade_50: grad.get(interp_shadepoint(0.05)).into(),
            shade_100: grad.get(interp_shadepoint(0.10)).into(),
            shade_200: grad.get(interp_shadepoint(0.20)).into(),
            shade_300: grad.get(interp_shadepoint(0.30)).into(),
            shade_400: grad.get(interp_shadepoint(0.40)).into(),
            shade_500: shade_500.into(),
            shade_600: grad.get(interp_shadepoint(0.60)).into(),
            shade_700: grad.get(interp_shadepoint(0.70)).into(),
            shade_800: grad.get(interp_shadepoint(0.80)).into(),
            shade_900: grad.get(interp_shadepoint(0.90)).into(),
        }
    }

    /// Returns a shade which is weak with regards to the contrast.
    /// For example, `swatch.weaken(ThemeContrast::Dark, 3)` will return
    /// `swatch.shade_800`. However, if `ThemeContrast::Light` is passed
    /// instead, then `swatch.shade_200` is returned instead. This is so
    /// that it gives of the effect of blending into the background; weakening.
    pub fn weaken_500(&self, contrast: ThemeContrast, steps: u8) -> Color {
        match contrast {
            ThemeContrast::Light => self[500 - (steps as u16 * 100)],
            ThemeContrast::Dark => self[500 + (steps as u16 * 100)],
        }
    }

    /// Returns a shade which is in the foreground with regards to the contrast.
    /// For example, `swatch.strengthen_500(ThemeContrast::Dark, 3)` will return
    /// `swatch.shade_200`. However, if `ThemeContrast::Light` is passed
    /// instead, then `swatch.shade_800` is returned instead. This is so
    /// that it gives of the effect of being pushed into the foreground; strengthening.
    pub fn strengthen_500(&self, contrast: ThemeContrast, steps: u8) -> Color {
        match contrast {
            ThemeContrast::Light => self[500 + (steps as u16 * 100)],
            ThemeContrast::Dark => self[500 - (steps as u16 * 100)],
        }
    }
}

impl std::ops::Index<u16> for ColorSwatch {
    type Output = Color;

    fn index(&self, shade: u16) -> &Color {
        match shade {
            0 | 50 => &self.shade_50,
            100 => &self.shade_100,
            200 => &self.shade_200,
            300 => &self.shade_300,
            400 => &self.shade_400,
            500 => &self.shade_500,
            600 => &self.shade_600,
            700 => &self.shade_700,
            800 => &self.shade_800,
            900 => &self.shade_900,
            _ => panic!("Invalid shade: {}", shade),
        }
    }
}

impl std::ops::IndexMut<u16> for ColorSwatch {
    fn index_mut(&mut self, shade: u16) -> &mut Color {
        match shade {
            0 | 50 => &mut self.shade_50,
            100 => &mut self.shade_100,
            200 => &mut self.shade_200,
            300 => &mut self.shade_300,
            400 => &mut self.shade_400,
            500 => &mut self.shade_500,
            600 => &mut self.shade_600,
            700 => &mut self.shade_700,
            800 => &mut self.shade_800,
            900 => &mut self.shade_900,
            _ => panic!("Invalid shade: {}", shade),
        }
    }
}

impl Into<Color> for ColorSwatch {
    fn into(self) -> Color {
        self.shade_500
    }
}

/// A consistent palette of colors used throughout the UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    /// Background color.
    pub background: ColorSwatch,
    /// A color which indicates an error.
    pub error: ColorSwatch,
    /// A color which indicates component focus.
    pub focus: ColorSwatch,
    /// A primary color used often.
    pub primary: ColorSwatch,
    /// A control which is "outset", like a button.
    pub control_outset: ColorSwatch,
    /// A control which is "inset", such as a text box.
    pub control_inset: ColorSwatch,
    /// A color which appears clearly over `error`.
    pub over_error: ColorSwatch,
    /// A color which appears clearly over `focus`.
    pub over_focus: ColorSwatch,
    /// A color which appears clearly over `primary`.
    pub over_primary: ColorSwatch,
    /// A color which appears clearly over `control_outset`.
    pub over_control_outset: ColorSwatch,
    /// A color which appears clearly over `control_inset`.
    pub over_control_inset: ColorSwatch,
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
