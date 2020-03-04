use {
    crate::{
        base::{self, Resizable},
        draw,
        geom::*,
        ui,
    },
    indexmap::IndexMap,
    reclutch::{
        display::{DisplayCommand, Rect, Size, Vector},
        euclid::SideOffsets2D,
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener},
        prelude::*,
    },
};

#[derive(Debug)]
struct ChildData {
    evq: BidirSingleEventQueue<AbsoluteRect, AbsoluteRect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: AbsoluteRect,
    original_rect: AbsoluteRect,
    distance_from_tl: Vector,
    id: u64,
}

pub type SideMargins = SideOffsets2D<f32, AbsoluteUnit>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    pub margins: SideMargins,
}

impl<U, G> ui::WidgetDataTarget<U, G> for Margins
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = MarginsWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for Margins
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(_theme: &dyn draw::Theme) -> Self {
        Margins { margins: Default::default() }
    }

    fn construct(
        self,
        _theme: &dyn draw::Theme,
        _u_aux: &mut U,
    ) -> MarginsWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        MarginsWidgetBuilder {
            rect: Default::default(),
            graph: None,
            data,

            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<()> for MarginsWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) {}

    fn on_transform(&mut self) {
        self.dirty = true;
        self.layout.notify(self.abs_rect());
    }
}

use crate as thunderclap;
crate::widget! {
    pub struct MarginsWidget {
        widget::MAX,

        <Margins> State,

        {
            rects: IndexMap<u64, ChildData>,
            next_rect_id: u64,
            dirty: bool,
        },
    }
}

impl<U, G> MarginsWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn resize_to_fit(&mut self) {
        let mut max_rect = AbsoluteRect::default();
        for (_, child) in &self.rects {
            max_rect = max_rect.union(&child.rect);
        }

        self.set_size(
            max_rect.size.cast_unit()
                + Size::new(
                    self.data.margins.left + self.data.margins.right,
                    self.data.margins.top + self.data.margins.bottom,
                ),
        );
    }
}

impl<U, G> base::Layout for MarginsWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type PushData = ();

    fn push(&mut self, _data: Option<()>, child: &mut impl base::LayableWidget) {
        self.dirty = true;

        let id = self.next_rect_id;
        self.next_rect_id += 1;

        let evq = BidirSingleEventQueue::new();

        child.listen_to_layout(base::WidgetLayoutEventsInner { id, evq: evq.secondary() });

        let rect = child.abs_rect();

        self.rects.insert(
            id,
            ChildData {
                evq,
                drop_listener: child.drop_event().listen(),
                rect,
                original_rect: rect,
                distance_from_tl: (rect.origin - self.abs_rect().origin).cast_unit(),
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

impl<U, G> Widget for MarginsWidget<U, G>
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
            let abs_rect = self.abs_rect().inner_rect(self.data.margins);
            let mut removals = Vec::new();
            for (_, data) in &mut self.rects {
                if !data.drop_listener.peek().is_empty() {
                    removals.push(data.id);
                    self.dirty = true;
                    continue;
                }

                if let Some(new_ev) = data.evq.retrieve_newest() {
                    self.dirty = true;
                    data.rect = new_ev;
                    data.distance_from_tl = (new_ev.origin - abs_rect.origin).cast_unit();
                }
            }
            for removal in removals {
                self.rects.remove(&removal);
            }
        }

        if self.dirty {
            self.resize_to_fit();
            let abs_rect = self.abs_rect();
            for (_, data) in &mut self.rects {
                let mut rect = data.rect;
                rect.origin = abs_rect.origin
                    + data.distance_from_tl.cast_unit()
                    + Vector::new(self.data.margins.left, self.data.margins.top).cast_unit();
                data.evq.emit_owned(rect);
                data.rect = rect;
            }

            self.dirty = false;
        }
    }
}
