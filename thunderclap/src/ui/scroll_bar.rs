use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw::{self, state, HasTheme},
        geom::*,
        ui,
    },
    reclutch::{
        display::{Color, CommandGroup, DisplayCommand, GraphicsDisplay, Rect, Size},
        event::RcEventQueue,
        prelude::*,
        verbgraph as vg,
        widget::Widget,
    },
    std::marker::PhantomData,
};

/// Information about how far a scroll bar has been scrolled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPosition {
    /// The simple progression of the scrolling, from 0.0 to 1.0,
    /// This is the most useful position, and typically the one you want.
    pub amount: f32,
    /// The range (0.0 to 1.0) of the physical scroll bar.
    /// For example, 0.25..0.5 means that the top of the scroll bar is
    /// a quarter way down, and the bottom is halfway down.
    pub amount_range: (f32, f32),
}

#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub enum ScrollBarEvent {
    #[event_key(begin_scroll)]
    BeginScroll,
    #[event_key(end_scroll)]
    EndScroll,
    #[event_key(scroll)]
    Scroll(ScrollPosition),
}

/// A simple scroll bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBar {
    /// Whether changing the width has any effect on the drawn size.
    pub lock_width: bool,
    /// Length of the content.
    pub document_length: f32,
    /// Length of a single page of content.
    pub page_length: f32,
    /// Color of the scroll track.
    pub background: Color,
    /// Color of the scroll bar.
    pub foreground: Color,
    /// Color contrast.
    pub contrast: draw::ThemeContrast,
}

impl<U, G> ui::WidgetDataTarget<U, G> for ScrollBar
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = ScrollBarWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for ScrollBar
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        ScrollBar {
            lock_width: true,
            document_length: 1.0,
            page_length: 1.0,
            background: data.scheme.control_inset,
            foreground: data.scheme.over_control_inset,
            contrast: data.contrast,
        }
    }

    fn construct(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        g_aux: &mut G,
    ) -> ScrollBarWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        let mut graph = vg::verbgraph! {
            ScrollBarWidget<U, G> as obj,
            U as _aux,
        };

        let painter = theme.scroll_bar();
        let rect = RelativeRect::new(
            Default::default(),
            painter
                .size_hint(state::ScrollBarState {
                    rect: AbsoluteRect::new(Default::default(), Size::new(10.0, 100.0).cast_unit()),
                    data: data.clone(),
                    scroll_bar: Default::default(),
                    interaction: state::InteractionState::empty(),
                })
                .cast_unit(),
        );

        ScrollBarWidgetBuilder {
            rect,
            graph: graph.into(),
            data,
            painter,

            scroll_position: ScrollPosition { amount: 0.0, amount_range: (0.0, 0.0) },
            locked_width: rect.size.width,
            interaction: state::InteractionState::empty(),
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<state::ScrollBarState> for ScrollBarWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) -> state::ScrollBarState {
        let abs_rect = self.abs_rect();

        state::ScrollBarState {
            rect: abs_rect,
            data: self.data.clone(),
            scroll_bar: AbsoluteRect::new(
                AbsolutePoint::new(
                    abs_rect.origin.x,
                    abs_rect.origin.y
                        + (abs_rect.size.height * self.scroll_position.amount_range.0),
                ),
                Size::new(
                    abs_rect.size.width,
                    abs_rect.size.height * self.scroll_position.amount_range.0,
                )
                .cast_unit(),
            ),
            interaction: self.interaction,
        }
        .into()
    }

    fn on_transform(&mut self) {
        if self.data.lock_width {
            self.rect.size.width = self.locked_width;
        } else {
            self.locked_width = self.rect.size.width;
        }

        self.repaint();
        self.layout.notify(self.abs_rect());
    }
}

use crate as thunderclap;
crate::widget! {
    pub struct ScrollBarWidget {
        widget::MAX,

        <ScrollBarEvent> EventQueue,
        <ScrollBar> State,
        <state::ScrollBarState> Painter,

        {
            scroll_position: ScrollPosition,
            locked_width: f32,

            interaction: state::InteractionState,
        }
    }
}

impl<U, G> Widget for ScrollBarWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect.cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        let mut graph = self.graph.take().unwrap();
        graph.update_all(self, aux);
        self.graph = Some(graph);
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
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
