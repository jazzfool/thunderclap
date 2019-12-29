//! Button control widget.

use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state, HasTheme},
        pipe, ui,
    },
    reclutch::{
        display::{Color, CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Point, Rect},
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

    pub data: base::Observed<ButtonData>,
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

impl<U, G> ui::InteractiveWidget for Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline]
    fn mouse_bounds(&self) -> Rect {
        self.painter.mouse_hint(self.rect)
    }

    #[inline(always)]
    fn disabled(&self) -> bool {
        self.data.disabled
    }

    fn on_interaction_event(&mut self, event: ui::InteractionEvent) {
        self.repaint();
        self.event_queue.emit_owned(match event {
            ui::InteractionEvent::Pressed(pos) => ButtonEvent::Press(pos),
            ui::InteractionEvent::Released(pos) => ButtonEvent::Release(pos),
            ui::InteractionEvent::BeginHover(pos) => ButtonEvent::BeginHover(pos),
            ui::InteractionEvent::EndHover(pos) => ButtonEvent::EndHover(pos),
            ui::InteractionEvent::Focus => ButtonEvent::Focus,
            ui::InteractionEvent::Blur => ButtonEvent::Blur,
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonData {
    pub text: DisplayText,
    pub typeface: draw::TypefaceStyle,
    pub color: Color,
    pub background: Color,
    pub focus: Color,
    pub contrast: draw::ThemeContrast,
    pub disabled: bool,
}

impl ButtonData {
    pub fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        ButtonData {
            text: "".to_string().into(),
            typeface: data.typography.button.clone(),
            color: data.scheme.over_control_outset,
            background: data.scheme.control_outset,
            focus: data.scheme.focus,
            contrast: data.contrast,
            disabled: false,
        }
    }

    pub fn construct<U, G>(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        _g_aux: &mut G,
    ) -> Button<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        let mut pipe = pipeline! {
            Button<U, G> as obj,
            U as _aux,
            _ev in &data.on_change => {
                change {
                    obj.resize_from_theme();
                    obj.command_group.repaint();
                }
            }
        };

        pipe = pipe
            .add(ui::basic_interaction_terminal::<Button<U, G>, U>().bind(u_aux.window_queue()));

        let painter = theme.button();
        let rect = Rect::new(
            Default::default(),
            painter.size_hint(state::ButtonState {
                rect: Default::default(),
                data: data.clone(),
                interaction: state::InteractionState::empty(),
            }),
        );

        Button {
            event_queue: Default::default(),
            data,
            pipe: pipe.into(),
            interaction: state::InteractionState::empty(),
            painter,
            rect,
            visibility: Default::default(),
            command_group: Default::default(),
            layout: Default::default(),
            drop_event: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U, G> Button<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.rect);
    }

    fn derive_state(&self, tracer: &base::AdditiveTracer) -> state::ButtonState {
        state::ButtonState {
            rect: tracer.absolute_bounds(self.rect),
            data: self.data.clone(),
            interaction: self.interaction,
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
        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let button_state = self.derive_state(aux.tracer());
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
        self.set_size(self.painter.size_hint(self.derive_state(&Default::default())));
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
