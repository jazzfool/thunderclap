//! The main part of Reui; a widget toolkit built atop Reclutch.

pub mod button;
pub mod checkbox;
pub mod container;
pub mod hstack;
pub mod label;
pub mod text_area;
pub mod vstack;

pub use {button::*, checkbox::*, container::*, hstack::*, label::*, text_area::*, vstack::*};

use {
    crate::{base, draw::state, pipe},
    reclutch::display::{Point, Rect},
};

/// Simply pushes a list of widgets, each with specified layout data, into a layout, then returns a mutable reference to the layout.
///
/// # Example
/// ```ignore
/// define_layout! {
///     for layout {
///         layout_data => &mut widget_1,
///         layout_data => &mut widget_2
///     }
/// }
/// ```
/// Where `layout` implements `reui::base::Layout`, `layout_data` is of type `<layout as Layout>::PushData` and `widget_1`/`widget_2` implement `Layable`.
/// The above is equivalent to:
/// ```ignore
/// layout.push(layout_data, &mut widget_1);
/// layout.push(layout_data, &mut widget_2);
/// ```
///
/// Due to returning a mutable reference to the layout, this macro can be nested so as to nest layouts:
/// ```ignore
/// define_layout! {
///    for parent_layout {
///        layout_data => define_layout! {
///            for child_layout { layout_data => child }
///        }
///    }
/// }
/// ```
///
///
#[macro_export]
macro_rules! define_layout {
    (for $layout:expr => {$($data:expr => $target:expr),*}) => {
        {
            use $crate::base::Layout;
            $(
                $layout.push($data, $target);
            )*
            &mut $layout
        }
    }
}

/// How a child should be aligned within a layout.
/// On which axis the align applies to depends on the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// The child is aligned to the beginning of the layout.
    Begin,
    /// The child is centered.
    Middle,
    /// The child is aligned to the end of the layout.
    End,
    /// The child is stretched to fill the container.
    Stretch,
}

impl Default for Align {
    fn default() -> Self {
        Align::Begin
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionEvent {
    Pressed(Point),
    Released(Point),
    BeginHover(Point),
    EndHover(Point),
    Focus,
    Blur,
}

pub trait InteractiveWidget {
    fn interaction(&mut self) -> &mut state::InteractionState;
    fn mouse_bounds(&self) -> Rect;
    fn disabled(&self) -> bool;
    fn on_interaction_event(&mut self, event: InteractionEvent);
}

/// Generates an unbound terminal which handles basic interactivity.
/// This simply means it will appropriately modify a `state::InteractionState` and emit events
/// when interactivity changes occur.
pub fn basic_interaction_terminal<W: InteractiveWidget, U: base::UpdateAuxiliary + 'static>(
) -> pipe::UnboundTerminal<W, U, base::WindowEvent> {
    unbound_terminal! {
        W as obj,
        U as aux,
        base::WindowEvent as event,

        mouse_press {
            let bounds = aux.tracer().absolute_bounds(obj.mouse_bounds());
            if let Some((pos, _, _)) = event.with(|(pos, button, _)| {
                !obj.disabled()
                    && *button == base::MouseButton::Left
                    && bounds.contains(*pos)
            }) {
                obj.interaction().insert(state::InteractionState::PRESSED);
                obj.on_interaction_event(InteractionEvent::Pressed(*pos));
            }
        }

        mouse_release {
            if let Some((pos, _, _)) = event.with(|(_, button, _)| {
                !obj.disabled()
                    && *button == base::MouseButton::Left
                    && obj.interaction().contains(state::InteractionState::PRESSED)
            }) {
                obj.interaction().remove(state::InteractionState::PRESSED);
                obj.interaction().insert(state::InteractionState::FOCUSED);
                obj.on_interaction_event(InteractionEvent::Released(*pos));
                obj.on_interaction_event(InteractionEvent::Focus);
            }
        }

        mouse_move {
            let bounds = aux.tracer().absolute_bounds(obj.mouse_bounds());
            if let Some((pos, _)) = event.with(|(pos, _)| bounds.contains(*pos)) {
                if !obj.interaction().contains(state::InteractionState::HOVERED) {
                    obj.interaction().insert(state::InteractionState::HOVERED);
                    obj.on_interaction_event(InteractionEvent::BeginHover(*pos));
                }
            } else if obj.interaction().contains(state::InteractionState::HOVERED) {
                obj.interaction().remove(state::InteractionState::HOVERED);
                obj.on_interaction_event(InteractionEvent::EndHover(event.get().0));
            }
        }

        clear_focus {
            obj.interaction().remove(state::InteractionState::FOCUSED);
            obj.on_interaction_event(InteractionEvent::Blur);
        }
    }
}
