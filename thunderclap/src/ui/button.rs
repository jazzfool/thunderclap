//! Button control widget.

use {
    crate::{
        base::{self, Repaintable},
        draw::{self, state, HasTheme},
        geom::*,
        ui,
    },
    reclutch::{
        display::{Color, DisplayCommand, DisplayText, GraphicsDisplay, Rect},
        prelude::*,
        verbgraph as vg,
        widget::Widget,
    },
};

/// Events emitted by a button.
#[derive(Event, Debug, Clone, Copy, PartialEq)]
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
            _ => return,
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

impl<U, G> ui::WidgetConstructor<U, G> for Button
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(theme: &dyn draw::Theme) -> Self {
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

    fn construct(self, theme: &dyn draw::Theme, u_aux: &mut U) -> ButtonWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        let mut graph = vg::verbgraph! {
            ButtonWidget<U, G> as obj,
            U as _aux,
            "bind" => _ev in &data.on_change => {
                change => {
                    obj.resize_from_theme();
                    obj.command_group.repaint();
                }
            }
        };

        graph = graph.add(
            "interaction",
            ui::basic_interaction_handler::<ButtonWidget<U, G>, U>().bind(u_aux.window_queue()),
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

        ButtonWidgetBuilder {
            rect,
            graph: graph.into(),

            data,
            painter,

            interaction: state::InteractionState::empty(),
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<state::ButtonState> for ButtonWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) -> state::ButtonState {
        state::ButtonState {
            rect: self.abs_rect(),
            data: self.data.clone(),
            interaction: self.interaction,
        }
    }

    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.abs_rect());
    }
}

use crate as thunderclap;
crate::widget! {
    pub struct ButtonWidget {
        widget::MAX,

        <ButtonEvent> EventQueue,
        <Button> State,
        <state::ButtonState> Painter,

        {
            interaction: state::InteractionState,
        },
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
        let mut graph = self.graph.take().unwrap();
        graph.update_all(self, aux);
        self.graph = Some(graph);

        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let button_state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(
            display,
            || painter.draw(button_state),
            Default::default(),
            None,
            None,
        );
    }
}
