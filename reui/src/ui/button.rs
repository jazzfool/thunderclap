//! Button control widget.

use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state, HasTheme},
        pipe,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Point, Rect},
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

/// Creates an unbound terminal which handles window events for a logical button.
pub fn button_terminal<B, U>() -> pipe::UnboundTerminal<B, U, base::WindowEvent>
where
    B: LogicalButton,
    U: base::UpdateAuxiliary + 'static,
{
    unbound_terminal! {
        B as obj,
        U as _aux,
        base::WindowEvent as event,

        mouse_press {
            if let Some((pos, _, _)) = event.with(|(pos, button, _)| {
                !obj.disabled()
                    && *button == base::MouseButton::Left
                    && obj.mouse_bounds().contains(*pos)
            }) {
                obj.interaction().insert(state::InteractionState::PRESSED);
                obj.event_queue().emit_owned(ButtonEvent::Press(*pos));
                obj.repaint();
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
                obj.event_queue().emit_owned(ButtonEvent::Release(*pos));
                obj.repaint();
            }
        }

        mouse_move {
            if let Some((pos, _)) = event.with(|(pos, _)| obj.mouse_bounds().contains(*pos)) {
                if !obj.interaction().contains(state::InteractionState::HOVERED) {
                    obj.interaction().insert(state::InteractionState::HOVERED);
                    obj.event_queue()
                        .emit_owned(ButtonEvent::BeginHover(pos.clone()));
                    obj.repaint();
                }
            } else if obj.interaction().contains(state::InteractionState::HOVERED) {
                obj.interaction().remove(state::InteractionState::HOVERED);
                obj.event_queue()
                    .emit_owned(ButtonEvent::EndHover(event.get().0));
                obj.repaint();
            }
        }

        clear_focus {
            obj.interaction().remove(state::InteractionState::FOCUSED);
        }
    }
}

/// Getters required for a button window event handler.
pub trait LogicalButton: Repaintable {
    /// Returns a mutable reference to the user interaction state.
    fn interaction(&mut self) -> &mut state::InteractionState;
    /// Returns a mutable reference to the output `ButtonEvent` event queue.
    fn event_queue(&mut self) -> &mut RcEventQueue<ButtonEvent>;
    /// Returns the rectangle which captures mouse events.
    fn mouse_bounds(&self) -> Rect;
    /// Returns the disabled state.
    fn disabled(&self) -> bool;
}

/// Focus-able button widget.
#[derive(
    WidgetChildren, LayableWidget, DropNotifier, HasVisibility, Repaintable, Movable, Resizable,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
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
    pipe: Option<pipe::Pipeline<Self, U>>,
    interaction: state::InteractionState,
    painter: Box<dyn draw::Painter<state::ButtonState>>,

    #[widget_rect]
    rect: Rect,
    #[widget_visibility]
    visibility: base::Visibility,
    #[repaint_target]
    command_group: CommandGroup,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,
    #[widget_drop_event]
    drop_event: RcEventQueue<base::DropEvent>,

    phantom_g: PhantomData<G>,
}

impl<U, G> LogicalButton for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline(always)]
    fn event_queue(&mut self) -> &mut RcEventQueue<ButtonEvent> {
        &mut self.event_queue
    }

    #[inline]
    fn mouse_bounds(&self) -> Rect {
        self.painter.mouse_hint(self.rect)
    }

    #[inline(always)]
    fn disabled(&self) -> bool {
        *self.disabled.get()
    }
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
        u_aux: &mut U,
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

        observe![text, text_size, button_type, disabled];

        let mut pipe = pipeline! {
            Self as obj,
            U as _aux,
            _ev in &text.on_change => {
                change { obj.resize_from_theme(); obj.command_group.repaint(); }
            }
            _ev in &text_size.on_change => {
                change { obj.resize_from_theme(); obj.command_group.repaint(); }
            }
            _ev in &button_type.on_change => {
                change { obj.resize_from_theme(); obj.command_group.repaint(); }
            }
            _ev in &disabled.on_change => {
                change { obj.command_group.repaint(); }
            }
        };

        pipe = pipe.add(button_terminal::<Self, U>().bind(u_aux.window_queue()));

        let size = painter.size_hint(temp_state);

        Self {
            event_queue: RcEventQueue::new(),

            text,
            text_size,
            button_type,
            disabled,
            pipe: pipe.into(),
            interaction: state::InteractionState::empty(),
            painter,

            rect: Rect::new(position, size),
            visibility: Default::default(),
            command_group: CommandGroup::new(),
            layout: Default::default(),
            drop_event: Default::default(),

            phantom_g: Default::default(),
        }
    }

    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.rect);
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

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let button_state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(button_state), None, None);
    }
}

impl<U, G> HasTheme for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {
        self.set_size(self.painter.size_hint(state::ButtonState {
            state: state::ControlState::Normal(state::InteractionState::empty()),
            button_type: state::ButtonType::Normal,
            ..self.derive_state()
        }));
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
