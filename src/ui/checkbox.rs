use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        ui::ToggledEvent,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, GraphicsDisplay, Point, Rect, Size},
        event::{RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckboxEvent {
    Press(ToggledEvent<Point>),
    Check(ToggledEvent<Point>),
    MouseHover(ToggledEvent<Point>),
    Focus(ToggledEvent<()>),
}

#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Checkbox<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub event_queue: RcEventQueue<CheckboxEvent>,

    pub checked: base::Observed<bool>,
    pub disabled: base::Observed<bool>,
    rect: Rect,

    command_group: CommandGroup,
    painter: Box<dyn draw::Painter<state::CheckboxState>>,
    layout: base::WidgetLayoutEvents,
    visibility: base::Visibility,
    repaint_listeners: Vec<RcEventListener<()>>,
    interaction: state::InteractionState,
    window_listener: RcEventListener<base::WindowEvent>,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U, G> Checkbox<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub fn new(
        theme: &dyn draw::Theme,
        checked: bool,
        disabled: bool,
        position: Point,
        u_aux: &mut U,
        g_aux: &mut G,
    ) -> Self {
        let temp_state = state::CheckboxState {
            rect: Default::default(),
            checked,
            state: state::ControlState::Normal(state::InteractionState::empty()),
        };

        let painter = theme.checkbox();
        let rect = Rect::new(position, painter.size_hint(temp_state, g_aux));

        let checked = base::Observed::new(checked);
        let disabled = base::Observed::new(disabled);

        let repaint_listeners = vec![checked.on_change.listen(), disabled.on_change.listen()];

        Checkbox {
            event_queue: Default::default(),

            checked,
            disabled,
            rect,

            command_group: Default::default(),
            painter,
            layout: Default::default(),
            visibility: Default::default(),
            repaint_listeners,
            interaction: state::InteractionState::empty(),
            window_listener: u_aux.window_queue().listen(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    fn derive_state(&self) -> state::CheckboxState {
        state::CheckboxState {
            rect: self.rect,
            checked: *self.checked.get(),
            state: if *self.disabled.get() {
                state::ControlState::Disabled
            } else {
                state::ControlState::Normal(self.interaction)
            },
        }
    }
}

impl<U, G> Widget for Checkbox<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect)
    }

    fn update(&mut self, _aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);
        let bounds = self.painter.mouse_hint(self.rect);
        let cmd_group = &mut self.command_group;
        let disabled = *self.disabled.get();

        for rl in &mut self.repaint_listeners {
            if !rl.peek().is_empty() {
                cmd_group.repaint();
            }
        }

        {
            let interaction = &mut self.interaction;
            let event_queue = &mut self.event_queue;
            let checked = &mut self.checked;

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
                                event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                                    true, *pos,
                                )));
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
                                event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                                    false, *pos,
                                )));

                                checked.set(!*checked.get());
                                event_queue.emit_owned(CheckboxEvent::Press(ToggledEvent::new(
                                    *checked.get(),
                                    *pos,
                                )));

                                cmd_group.repaint();
                            }
                        }
                        base::WindowEvent::MouseMove(move_event) => {
                            if let Some(pos) = move_event.with(|pos| bounds.contains(*pos)) {
                                if !interaction.contains(state::InteractionState::HOVERED) {
                                    interaction.insert(state::InteractionState::HOVERED);
                                    event_queue.emit_owned(CheckboxEvent::MouseHover(
                                        ToggledEvent::new(true, pos.clone()),
                                    ));
                                    cmd_group.repaint();
                                }
                            } else if interaction.contains(state::InteractionState::HOVERED) {
                                interaction.remove(state::InteractionState::HOVERED);
                                event_queue.emit_owned(CheckboxEvent::MouseHover(
                                    ToggledEvent::new(false, move_event.get().clone()),
                                ));
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
                .emit_owned(CheckboxEvent::Focus(ToggledEvent::new(!was_focused, ())));
        }

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            cmd_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group
            .push_with(display, || painter.draw(state, aux), None, None);
    }
}

impl<U, G> base::LayableWidget for Checkbox<U, G>
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

impl<U, G> base::HasVisibility for Checkbox<U, G>
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

impl<U, G> Repaintable for Checkbox<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn repaint(&mut self) {
        self.command_group.repaint();
    }
}

impl<U, G> base::Movable for Checkbox<U, G>
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

impl<U, G> Resizable for Checkbox<U, G>
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

impl<U, G> draw::HasTheme for Checkbox<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self, aux: &dyn base::GraphicalAuxiliary) {
        self.set_size(self.painter.size_hint(self.derive_state(), aux));
    }
}
