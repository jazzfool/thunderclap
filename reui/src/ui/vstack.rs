use {
    super::Align,
    crate::{base, draw},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Rect},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Information about how a `VStack` child should be layed out.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct VStackData {
    /// The margin given between the above widget (or top of container) and the top of the child.
    pub top_margin: f32,
    /// The margin given between the below widget and bottom side of the child.
    pub bottom_margin: f32,
    /// How the child should be horizontally aligned within the `VStack`.
    pub alignment: Align,
}

impl VStackData {
    /// Sets the `top_margin` value.
    pub fn top_margin(self, top_margin: f32) -> VStackData {
        VStackData { top_margin, ..self }
    }

    /// Sets the `bottom_margin` value.
    pub fn bottom_margin(self, bottom_margin: f32) -> VStackData {
        VStackData { bottom_margin, ..self }
    }

    /// Sets the `align` value.
    pub fn align(self, alignment: Align) -> VStackData {
        VStackData { alignment, ..self }
    }
}

#[derive(Debug)]
struct ChildData {
    data: VStackData,
    evq: BidirSingleEventQueue<Rect, Rect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: Rect,
    original_rect: Rect,
    id: u64,
}

lazy_widget! {
    generic VStack,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Abstract layout widget which arranges children in a vertical list, possibly with top/bottom margins and horizontal alignment (see `VStackData`).
#[derive(WidgetChildren, LayableWidget, Movable, Resizable, Debug)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
pub struct VStack<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    rects: IndexMap<u64, ChildData>,
    next_rect_id: u64,
    dirty: bool,
    visibility: base::Visibility,
    themed: draw::PhantomThemed,
    drop_event: RcEventQueue<base::DropEvent>,

    #[widget_rect]
    rect: Rect,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> VStack<U, G> {
    /// Creates a new vertical stack widget with a given rectangle.
    pub fn new(rect: Rect) -> Self {
        VStack {
            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,
            visibility: Default::default(),
            themed: Default::default(),
            drop_event: Default::default(),

            rect,
            layout: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Layout for VStack<U, G> {
    type PushData = VStackData;

    fn push(&mut self, data: Self::PushData, child: &mut impl base::LayableWidget) {
        self.dirty = true;

        let id = self.next_rect_id;
        self.next_rect_id += 1;

        let evq = BidirSingleEventQueue::new();

        child.listen_to_layout(base::WidgetLayoutEventsInner { id, evq: evq.secondary() });

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
