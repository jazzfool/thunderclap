use {
    super::Align,
    crate::{base, draw},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Point, Rect, Size},
        event::bidir_single::Queue as BidirSingleEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct VStackData {
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub alignment: Align,
}

impl VStackData {
    pub fn top_margin(self, top_margin: f32) -> VStackData {
        VStackData { top_margin, ..self }
    }

    pub fn bottom_margin(self, bottom_margin: f32) -> VStackData {
        VStackData {
            bottom_margin,
            ..self
        }
    }

    pub fn align(self, alignment: Align) -> VStackData {
        VStackData { alignment, ..self }
    }
}

#[derive(Debug)]
struct ChildData {
    data: VStackData,
    evq: BidirSingleEventQueue<Rect, Rect>,
    rect: Rect,
    original_rect: Rect,
}

#[derive(WidgetChildren, Debug)]
#[widget_children_trait(base::WidgetChildren)]
pub struct VStack<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    rect: Rect,
    rects: IndexMap<u64, ChildData>,
    next_rect_id: u64,
    dirty: bool,

    themed: draw::PhantomThemed,
    layout: base::WidgetLayoutEvents,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> VStack<U, G> {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,

            themed: Default::default(),
            layout: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Layout for VStack<U, G> {
    type PushData = VStackData;

    /// "Registers" a widget to the layout.
    fn push(&mut self, data: Self::PushData, child: &mut impl base::LayableWidget) {
        self.dirty = true;

        let id = self.next_rect_id;
        self.next_rect_id += 1;

        let evq = BidirSingleEventQueue::new();

        child.listen_to_layout(base::WidgetLayoutEventsInner {
            id,
            evq: evq.secondary(),
        });

        let rect = child.rect();

        self.rects.insert(
            id,
            ChildData {
                data,
                evq,
                rect,
                original_rect: rect,
            },
        );
    }

    /// De-registers a widget from the layout, optionally restoring the original widget rectangle.
    fn remove(&mut self, child: &mut impl base::LayableWidget, restore_original: bool) {
        if let Some(data) = child.layout_id().and_then(|id| self.rects.remove(&id)) {
            child.listen_to_layout(None);
            if restore_original {
                child.set_rect(data.original_rect);
            }
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for VStack<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect
    }

    fn update(&mut self, _aux: &mut U) {
        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.dirty = true;
        }

        {
            let dirty = &mut self.dirty;
            for (_, data) in &mut self.rects {
                if let Some(new_ev) = data.evq.retrieve_newest() {
                    *dirty = true;
                    data.rect = new_ev;
                }
            }
        }

        if self.dirty {
            let mut advance = self.rect.origin.y;
            for (_, data) in &mut self.rects {
                advance += data.data.top_margin;

                let mut rect = data.rect;
                rect.origin.y = advance;
                rect.origin.x = match data.data.alignment {
                    Align::Begin => self.rect.origin.x,
                    Align::Middle => display::center_horizontally(rect, self.rect).x,
                    Align::End => self.rect.origin.x + self.rect.size.width - rect.size.width,
                    Align::Stretch => {
                        rect.size.width = self.rect.size.width;
                        self.rect.origin.x
                    }
                };

                data.evq.emit_owned(rect);
                data.rect = rect;

                advance += rect.size.height + data.data.bottom_margin;
            }

            self.dirty = false;
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::LayableWidget for VStack<U, G> {
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Repaintable for VStack<U, G> {
    #[inline]
    fn repaint(&mut self) {}
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Movable for VStack<U, G> {
    #[inline]
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
    }

    #[inline]
    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Resizable for VStack<U, G> {
    #[inline]
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
    }

    #[inline]
    fn size(&self) -> Size {
        self.rect.size
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> draw::HasTheme for VStack<U, G> {
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    #[inline]
    fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
}
