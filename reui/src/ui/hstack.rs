use {
    super::Align,
    crate::{base, draw, ui},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Rect},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Information about how a `HStack` child should be layed out.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct HStackItem {
    /// The margin given between the previous widget (or left of container) and the left side of the child.
    pub left_margin: f32,
    /// The margin given between the next widget and right side of the child.
    pub right_margin: f32,
    /// How the child should be vertically aligned within the `HStack`.
    pub alignment: Align,
}

impl HStackItem {
    /// Sets the `top_margin` value.
    pub fn left_margin(self, left_margin: f32) -> HStackItem {
        HStackItem { left_margin, ..self }
    }

    /// Sets the `right_margin` value.
    pub fn right_margin(self, right_margin: f32) -> HStackItem {
        HStackItem { right_margin, ..self }
    }

    /// Sets the `align` value.
    pub fn align(self, alignment: Align) -> HStackItem {
        HStackItem { alignment, ..self }
    }
}

#[derive(Debug)]
struct ChildData {
    data: HStackItem,
    evq: BidirSingleEventQueue<Rect, Rect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: Rect,
    original_rect: Rect,
    id: u64,
}

lazy_widget! {
    generic HStackWidget,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Abstract layout widget which arranges children in a horizontal list, possibly with left/right margins and vertical alignment (see `HStackData`).
#[derive(WidgetChildren, LayableWidget, Movable, Resizable, Debug)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct HStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub data: base::Observed<HStack>,

    rects: IndexMap<u64, ChildData>,
    next_rect_id: u64,
    dirty: bool,
    themed: draw::PhantomThemed,
    drop_event: RcEventQueue<base::DropEvent>,
    visibility: base::Visibility,

    #[widget_rect]
    rect: Rect,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HStack {
    pub left_margin: f32,
    pub right_margin: f32,
    pub alignment: Align,
}

impl<U, G> ui::WidgetDataTarget<U, G> for HStack
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type Target = HStackWidget<U, G>;
}

impl HStack {
    pub fn from_theme(_theme: &dyn draw::Theme) -> Self {
        HStack { left_margin: 0.0, right_margin: 0.0, alignment: Align::Begin }
    }

    pub fn construct<U, G>(
        self,
        _theme: &dyn draw::Theme,
        _u_aux: &mut U,
        _g_aux: &mut G,
    ) -> HStackWidget<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        HStackWidget {
            data,

            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,
            themed: Default::default(),
            drop_event: Default::default(),
            visibility: Default::default(),

            rect: Default::default(),
            layout: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U, G> HStackWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn resize_to_fit(&mut self) {
        let mut max_rect: Option<Rect> = None;
        for (_, child) in &self.rects {
            if let Some(ref mut max_rect) = max_rect {
                *max_rect = max_rect.union(&child.rect);
            } else {
                max_rect = child.rect.into();
            }
        }

        if let Some(rect) = max_rect {
            self.rect = rect;
            self.layout.notify(self.rect);
            use base::Repaintable;
            self.repaint();
        }
    }

    fn on_transform(&mut self) {
        self.dirty = true;
        self.layout.notify(self.rect);
    }
}

impl<U, G> base::Layout for HStackWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type PushData = Option<HStackItem>;

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
                data: data.unwrap_or(HStackItem {
                    left_margin: self.data.left_margin,
                    right_margin: self.data.right_margin,
                    alignment: self.data.alignment,
                }),
                evq,
                drop_listener: child.drop_event().listen(),
                rect,
                original_rect: rect,
                id,
            },
        );

        self.resize_to_fit();
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

impl<U, G> Widget for HStackWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
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
            self.resize_to_fit();
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

impl<U, G> ui::DefaultWidgetData<HStack> for HStackWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline]
    fn default_data(&mut self) -> &mut base::Observed<HStack> {
        &mut self.data
    }
}
