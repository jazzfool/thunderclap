use {
    super::Align,
    crate::{base, draw},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Point, Rect, Size},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Information about how a `HStack` child should be layed out.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct HStackData {
    /// The margin given between the previous widget (or left of container) and the left side of the child.
    pub left_margin: f32,
    /// The margin given between the next widget and right side of the child.
    pub right_margin: f32,
    /// How the child should be vertically aligned within the `HStack`.
    pub alignment: Align,
}

impl HStackData {
    /// Sets the `top_margin` value.
    pub fn left_margin(self, left_margin: f32) -> HStackData {
        HStackData {
            left_margin,
            ..self
        }
    }

    /// Sets the `right_margin` value.
    pub fn right_margin(self, right_margin: f32) -> HStackData {
        HStackData {
            right_margin,
            ..self
        }
    }

    /// Sets the `align` value.
    pub fn align(self, alignment: Align) -> HStackData {
        HStackData { alignment, ..self }
    }
}

#[derive(Debug)]
struct ChildData {
    data: HStackData,
    evq: BidirSingleEventQueue<Rect, Rect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: Rect,
    original_rect: Rect,
    id: u64,
}

lazy_widget! {
    generic HStack,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Abstract layout widget which arranges children in a horizontal list, possibly with left/right margins and vertical alignment (see `HStackData`).
#[derive(WidgetChildren, Debug)]
#[widget_children_trait(base::WidgetChildren)]
pub struct HStack<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    rect: Rect,
    rects: IndexMap<u64, ChildData>,
    next_rect_id: u64,
    dirty: bool,
    visibility: base::Visibility,

    themed: draw::PhantomThemed,
    layout: base::WidgetLayoutEvents,
    drop_event: RcEventQueue<base::DropEvent>,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> HStack<U, G> {
    /// Creates a new horizontal stack widget with a given rectangle.
    pub fn new(rect: Rect) -> Self {
        HStack {
            rect,
            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,

            themed: Default::default(),
            layout: Default::default(),
            visibility: Default::default(),
            drop_event: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Layout for HStack<U, G> {
    type PushData = HStackData;

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
                drop_listener: child.drop_event().listen(),
                rect,
                original_rect: rect,
                id,
            },
        );
    }

    fn remove(&mut self, child: &mut impl base::LayableWidget, restore_original: bool) {
        if let Some(data) = child.layout_id().and_then(|id| self.rects.remove(&id)) {
            child.listen_to_layout(None);
            if restore_original {
                child.set_rect(data.original_rect);
            }
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for HStack<U, G> {
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
            let mut removals = Vec::new();
            let dirty = &mut self.dirty;
            for (_, data) in &mut self.rects {
                if !data.drop_listener.peek().is_empty() {
                    removals.push(data.id);
                    *dirty = true;
                    continue;
                }

                if let Some(new_ev) = data.evq.retrieve_newest() {
                    *dirty = true;
                    data.rect = new_ev;
                }
            }
            for removal in removals {
                self.rects.remove(&removal);
            }
        }

        if self.dirty {
            let mut advance = self.rect.origin.x;
            for (_, data) in &mut self.rects {
                advance += data.data.left_margin;

                let mut rect = data.rect;
                rect.origin.x = advance;
                rect.origin.y = match data.data.alignment {
                    Align::Begin => self.rect.origin.y,
                    Align::Middle => display::center_vertically(rect, self.rect).y,
                    Align::End => self.rect.origin.y + self.rect.size.height - rect.size.height,
                    Align::Stretch => {
                        rect.size.height = self.rect.size.height;
                        self.rect.origin.y
                    }
                };

                data.evq.emit_owned(rect);
                data.rect = rect;

                advance += rect.size.width + data.data.right_margin;
            }

            self.dirty = false;
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::LayableWidget for HStack<U, G> {
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Movable for HStack<U, G> {
    #[inline]
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
    }

    #[inline]
    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Resizable for HStack<U, G> {
    #[inline]
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
    }

    #[inline]
    fn size(&self) -> Size {
        self.rect.size
    }
}
