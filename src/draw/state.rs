//! The visual states for a widget.
//!
//! These are simply the fields relevant to rendering, existing only
//! in the scope of the `draw` method.

use reclutch::display::{DisplayText, Rect};

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonState {
    pub rect: Rect,
    pub text: DisplayText,
    pub text_size: Option<f32>,
    pub state: ControlState,
    pub button_type: ButtonType,
}

bitflags::bitflags! {
    pub struct InteractionState: u32 {
        const HOVERED = 1 << 0;
        const PRESSED = 1 << 1;
        const FOCUSED = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlState {
    Normal(InteractionState),
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonType {
    Normal,
    Primary,
    Danger,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxState {
    pub rect: Rect,
    pub checked: bool,
    pub state: ControlState,
}
