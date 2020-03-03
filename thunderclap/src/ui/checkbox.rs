use {
    crate::{
        base::{self, Repaintable},
        draw::{self, state},
        geom::*,
        ui,
    },
    reclutch::{
        display::{Color, DisplayCommand, GraphicsDisplay, Rect},
        prelude::*,
        verbgraph as vg,
    },
};

/// Events emitted by a checkbox.
#[derive(Event, Debug, Clone, Copy, PartialEq)]
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

impl<U, G> ui::InteractiveWidget for CheckboxWidget<U, G>
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

impl<U, G> ui::WidgetDataTarget<U, G> for Checkbox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = CheckboxWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for Checkbox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(theme: &dyn draw::Theme) -> Self {
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

    fn construct(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        _g_aux: &mut G,
    ) -> CheckboxWidget<U, G> {
        let data = base::Observed::new(self);

        let mut graph = vg::verbgraph! {
            CheckboxWidget<U, G> as obj,
            U as _aux,
            "bind" => _ev in &data.on_change => { change => { obj.command_group.repaint(); } }
        };

        graph = graph.add(
            "handler",
            ui::basic_interaction_handler::<CheckboxWidget<U, G>, U>().bind(u_aux.window_queue()),
        );

        let painter = theme.checkbox();
        let rect = RelativeRect::new(
            Default::default(),
            painter
                .size_hint(state::CheckboxState {
                    rect: Default::default(),
                    data: *data,
                    interaction: state::InteractionState::empty(),
                })
                .cast_unit(),
        );

        CheckboxWidgetBuilder {
            rect,

            graph: graph.into(),

            data,
            painter,

            interaction: state::InteractionState::empty(),
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<state::CheckboxState> for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) -> state::CheckboxState {
        state::CheckboxState {
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
    pub struct CheckboxWidget {
        widget::MAX,

        <CheckboxEvent> EventQueue,
        <Checkbox> State,
        <state::CheckboxState> Painter,

        {
            interaction: state::InteractionState,
        },
    }
}

impl<U, G> Widget for CheckboxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.painter.paint_hint(self.rect).cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        let was_focused = self.interaction.contains(state::InteractionState::FOCUSED);

        let mut graph = self.graph.take().unwrap();
        graph.update_all(self, aux);
        self.graph = Some(graph);

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
        self.command_group.push_with(
            display,
            || painter.draw(state),
            Default::default(),
            None,
            None,
        );
    }
}
