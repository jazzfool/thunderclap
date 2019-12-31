use {
    super::Align,
    crate::{
        base::{self, Resizable},
        draw,
        geom::*,
        ui,
    },
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Rect, Size},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// Information about how a `VStack` child should be layed out.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct VStackItem {
    /// The margin given between the above widget (or top of container) and the top of the child.
    pub top_margin: f32,
    /// The margin given between the below widget and bottom side of the child.
    pub bottom_margin: f32,
    /// How the child should be horizontally aligned within the `VStack`.
    pub alignment: Align,
}

impl VStackItem {
    /// Sets the `top_margin` value.
    pub fn top_margin(self, top_margin: f32) -> VStackItem {
        VStackItem { top_margin, ..self }
    }

    /// Sets the `bottom_margin` value.
    pub fn bottom_margin(self, bottom_margin: f32) -> VStackItem {
        VStackItem { bottom_margin, ..self }
    }

    /// Sets the `align` value.
    pub fn align(self, alignment: Align) -> VStackItem {
        VStackItem { alignment, ..self }
    }
}

#[derive(Debug)]
struct ChildData {
    data: VStackItem,
    evq: BidirSingleEventQueue<AbsoluteRect, AbsoluteRect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: AbsoluteRect,
    original_rect: AbsoluteRect,
    id: u64,
}

lazy_widget! {
    generic VStackWidget,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Abstract layout widget which arranges children in a vertical list, possibly with top/bottom margins and horizontal alignment (see `VStackData`).
#[derive(WidgetChildren, LayableWidget, Movable, Resizable, Debug)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct VStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    pub data: base::Observed<VStack>,

    rects: IndexMap<u64, ChildData>,
    next_rect_id: u64,
    dirty: bool,
    visibility: base::Visibility,
    themed: draw::PhantomThemed,
    drop_event: RcEventQueue<base::DropEvent>,
    parent_position: AbsolutePoint,

    #[widget_rect]
    rect: RelativeRect,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VStack {
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub alignment: Align,
}

impl<U, G> ui::WidgetDataTarget<U, G> for VStack
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = VStackWidget<U, G>;
}

impl VStack {
    pub fn from_theme(_theme: &dyn draw::Theme) -> Self {
        VStack { top_margin: 0.0, bottom_margin: 0.0, alignment: Align::Begin }
    }

    pub fn construct<U, G>(
        self,
        _theme: &dyn draw::Theme,
        _u_aux: &mut U,
        _g_aux: &mut G,
    ) -> VStackWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        VStackWidget {
            data,

            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,
            visibility: Default::default(),
            themed: Default::default(),
            drop_event: Default::default(),
            parent_position: Default::default(),

            rect: Default::default(),
            layout: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }
}

impl<U, G> VStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn resize_to_fit(&mut self) {
        let mut max_size = Size::zero();
        for (_, child) in &self.rects {
            let size: Size = child.rect.size.cast_unit();
            max_size.height += size.height + child.data.top_margin + child.data.bottom_margin;
            if size.width > max_size.width {
                max_size.width = size.width;
            }
        }

        self.set_size(max_size);
    }

    fn on_transform(&mut self) {
        self.dirty = true;
        self.layout.notify(self.abs_rect());
    }
}

impl<U, G> base::Layout for VStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type PushData = Option<VStackItem>;

    fn push(&mut self, data: Self::PushData, child: &mut impl base::LayableWidget) {
        self.dirty = true;

        let id = self.next_rect_id;
        self.next_rect_id += 1;

        let evq = BidirSingleEventQueue::new();

        child.listen_to_layout(base::WidgetLayoutEventsInner { id, evq: evq.secondary() });

        let rect = child.abs_rect();

        self.rects.insert(
            id,
            ChildData {
                data: data.unwrap_or(VStackItem {
                    top_margin: self.data.top_margin,
                    bottom_margin: self.data.bottom_margin,
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
                child.set_ctxt_rect(data.original_rect);
            }
        }
    }
}

impl<U, G> Widget for VStackWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect.cast_unit()
    }

    fn update(&mut self, _aux: &mut U) {
        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
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
            let abs_rect = self.abs_rect();
            let mut advance = abs_rect.origin.y;
            for (_, data) in &mut self.rects {
                advance += data.data.top_margin;

                let mut rect = data.rect;
                rect.origin.y = advance;
                rect.origin.x = match data.data.alignment {
                    Align::Begin => abs_rect.origin.x,
                    Align::Middle => {
                        display::center_horizontally(rect.cast_unit(), abs_rect.cast_unit()).x
                    }
                    Align::End => abs_rect.origin.x + abs_rect.size.width - rect.size.width,
                    Align::Stretch => {
                        rect.size.width = abs_rect.size.width;
                        abs_rect.origin.x
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

impl<U, G> ui::DefaultWidgetData<VStack> for VStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn default_data(&mut self) -> &mut base::Observed<VStack> {
        &mut self.data
    }
}

impl<U, G> StoresParentPosition for VStackWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint) {
        self.parent_position = parent_pos;
        self.on_transform();
    }

    fn parent_position(&self) -> AbsolutePoint {
        self.parent_position
    }
}
