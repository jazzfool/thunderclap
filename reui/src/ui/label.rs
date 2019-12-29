use {
    crate::{
        base::{self, Repaintable},
        draw, pipe,
    },
    reclutch::{
        display::{
            center_horizontally, Color, CommandGroup, DisplayCommand, DisplayListBuilder,
            DisplayText, FontInfo, GraphicsDisplay, Rect, ResourceReference, Size, TextDisplayItem,
        },
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Horizontal alignment of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAlign {
    Left,
    Middle,
    Right,
}

/// Label widget which displays text wrapped and clipped within a rectangle.
#[derive(
    WidgetChildren, LayableWidget, HasVisibility, Repaintable, Movable, Resizable, DropNotifier,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct Label<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub data: base::Observed<LabelData>,

    pipe: Option<pipe::Pipeline<Self, U>>,
    text_items: Vec<TextDisplayItem>,
    previous_rect: Rect,

    #[widget_rect]
    rect: Rect,
    #[repaint_target]
    command_group: CommandGroup,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,
    #[widget_visibility]
    visibility: base::Visibility,
    #[widget_drop_event]
    drop_event: RcEventQueue<base::DropEvent>,

    themed: draw::PhantomThemed,
    phantom_g: PhantomData<G>,
}

pub struct LabelData {
    pub text: DisplayText,
    pub typeface: draw::TypefaceStyle,
    pub color: Color,
    pub align: TextAlign,
    pub wrap: bool,
}

impl LabelData {
    pub fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        LabelData {
            text: "".to_string().into(),
            typeface: data.typography.body.clone(),
            color: data.scheme.over_primary,
            align: TextAlign::Left,
            wrap: true,
        }
    }

    pub fn construct<U, G>(self, _: &dyn draw::Theme, _u_aux: &mut U, _g_aux: &mut G) -> Label<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        let (text_items, rect) = Label::<U, G>::create_text_items(
            Rect::new(Default::default(), Size::new(std::f32::MAX, 0.0)),
            data.text.clone(),
            data.color.into(),
            data.align,
            data.typeface.typeface.pick(data.typeface.style),
            data.typeface.size,
            data.wrap,
        );

        let pipe = pipeline! {
            Label<U, G> as obj,
            U as _aux,
            _ev in &data.on_change => {
                change {
                    obj.update_text_items();
                }
            }
        };

        Label {
            data,

            pipe: pipe.into(),
            text_items,
            previous_rect: rect,

            rect,
            command_group: Default::default(),
            layout: Default::default(),
            visibility: Default::default(),
            drop_event: Default::default(),

            themed: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Label<U, G> {
    fn on_transform(&mut self) {
        if self.previous_rect.size.width != self.rect.size.width {
            self.update_text_items();
        } else if self.previous_rect.origin != self.rect.origin {
            let diff = self.rect.origin - self.previous_rect.origin;
            for item in &mut self.text_items {
                item.bottom_left += diff;
            }
        }

        self.previous_rect = self.rect;
        self.layout.notify(self.rect);
        self.repaint();
    }

    fn create_text_items(
        rect: Rect,
        text: DisplayText,
        color: Color,
        align: TextAlign,
        font: (ResourceReference, FontInfo),
        size: f32,
        wrap: bool,
    ) -> (Vec<TextDisplayItem>, Rect) {
        let mut text = TextDisplayItem {
            text,
            font: font.0,
            font_info: font.1.clone(),
            size,
            bottom_left: Default::default(),
            color: color.into(),
        };

        text.set_top_left(rect.origin);

        let metrics = font.1.font.metrics();
        let mut text_items = if wrap {
            text.linebreak(
                rect,
                (metrics.ascent + metrics.line_gap) / metrics.units_per_em as f32 * size,
                true,
            )
            .unwrap()
        } else {
            vec![text]
        };

        let mut total_bounds: Option<Rect> = None;
        for text_item in &mut text_items {
            let bounds = text_item.bounds().unwrap();
            if let Some(ref mut total_bounds) = total_bounds {
                *total_bounds = total_bounds.union(&bounds);
            } else {
                total_bounds = Some(bounds);
            }
            let left = match align {
                TextAlign::Left => text_item.bottom_left.x,
                TextAlign::Middle => center_horizontally(bounds, rect).x,
                TextAlign::Right => rect.max_x() - bounds.size.width,
            };
            text_item.bottom_left.x = left;
        }

        (text_items, total_bounds.unwrap_or_default())
    }

    fn update_text_items(&mut self) {
        let (text_items, bounds) = Self::create_text_items(
            self.rect,
            self.data.text.clone(),
            self.data.color.into(),
            self.data.align,
            self.data.typeface.typeface.pick(self.data.typeface.style),
            self.data.typeface.size,
            self.data.wrap,
        );

        self.text_items = text_items;
        self.rect = bounds;
        self.layout.notify(self.rect);
        self.repaint();
    }
}

impl<U, G> Widget for Label<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        self.rect
    }

    fn update(&mut self, aux: &mut U) {
        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let mut builder = DisplayListBuilder::new();
        builder.push_rectangle_clip(self.rect, true);
        for text_item in &self.text_items {
            builder.push_text(text_item.clone(), None);
        }
        self.command_group.push(display, &builder.build(), None, None);
    }
}

impl<U, G> draw::HasTheme for Label<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    fn resize_from_theme(&mut self) {}
}

impl<U, G> Drop for Label<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
