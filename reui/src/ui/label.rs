use {
    crate::{
        base::{self, Repaintable},
        draw, pipe,
    },
    reclutch::{
        display::{
            center_horizontally, CommandGroup, DisplayCommand, DisplayListBuilder, DisplayText,
            FontInfo, GraphicsDisplay, Rect, ResourceReference, StyleColor, TextDisplayItem,
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
pub struct Label<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub text: base::Observed<DisplayText>,
    pub size: base::Observed<f32>,
    pub color: base::Observed<StyleColor>,
    pub font: base::Observed<(ResourceReference, FontInfo)>,
    pub align: base::Observed<TextAlign>,
    pub wrap: base::Observed<bool>,
    pipe: Option<pipe::Pipeline<Self, U>>,
    text_items: Vec<TextDisplayItem>,

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
        wrap: bool,
        theme: &dyn draw::Theme,
        g_aux: &mut G,
    ) -> Self {
        let size = base::Observed::new(size.unwrap_or(theme.default_text_size()));
        let color = base::Observed::new(color.unwrap_or(theme.label_color()));
        let font = base::Observed::new(g_aux.ui_font());
        let align = base::Observed::new(align.unwrap_or(TextAlign::Left));
        observe![text, wrap];

        let text_items = Self::create_text_items(
            rect,
            text.get().clone(),
            color.get().clone(),
            *align.get(),
            (font.get().0.clone(), font.get().1.clone()),
            *size.get(),
            *wrap.get(),
        );

        let pipe = pipeline! {
            Self as obj,
            U as _aux,
            _ev in &text.on_change => { change {
                obj.update_text_items();
            }}
            _ev in &size.on_change => { change {
                obj.update_text_items();
            }}
            _ev in &color.on_change => { change {
                for item in &mut obj.text_items {
                    item.color = obj.color.get().clone();
                }
                obj.repaint();
            }}
            _ev in &font.on_change => { change {
                obj.update_text_items();
            }}
            _ev in &align.on_change => { change {
                obj.update_text_items();
            }}
            _ev in &wrap.on_change => { change {
                obj.update_text_items();
            }}
        };

        Label {
            text,
            size,
            color,
            font,
            align,
            wrap,
            rect,
            pipe: pipe.into(),
            text_items,

            command_group: Default::default(),
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
        wrap: bool,
    ) -> Vec<TextDisplayItem> {
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

        for text_item in &mut text_items {
            let bounds = text_item.bounds().unwrap();
            let left = match align {
                TextAlign::Left => text_item.bottom_left.x,
                TextAlign::Middle => center_horizontally(bounds, rect).x,
                TextAlign::Right => rect.max_x() - bounds.size.width,
            };

            text_item.bottom_left.x = left;
        }

        text_items
    }

    fn update_text_items(&mut self) {
        let text_items = Self::create_text_items(
            self.rect,
            self.text.get().clone(),
            self.color.get().clone(),
            *self.align.get(),
            (self.font.get().0.clone(), self.font.get().1.clone()),
            *self.size.get(),
            *self.wrap.get(),
        );

        self.text_items = text_items;
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
