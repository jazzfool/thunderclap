use {
    crate::{
        base::{self, Repaintable},
        draw,
        geom::*,
        ui,
    },
    reclutch::{
        display::{
            center_horizontally, Color, DisplayCommand, DisplayListBuilder, DisplayText,
            GraphicsDisplay, Rect, TextDisplayItem,
        },
        prelude::*,
        verbgraph as vg,
    },
};

/// Horizontal alignment of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAlign {
    Left,
    Middle,
    Right,
}

pub struct Label {
    pub text: DisplayText,
    pub typeface: draw::TypefaceStyle,
    pub color: Color,
    pub align: TextAlign,
    pub wrap: bool,
}

impl<U, G> ui::WidgetDataTarget<U, G> for Label
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = LabelWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for Label
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        Label {
            text: "".to_string().into(),
            typeface: data.typography.body.clone(),
            color: data.scheme.over_control_outset,
            align: TextAlign::Left,
            wrap: true,
        }
    }

    fn construct(
        self,
        _theme: &dyn draw::Theme,
        _u_aux: &mut U,
    ) -> LabelWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        let graph = vg::verbgraph! {
            LabelWidget<U, G> as obj,
            U as _aux,
            "bind" => _ev in &data.on_change => {
                change => {
                    obj.update_text_items();
                    obj.repaint();
                }
            }
        };

        let mut label = LabelWidgetBuilder {
            rect: Default::default(),
            graph: graph.into(),
            data,

            text_items: Vec::new(),
            previous_rect: Default::default(),
            dirty: true,
        }
        .build();

        label.update_text_items();
        label.previous_rect = label.rect;

        label
    }
}

impl<U, G> ui::core::CoreWidget<()> for LabelWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) {}

    fn on_transform(&mut self) {
        if (self.previous_rect.size.width - self.rect.size.width).abs() > std::f32::EPSILON {
            self.dirty = true;
        } else if self.previous_rect.origin != self.rect.origin {
            let diff = self.rect.origin - self.previous_rect.origin;
            for item in &mut self.text_items {
                item.bottom_left += diff.cast_unit();
            }
        }

        self.previous_rect = self.rect;
        self.layout.notify(self.abs_rect());
        self.repaint();
    }
}

use crate as thunderclap;
crate::widget! {
    pub struct LabelWidget {
        widget::MAX,

        <Label> State,

        {
            text_items: Vec<TextDisplayItem>,
            previous_rect: RelativeRect,
            dirty: bool,
        },
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> LabelWidget<U, G> {
    fn update_text_items(&mut self) {
        let font = self.data.typeface.typeface.pick(self.data.typeface.style);

        let mut text = TextDisplayItem {
            text: self.data.text.clone(),
            font: font.0,
            font_info: font.1.clone(),
            size: self.data.typeface.size,
            bottom_left: Default::default(),
            color: self.data.color.into(),
        };

        text.set_top_left(self.abs_rect().origin.cast_unit());

        let metrics = font.1.font.metrics();
        let mut text_items = if self.data.wrap {
            text.linebreak(
                self.abs_rect().size.width,
                (metrics.ascent + metrics.line_gap) / metrics.units_per_em as f32
                    * self.data.typeface.size,
                true,
            )
            .unwrap()
        } else {
            vec![text]
        };

        let mut total_bounds: Option<AbsoluteRect> = None;
        for text_item in &mut text_items {
            let bounds = text_item.bounds().unwrap().cast_unit();
            if let Some(ref mut total_bounds) = total_bounds {
                *total_bounds = total_bounds.union(&bounds);
            } else {
                total_bounds = Some(bounds);
            }
            let left = match self.data.align {
                TextAlign::Left => text_item.bottom_left.x,
                TextAlign::Middle => {
                    center_horizontally(bounds.cast_unit(), self.abs_rect().cast_unit()).x
                }
                TextAlign::Right => self.abs_rect().max_x() - bounds.size.width,
            };
            text_item.bottom_left.x = left;
        }

        self.text_items = text_items;
        self.set_ctxt_rect(total_bounds.unwrap_or_default());
    }
}

impl<U, G> Widget for LabelWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        self.rect.cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
            self.dirty = true;
        }

        if self.dirty {
            self.dirty = false;
            self.update_text_items();
        }

        let mut graph = self.graph.take().unwrap();
        graph.update_all(self, aux);
        self.graph = Some(graph);
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let mut builder = DisplayListBuilder::new();
        builder.push_rectangle_clip(self.abs_rect().cast_unit(), true);
        for text_item in &self.text_items {
            builder.push_text(text_item.clone(), None);
        }
        self.command_group.push(display, &builder.build(), Default::default(), None, None);
    }
}
