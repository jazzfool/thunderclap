use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state},
        geom::*,
        pipe, ui,
    },
    reclutch::{
        display::{Color, CommandGroup, DisplayCommand, GraphicsDisplay, Point, Rect},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Events emitted by a checkbox.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq)]
#[reui_crate(crate)]
pub enum CheckboxEvent {
    /// Emitted when the checkbox is pressed.
    #[event_key(press)]
    Press(AbsolutePoint),
    /// Emitted when the checkbox is released.
    #[event_key(release)]
    Release(AbsolutePoint),
    /// Emitted when the button is checked.
    #[event_key(check)]
    Check(AbsolutePoint),
    /// Emitted when the button is checked.
    #[event_key(uncheck)]
    Uncheck(AbsolutePoint),
    /// Emitted when the mouse enters the checkbox boundaries.
    #[event_key(begin_hover)]
    BeginHover(AbsolutePoint),
    /// Emitted when the mouse leaves the checkbox boundaries.
    #[event_key(end_hover)]
    EndHover(AbsolutePoint),
    /// Emitted when focus is gained.
    #[event_key(focus)]
    Focus,
    /// Emitted when focus is lost.
    #[event_key(blur)]
    Blur,
}

/// Checkbox widget; useful for boolean input.
#[derive(
    WidgetChildren, LayableWidget, DropNotifier, HasVisibility, Repaintable, Movable, Resizable,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<CheckboxEvent>,
    pub data: base::Observed<Checkbox>,

    pipe: Option<pipe::Pipeline<Self, U>>,
    painter: Box<dyn draw::Painter<state::CheckboxState>>,
    parent_position: AbsolutePoint,

    #[widget_rect]
    rect: RelativeRect,
    #[repaint_target]
    command_group: CommandGroup,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,
    #[widget_visibility]
    visibility: base::Visibility,
    interaction: state::InteractionState,
    #[widget_drop_event]
    drop_event: RcEventQueue<base::DropEvent>,

    phantom_g: PhantomData<G>,
}

impl<U, G> ui::InteractiveWidget for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline]
    fn mouse_bounds(&self) -> RelativeRect {
        self.painter.mouse_hint(self.rect)
    }

    #[inline(always)]
    fn disabled(&self) -> bool {
        self.data.disabled
    }

    fn on_interaction_event(&mut self, event: ui::InteractionEvent) {
        self.repaint();
        match event {
            ui::InteractionEvent::Pressed(pos) => {
                self.event_queue.emit_owned(CheckboxEvent::Press(pos));
            }
            ui::InteractionEvent::Released(pos) => {
                self.data.checked = !self.data.checked;
                self.event_queue.emit_owned(if self.data.checked {
                    CheckboxEvent::Check(pos)
                } else {
                    CheckboxEvent::Uncheck(pos)
                });
                self.event_queue.emit_owned(CheckboxEvent::Release(pos));
            }
            ui::InteractionEvent::BeginHover(pos) => {
                self.event_queue.emit_owned(CheckboxEvent::BeginHover(pos));
            }
            ui::InteractionEvent::EndHover(pos) => {
                self.event_queue.emit_owned(CheckboxEvent::EndHover(pos));
            }
            ui::InteractionEvent::Focus => {
                self.event_queue.emit_owned(CheckboxEvent::Focus);
            }
            ui::InteractionEvent::Blur => {
                self.event_queue.emit_owned(CheckboxEvent::Blur);
            }
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Checkbox {
    pub foreground: Color,
    pub background: Color,
    pub focus: Color,
    pub contrast: draw::ThemeContrast,
    pub checked: bool,
    pub disabled: bool,
}

impl Checkbox {
    pub fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        Checkbox {
            foreground: data.scheme.over_control_inset,
            background: data.scheme.control_inset,
            focus: data.scheme.focus,
            contrast: data.contrast,
            checked: false,
            disabled: false,
        }
    }

    pub fn construct<U, G>(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        _g_aux: &mut G,
    ) -> CheckboxWidget<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        let mut pipe = pipeline! {
            CheckboxWidget<U, G> as obj,
            U as _aux,
            _ev in &data.on_change => { change { obj.command_group.repaint(); } }
        };

        pipe = pipe.add(
            ui::basic_interaction_terminal::<CheckboxWidget<U, G>, U>().bind(u_aux.window_queue()),
        );

        let painter = theme.checkbox();
        let rect = RelativeRect::new(
            Default::default(),
            painter
                .size_hint(state::CheckboxState {
                    rect: Default::default(),
                    data: data.clone(),
                    interaction: state::InteractionState::empty(),
                })
                .cast_unit(),
        );

        CheckboxWidget {
            event_queue: Default::default(),
            data,

            pipe: pipe.into(),
            painter,
            parent_position: Default::default(),

            rect,
            command_group: Default::default(),
            layout: Default::default(),
            visibility: Default::default(),
            interaction: state::InteractionState::empty(),
            drop_event: Default::default(),

            phantom_g: Default::default(),
        }
    }
}

impl<U, G> CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.abs_rect());
    }

    fn derive_state(&self) -> state::CheckboxState {
        state::CheckboxState {
            rect: self.abs_rect(),
            data: self.data.clone(),
            interaction: self.interaction,
        }
    }
}

impl<U, G> Widget for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect).cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if was_focused != self.interaction.contains(state::InteractionState::FOCUSED) {
            self.command_group.repaint();
            self.event_queue.emit_owned(if !was_focused {
                CheckboxEvent::Focus
            } else {
                CheckboxEvent::Blur
            });
        }

        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(state), None, None);
    }
}

impl<U, G> StoresParentPosition for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint) {
        self.parent_position = parent_pos;
        self.on_transform();
    }

    fn parent_position(&self) -> AbsolutePoint {
        self.parent_position
    }
}

impl<U, G> draw::HasTheme for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {
        self.set_size(self.painter.size_hint(self.derive_state()));
    }
}

impl<U, G> Drop for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
