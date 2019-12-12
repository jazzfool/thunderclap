//! The main part of Reui; a widget toolkit built atop Reclutch.

pub mod button;
pub mod vstack;

pub use {button::*, vstack::*};

use {
    crate::{base, draw},
    reclutch::display,
};

pub fn simple_button<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary>(
    text: String,
    theme: &dyn draw::Theme,
    update_aux: &mut U,
    gfx_aux: &mut G,
) -> Button<U, G> {
    Button::new(
        display::DisplayText::Simple(text),
        display::Point::default(),
        None,
        draw::state::ButtonType::Normal,
        false,
        theme,
        update_aux,
        gfx_aux,
    )
}
