//! Button control widget.

use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        pipe,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Point, Rect, Size},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Events emitted by a button.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq)]
#[reui_crate(crate)]
pub enum ButtonEvent {
    /// Emitted when the checkbox is pressed.
    #[event_key(press)]
    Press(Point),
    /// Emitted when the checkbox is released.
    #[event_key(release)]
    Release(Point),
    /// Emitted when the mouse enters the checkbox boundaries.
    #[event_key(begin_hover)]
    BeginHover(Point),
    /// Emitted when the mouse leaves the checkbox boundaries.
    #[event_key(end_hover)]
    EndHover(Point),
    /// Emitted when focus is gained.
    #[event_key(focus)]
    Focus,
    /// Emitted when focus is lost.
    #[event_key(blur)]
    Blur,
}

/// Focus-able button widget.
#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<ButtonEvent>,

    pub text: base::Observed<DisplayText>,
    pub text_size: base::Observed<Option<f32>>,
    pub button_type: base::Observed<state::ButtonType>,
    pub disabled: base::Observed<bool>,
    rect: Rect,

    interaction: state::InteractionState,
    visibility: base::Visibility,
    painter: Box<dyn draw::Painter<state::ButtonState>>,
    command_group: CommandGroup,
    layout: base::WidgetLayoutEvents,
    drop_event: RcEventQueue<base::DropEvent>,
    pipe: Option<pipe::Pipeline<Self, U>>,

    phantom_g: PhantomData<G>,
}

impl<U, G> Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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

        let pipe = pipeline! {
            Self as obj,
            U as _aux,
            _ev in &text.on_change => { change { obj.command_group.repaint(); } }
            _ev in &text_size.on_change => { change { obj.command_group.repaint(); } }
            _ev in &button_type.on_change => { change { obj.command_group.repaint(); } }
            _ev in &disabled.on_change => { change { obj.command_group.repaint(); } }
            event in update_aux.window_queue() => {
                mouse_press {
                    if let Some((pos, _)) = event.with(|(pos, button)| {
                        !*obj.disabled.get() && *button == base::MouseButton::Left && obj.bounds().contains(*pos)
                    }) {
                        obj.interaction.insert(state::InteractionState::PRESSED);
                        obj.event_queue.emit_owned(ButtonEvent::Press(*pos));
                        obj.command_group.repaint();
                    }
                }
                mouse_release {
                    if let Some((pos, _)) = event.with(|(_, button)| {
                        !*obj.disabled.get()
                            && *button == base::MouseButton::Left
                            && obj.interaction.contains(state::InteractionState::PRESSED)
                    }) {
                        obj.interaction.remove(state::InteractionState::PRESSED);
                        obj.interaction.insert(state::InteractionState::FOCUSED);
                        obj.event_queue.emit_owned(ButtonEvent::Release(*pos));
                        obj.command_group.repaint();
                    }
                }
                mouse_move {
                    if let Some(pos) = event.with(|pos| obj.bounds().contains(*pos)) {
                        if !obj.interaction.contains(state::InteractionState::HOVERED) {
                            obj.interaction.insert(state::InteractionState::HOVERED);
                            obj.event_queue
                                .emit_owned(ButtonEvent::BeginHover(pos.clone()));
                            obj.command_group.repaint();
                        }
                    } else if obj.interaction.contains(state::InteractionState::HOVERED) {
                        obj.interaction.remove(state::InteractionState::HOVERED);
                        obj.event_queue
                            .emit_owned(ButtonEvent::EndHover(event.get().clone()));
                        obj.command_group.repaint();
                    }
                }
                clear_focus {
                    obj.interaction.remove(state::InteractionState::FOCUSED);
                }
            }
        };

        Self {
            event_queue: RcEventQueue::new(),

            text,
            text_size,
            button_type,
            disabled,
            rect: Rect::new(position, painter.size_hint(temp_state, gfx_aux)),

            interaction: state::InteractionState::empty(),
            visibility: Default::default(),
            painter,
            command_group: CommandGroup::new(),
            layout: Default::default(),
            drop_event: Default::default(),
            pipe: pipe.into(),

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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect)
    }

    fn update(&mut self, aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if was_focused != self.interaction.contains(state::InteractionState::FOCUSED) {
            self.command_group.repaint();
            self.event_queue.emit_owned(if !was_focused {
                ButtonEvent::Focus
            } else {
                ButtonEvent::Blur
            });
        }

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let button_state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(button_state, aux), None, None);
    }
}

impl<U, G> base::LayableWidget for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
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

impl<U, G> base::DropNotifier for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn drop_event(&self) -> &RcEventQueue<base::DropEvent> {
        &self.drop_event
    }
}

impl<U, G> Drop for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
