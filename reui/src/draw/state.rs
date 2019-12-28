//! The visual states for a widget.
//!
//! These are simply the fields relevant to rendering, existing only
//! in the scope of the `draw` method.

use {crate::ui, reclutch::display::Rect};

/// Visually relevant states of a [`Button`](../ui/struct.Button.html).
#[derive(Debug, Clone)]
pub struct ButtonState {
    pub rect: Rect,
    pub data: ui::ButtonData,
    pub interaction: InteractionState,
}

bitflags::bitflags! {
    pub struct InteractionState: u32 {
        const HOVERED = 1 << 0;
        const PRESSED = 1 << 1;
        const FOCUSED = 1 << 2;
    }
}

/// Visually relevant states of a [`Checkbox`](../ui/struct.Checkbox.html).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxState {
    pub rect: Rect,
    pub data: ui::CheckboxData,
    pub interaction: InteractionState,
}

/// Visually relevant states of a [`TextArea`](../ui/struct.TextArea.html).
#[derive(Debug, Clone, PartialEq)]
pub struct TextAreaState {
    pub rect: Rect,
    pub data: ui::TextAreaData,
    pub interaction: InteractionState,
}

/// Text which can either be display normally or as placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputText {
    Normal(String),
    Placeholder(String),
}
