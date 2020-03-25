use {
    crate::{
        base::{self, Resizable},
        draw,
        geom::*,
        ui,
    },
    indexmap::IndexMap,
    reclutch::{
        display::{DisplayCommand, Rect},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener},
        prelude::*,
        verbgraph as vg,
    },
};

struct ChildData {
    evq: BidirSingleEventQueue<AbsoluteRect, AbsoluteRect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: AbsoluteRect,
    original_rect: AbsoluteRect,
    id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    offset: RelativePoint,
    size: Size,
}

impl<U, G> ui::WidgetDataTarget<U, G> for Fill
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = FillWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for Fill
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(_theme: &dyn draw::Theme) -> Self {
        Fill { offset: Default::default(), size: Default::default() }
    }

    fn construct(self, _theme: &dyn draw::Theme, _u_aux: &mut U) -> FillWidget<U, G>
    where
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
    {
        let data = base::Observed::new(self);

        let graph = vg::verbgraph! {
            FillWidget<U, G> as obj,
            U as _aux,
            "data" => _ev in &data.on_change => {
                change => {
                    obj.rect.origin = obj.data.position.cast_unit();
                    obj.rect.size = obj.data.size.cast_unit();
                }
            }
        };

        FillWidgetBuilder {
            rect: Default::default(),
            graph: graph.into(),
            data,

            rects: IndexMap::new(),
            next_rect_id: 0,
            dirty: true,
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<()> for FillWidget<U, G>
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
    pub struct FillWidget {
        widget::MAX,

        <Fill> State,

        {
            rects: IndexMap<u64, ChildData>,
            next_rect_id: u64,
            dirty: bool,
        },
    }
}

impl<U, G> base::Layout for FillWidget<U, G>
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
                id,
            },
        );
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

impl<U, G> Widget for FillWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
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
                    continue;
                }

                if data.evq.retrieve_newest().is_some() {
                    *dirty = true;
                }
            }
            for removal in removals {
                self.rects.remove(&removal);
            }
        }

        if self.dirty {
            let abs_rect = self.abs_rect();
            for (_, data) in &mut self.rects {
                data.evq.emit_owned(abs_rect);
                data.rect = abs_rect;
            }

            self.dirty = false;
        }
    }
}
