use {
    crate::{
        base::{self, Repaintable, Resizable},
        draw,
    },
    reclutch::{
        display::{
            center_horizontally, CommandGroup, DisplayCommand, DisplayListBuilder, DisplayText,
            FontInfo, GraphicsDisplay, Point, Rect, ResourceReference, Size, StyleColor,
            TextDisplayItem,
        },
        event::{RcEventListener, RcEventQueue},
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
#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
pub struct Label<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub text: base::Observed<DisplayText>,
    pub size: base::Observed<f32>,
    pub color: base::Observed<StyleColor>,
    pub font: base::Observed<(ResourceReference, FontInfo)>,
    pub align: base::Observed<TextAlign>,
    rect: base::Observed<Rect>,

    repaint_listeners: Vec<RcEventListener<()>>,
    recolor_listener: RcEventListener<()>,
    command_group: CommandGroup,
    text_items: Vec<TextDisplayItem>,
    layout: base::WidgetLayoutEvents,
    visibility: base::Visibility,
    drop_event: RcEventQueue<()>,

    themed: draw::PhantomThemed,
    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Label<U, G> {
    /// Creates a new label widget.
    pub fn new(
        size: Option<f32>,
        color: Option<StyleColor>,
        align: Option<TextAlign>,
        rect: Rect,
        text: DisplayText,
        theme: &dyn draw::Theme,
        g_aux: &mut G,
    ) -> Self {
        let text = base::Observed::new(text);
        let size = base::Observed::new(size.unwrap_or(theme.default_text_size()));
        let color = base::Observed::new(color.unwrap_or(theme.label_color()));
        let font = base::Observed::new(g_aux.ui_font());
        let align = base::Observed::new(align.unwrap_or(TextAlign::Left));
        let mut rect = base::Observed::new(rect);

        let recolor_listener = color.on_change.listen();

        let repaint_listeners = vec![
            text.on_change.listen(),
            size.on_change.listen(),
            font.on_change.listen(),
            align.on_change.listen(),
            rect.on_change.listen(),
        ];

        let (text_items, height) = Self::create_text_items(
            *rect.get(),
            text.get().clone(),
            color.get().clone(),
            *align.get(),
            (font.get().0.clone(), font.get().1.clone()),
            *size.get(),
        );

        rect.get_mut().size.height = height;

        Label {
            text,
            size,
            color,
            font,
            align,
            rect,

            repaint_listeners,
            recolor_listener,
            command_group: Default::default(),
            text_items,
            layout: Default::default(),
            visibility: Default::default(),
            drop_event: Default::default(),

            themed: Default::default(),
            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    fn create_text_items(
        rect: Rect,
        text: DisplayText,
        color: StyleColor,
        align: TextAlign,
        font: (ResourceReference, FontInfo),
        size: f32,
    ) -> (Vec<TextDisplayItem>, f32) {
        let mut text = TextDisplayItem {
            text,
            font: font.0,
            font_info: font.1.clone(),
            size,
            bottom_left: Default::default(),
            color,
        };

        text.set_top_left(rect.origin);

        let metrics = font.1.font.metrics();
        let mut text_items = text
            .linebreak(
                rect,
                (metrics.ascent + metrics.line_gap) / metrics.units_per_em as f32 * size,
                true,
            )
            .unwrap();

        let mut max_y = rect.origin.y;
        for text_item in &mut text_items {
            let bounds = text_item.bounds().unwrap();
            let left = match align {
                TextAlign::Left => text_item.bottom_left.x,
                TextAlign::Middle => center_horizontally(bounds, rect).x,
                TextAlign::Right => rect.max_x() - bounds.size.width,
            };

            if bounds.max_y() > max_y {
                max_y = bounds.max_y();
            }

            text_item.bottom_left.x = left;
        }

        (text_items, max_y - rect.origin.y)
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for Label<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    #[inline]
    fn bounds(&self) -> Rect {
        *self.rect.get()
    }

    fn update(&mut self, _aux: &mut U) {
        let mut dirty = false;

        if let Some(rect) = self.layout.receive() {
            self.rect.set(rect);
            dirty = true;
        }

        for rl in &mut self.repaint_listeners {
            if !rl.peek().is_empty() {
                dirty = true;
                break;
            }
        }

        if !self.recolor_listener.peek().is_empty() && !dirty {
            for item in &mut self.text_items {
                item.color = self.color.get().clone();
            }
        }

        if dirty {
            let (text_items, height) = Self::create_text_items(
                *self.rect.get(),
                self.text.get().clone(),
                self.color.get().clone(),
                *self.align.get(),
                (self.font.get().0.clone(), self.font.get().1.clone()),
                *self.size.get(),
            );

            self.text_items = text_items;
            self.set_size(Size::new(self.size().width, height));
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let mut builder = DisplayListBuilder::new();

        builder.push_rectangle_clip(*self.rect.get(), true);

        for text_item in &self.text_items {
            builder.push_text(text_item.clone(), None);
        }

        self.command_group
            .push(display, &builder.build(), None, None);
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::LayableWidget for Label<U, G> {
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::HasVisibility for Label<U, G> {
    #[inline]
    fn set_visibility(&mut self, visibility: base::Visibility) {
        self.visibility = visibility
    }

    #[inline]
    fn visibility(&self) -> base::Visibility {
        self.visibility
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Repaintable for Label<U, G> {
    #[inline]
    fn repaint(&mut self) {
        self.command_group.repaint();
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Movable for Label<U, G> {
    fn set_position(&mut self, position: Point) {
        self.rect.get_mut().origin = position;
        self.repaint();
        self.layout.notify(*self.rect.get());
    }

    #[inline]
    fn position(&self) -> Point {
        self.rect.get().origin
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Resizable for Label<U, G> {
    fn set_size(&mut self, size: Size) {
        self.rect.get_mut().size = size;
        self.repaint();
        self.layout.notify(*self.rect.get());
    }

    #[inline]
    fn size(&self) -> Size {
        self.rect.get().size
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> draw::HasTheme for Label<U, G> {
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::DropEvent for Label<U, G> {
    #[inline(always)]
    fn drop_event(&self) -> &RcEventQueue<()> {
        &self.drop_event
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Drop for Label<U, G> {
    fn drop(&mut self) {
        self.drop_event.emit_owned(());
    }
}
