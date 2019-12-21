//! Button control widget.

use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        ui::ToggledEvent,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Point, Rect, Size},
        event::{RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Events emitted by a button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonEvent {
    /// Emitted when the button is pressed or released.
    /// Corresponds to `WindowEvent::MousePress` or `WindowEvent::MouseRelease`.
    Press(ToggledEvent<Point>),
    /// Emitted when the mouse enters (`true`) or leaves (`false`) the button boundaries.
    /// Corresponds to `WindowEvent::MouseMove`.
    MouseHover(ToggledEvent<Point>),
    /// Emitted when focus is gained (`true`) or lost (`false`).
    Focus(ToggledEvent<()>),
}

/// Focus-able button widget.
#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub event_queue: RcEventQueue<ButtonEvent>,

    pub text: base::Observed<DisplayText>,
    pub text_size: base::Observed<Option<f32>>,
    rect: Rect,
    pub button_type: base::Observed<state::ButtonType>,
    pub disabled: base::Observed<bool>,
    interaction: state::InteractionState,
    visibility: base::Visibility,

    repaint_listeners: Vec<RcEventListener<()>>,
    painter: Box<dyn draw::Painter<state::ButtonState>>,
    command_group: CommandGroup,
    window_listener: RcEventListener<base::WindowEvent>,
    layout: base::WidgetLayoutEvents,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U, G> Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    /// Creates a new button widget with a specified label, position, label size, visual type, disabled state and theme.
    /// If `None` is passed to `text_size` then the text size will be decided by the theme (`theme`).
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

        let text = base::Observed::new(text);
        let text_size = base::Observed::new(text_size);
        let button_type = base::Observed::new(button_type);
        let disabled = base::Observed::new(disabled);

        let repaint_listeners = vec![
            text.on_change.listen(),
            text_size.on_change.listen(),
            button_type.on_change.listen(),
            disabled.on_change.listen(),
        ];

        Self {
            event_queue: RcEventQueue::new(),

            text,
            text_size,
            rect: Rect::new(position, painter.size_hint(temp_state, gfx_aux)),
            button_type,
            disabled,
            interaction: state::InteractionState::empty(),
            visibility: Default::default(),

            repaint_listeners,
            painter,
            command_group: CommandGroup::new(),
            window_listener: update_aux.window_queue_mut().listen(),
            layout: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    fn derive_state(&self) -> state::ButtonState {
        state::ButtonState {
            rect: self.rect,
            text: self.text.get().clone(),
            text_size: self.text_size.get().clone(),
            state: if *self.disabled.get() {
                state::ControlState::Disabled
            } else {
                state::ControlState::Normal(self.interaction)
            },
            button_type: self.button_type.get().clone(),
        }
    }
}

impl<U, G> Widget for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect)
    }

    fn update(&mut self, _aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);
        let disabled = *self.disabled.get();

        let bounds = self.painter.mouse_hint(self.rect);
        let cmd_group = &mut self.command_group;

        for rl in &mut self.repaint_listeners {
            if !rl.peek().is_empty() {
                cmd_group.repaint();
            }
        }

        {
            let interaction = &mut self.interaction;
            let event_queue = &mut self.event_queue;

            self.window_listener.with(|events| {
                for event in events {
                    match event {
                        base::WindowEvent::MousePress(press_event) => {
                            if let Some((pos, _)) = press_event.with(|(pos, button)| {
                                !disabled
                                    && *button == base::MouseButton::Left
                                    && bounds.contains(*pos)
                            }) {
                                interaction.insert(state::InteractionState::PRESSED);
                                event_queue
                                    .emit_owned(ButtonEvent::Press(ToggledEvent::new(true, *pos)));
                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::MouseRelease(release_event) => {
                            if let Some((pos, _)) = release_event.with(|(_, button)| {
                                !disabled
                                    && *button == base::MouseButton::Left
                                    && interaction.contains(state::InteractionState::PRESSED)
                            }) {
                                interaction.remove(state::InteractionState::PRESSED);
                                interaction.insert(state::InteractionState::FOCUSED);
                                event_queue
                                    .emit_owned(ButtonEvent::Press(ToggledEvent::new(false, *pos)));
                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::MouseMove(move_event) => {
                            if let Some(pos) = move_event.with(|pos| bounds.contains(*pos)) {
                                if !interaction.contains(state::InteractionState::HOVERED) {
                                    interaction.insert(state::InteractionState::HOVERED);
                                    event_queue.emit_owned(ButtonEvent::MouseHover(
                                        ToggledEvent::new(true, pos.clone()),
                                    ));
                                    cmd_group.repaint();
                                }
                            } else if interaction.contains(state::InteractionState::HOVERED) {
                                interaction.remove(state::InteractionState::HOVERED);
                                event_queue.emit_owned(ButtonEvent::MouseHover(ToggledEvent::new(
                                    false,
                                    move_event.get().clone(),
                                )));
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
            cmd_group.repaint();
            self.event_queue
                .emit_owned(ButtonEvent::Focus(ToggledEvent::new(!was_focused, ())));
        }

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            cmd_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let button_state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group
            .push_with(display, || painter.draw(button_state, aux), None, None);
    }
}

impl<U, G> base::LayableWidget for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}

impl<U, G> base::HasVisibility for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn set_visibility(&mut self, visibility: base::Visibility) {
        self.visibility = visibility
    }

    #[inline]
    fn visibility(&self) -> base::Visibility {
        self.visibility
    }
}

impl<U, G> Repaintable for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn repaint(&mut self) {
        self.command_group.repaint();
    }
}

// FIXME(jazzfool): the blanket `Rectangular` implementation causes `self.layout.notify()` to be called twice.
// to be frank, this isn't a big deal since bidir_single overwrites the previous event, but in the old implementation this meant emitting the event twice.

impl<U, G> base::Movable for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
        self.repaint();
        self.layout.notify(self.rect);
    }

    #[inline]
    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U, G> Resizable for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
        self.repaint();
        self.layout.notify(self.rect);
    }

    #[inline]
    fn size(&self) -> Size {
        self.rect.size
    }
}

impl<U, G> draw::HasTheme for Button<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self, aux: &dyn base::GraphicalAuxiliary) {
        self.set_size(self.painter.size_hint(
            state::ButtonState {
                state: state::ControlState::Normal(state::InteractionState::empty()),
                button_type: state::ButtonType::Normal,
                ..self.derive_state()
            },
            aux,
        ));
    }
}
