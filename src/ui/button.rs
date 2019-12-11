//! Button control widget.

use {
    crate::{
        base,
        draw::{self, state},
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Point, Rect, Size},
        event::{RcEventListener, RcEventQueue},
        prelude::*,
        widget::Widget,
    },
    std::marker::PhantomData,
};

/// Focus-able button widget.
#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    /// Emitted when the button is pressed.
    /// Corresponds to `WindowEvent::MousePress`.
    pub on_press: RcEventQueue<Point>,
    /// Emitted when the button is released.
    /// Corresponds to `WindowEvent::MouseRelease`.
    pub on_release: RcEventQueue<Point>,
    /// Emitted when the mouse enters the button boundaries.
    /// Corresponds to `WindowEvent::MouseMove`.
    pub on_mouse_enter: RcEventQueue<Point>,
    /// Emitted when the mouse leaves the button boundaries.
    /// Corresponds to `WindowEvent::MouseMove`.
    /// Complements `on_mouse_enter`.
    pub on_mouse_leave: RcEventQueue<Point>,
    /// Emitted when focus is gained.
    pub on_focus: RcEventQueue<()>,
    /// Emitted when focus is lost.
    /// Complements `on_focus`.
    pub on_blur: RcEventQueue<()>,

    text: DisplayText,
    text_size: Option<f32>,
    rect: Rect,
    button_type: state::ButtonType,
    disabled: bool,
    interaction: state::InteractionState,

    painter: Box<dyn draw::Painter<state::ButtonState>>,
    command_group: CommandGroup,
    window_listener: RcEventListener<base::WindowEvent>,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Button<U, G> {
    pub fn new(
        text: DisplayText,
        position: Point,
        text_size: Option<f32>,
        button_type: state::ButtonType,
        disabled: bool,
        theme: &dyn draw::Theme,
        update_aux: &mut U,
        gfx_aux: &mut G,
    ) -> Self {
        let painter = theme.button();
        let temp_state = state::ButtonState {
            rect: Rect::default(),
            text: text.clone(),
            text_size,
            state: if disabled {
                state::ControlState::Disabled
            } else {
                state::ControlState::Normal(state::InteractionState::empty())
            },
            button_type,
        };

        Self {
            on_press: RcEventQueue::new(),
            on_release: RcEventQueue::new(),
            on_mouse_enter: RcEventQueue::new(),
            on_mouse_leave: RcEventQueue::new(),
            on_focus: RcEventQueue::new(),
            on_blur: RcEventQueue::new(),

            text,
            text_size,
            rect: Rect::new(position, painter.size_hint(temp_state, gfx_aux)),
            button_type,
            disabled,
            interaction: state::InteractionState::empty(),

            painter,
            command_group: CommandGroup::new(),
            window_listener: update_aux.window_queue_mut().listen(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for Button<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect
    }

    fn update(&mut self, _aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let bounds = self.bounds();
        let cmd_group = &mut self.command_group;

        {
            let interaction = &mut self.interaction;
            let on_press = &mut self.on_press;
            let on_release = &mut self.on_release;
            let on_mouse_enter = &mut self.on_mouse_enter;
            let on_mouse_leave = &mut self.on_mouse_leave;

            self.window_listener.with(|events| {
                for event in events {
                    match event {
                        base::WindowEvent::MousePress(press_event) => {
                            if let Some((pos, _)) = press_event.with(|(pos, button)| {
                                *button == base::MouseButton::Left && bounds.contains(*pos)
                            }) {
                                interaction.insert(state::InteractionState::PRESSED);
                                on_press.emit_owned(*pos);
                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::MouseRelease(release_event) => {
                            if let Some((pos, _)) = release_event.with(|(_, button)| {
                                *button == base::MouseButton::Left
                                    && interaction.contains(state::InteractionState::PRESSED)
                            }) {
                                interaction.remove(state::InteractionState::PRESSED);
                                interaction.insert(state::InteractionState::FOCUSED);
                                on_release.emit_owned(*pos);
                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::MouseMove(move_event) => {
                            if let Some(pos) = move_event.with(|pos| bounds.contains(*pos)) {
                                if !interaction.contains(state::InteractionState::HOVERED) {
                                    interaction.insert(state::InteractionState::HOVERED);
                                    on_mouse_enter.emit_owned(pos.clone());
                                    cmd_group.repaint();
                                }
                            } else if interaction.contains(state::InteractionState::HOVERED) {
                                interaction.remove(state::InteractionState::HOVERED);
                                on_mouse_leave.emit_owned(move_event.get().clone());
                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::ClearFocus => {
                            interaction.remove(state::InteractionState::FOCUSED);
                        }
                    }
                }
            });
        }

        if was_focused != self.interaction.contains(state::InteractionState::FOCUSED) {
            self.command_group.repaint();
            if was_focused {
                self.on_blur.emit_owned(());
            } else {
                self.on_focus.emit_owned(());
            }
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let bounds = self.bounds();
        let text = self.text.clone();
        let text_size = self.text_size;
        let disabled = self.disabled;
        let interaction = self.interaction;
        let button_type = self.button_type;
        let painter = &mut self.painter;

        self.command_group.push_with(
            display,
            || {
                painter.draw(
                    state::ButtonState {
                        rect: bounds,
                        text,
                        text_size,
                        state: if disabled {
                            state::ControlState::Disabled
                        } else {
                            state::ControlState::Normal(interaction)
                        },
                        button_type,
                    },
                    aux,
                )
            },
            None,
        );
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Movable for Button<U, G> {
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
    }

    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Resizable for Button<U, G> {
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
    }

    fn size(&self) -> Size {
        self.rect.size
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> draw::HasTheme for Button<U, G> {
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self, aux: &dyn base::GraphicalAuxiliary) {
        self.rect.size = self.painter.size_hint(
            state::ButtonState {
                rect: self.bounds(),
                text: self.text.clone(),
                text_size: self.text_size,
                state: state::ControlState::Normal(state::InteractionState::empty()),
                button_type: state::ButtonType::Normal,
            },
            aux,
        );
    }
}
