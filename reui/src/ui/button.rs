//! Button control widget.

use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state, HasTheme},
        geom::*,
        pipe, ui,
    },
    reclutch::{
        display::{Color, CommandGroup, DisplayCommand, DisplayText, GraphicsDisplay, Rect},
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
    Press(AbsolutePoint),
    /// Emitted when the checkbox is released.
    #[event_key(release)]
    Release(AbsolutePoint),
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

/// Focus-able button widget.
#[derive(
    WidgetChildren, LayableWidget, DropNotifier, HasVisibility, Repaintable, Movable, Resizable,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub event_queue: RcEventQueue<ButtonEvent>,

    pub data: base::Observed<Button>,
    pipe: Option<pipe::Pipeline<Self, U>>,
    interaction: state::InteractionState,
    painter: Box<dyn draw::Painter<state::ButtonState>>,
    parent_position: AbsolutePoint,

    #[widget_rect]
    rect: RelativeRect,
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

impl<U, G> ui::InteractiveWidget for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
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
pub struct Button {
    pub text: DisplayText,
    pub typeface: draw::TypefaceStyle,
    pub color: Color,
    pub background: Color,
    pub focus: Color,
    pub contrast: draw::ThemeContrast,
    pub disabled: bool,
}

impl<U, G> ui::WidgetDataTarget<U, G> for Button
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = ButtonWidget<U, G>;
}

impl Button {
    pub fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        Button {
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
    ) -> ButtonWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        let mut pipe = pipeline! {
            ButtonWidget<U, G> as obj,
            U as _aux,
            _ev in &data.on_change => {
                change {
                    obj.resize_from_theme();
                    obj.command_group.repaint();
                }
            }
        };

        pipe = pipe.add(
            ui::basic_interaction_terminal::<ButtonWidget<U, G>, U>().bind(u_aux.window_queue()),
        );

        let painter = theme.button();
        let rect = RelativeRect::new(
            Default::default(),
            painter
                .size_hint(state::ButtonState {
                    rect: Default::default(),
                    data: data.clone(),
                    interaction: state::InteractionState::empty(),
                })
                .cast_unit(),
        );

        ButtonWidget {
            event_queue: Default::default(),
            data,
            pipe: pipe.into(),
            interaction: state::InteractionState::empty(),
            painter,
            parent_position: Default::default(),
            rect,
            visibility: Default::default(),
            command_group: Default::default(),
            layout: Default::default(),
            drop_event: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U, G> ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.abs_rect());
    }

    fn derive_state(&self) -> state::ButtonState {
        state::ButtonState {
            rect: self.abs_rect(),
            data: self.data.clone(),
            interaction: self.interaction,
        }
    }
}

impl<U, G> Widget for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect).cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        let button_state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(button_state), None, None);
    }
}

impl<U, G> ui::Bindable<U> for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn perform_bind(&mut self, _aux: &mut U) {
        self.repaint();
    }
}

impl<U, G> StoresParentPosition for ButtonWidget<U, G>
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

impl<U, G> HasTheme for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {
        self.set_size(self.painter.size_hint(self.derive_state()));
    }
}

impl<U, G> ui::DefaultEventQueue<ButtonEvent> for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn default_event_queue(&self) -> &RcEventQueue<ButtonEvent> {
        &self.event_queue
    }
}

impl<U, G> ui::DefaultWidgetData<Button> for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn default_data(&mut self) -> &mut base::Observed<Button> {
        &mut self.data
    }
}

impl<U, G> Drop for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
