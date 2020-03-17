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
        display::{self, DisplayCommand, Rect, Size, Vector},
        event::{bidir_single::Queue as BidirSingleEventQueue, RcEventListener},
        prelude::*,
    },
};

/// A 2D position based on a relative offset and an absolute offset.
/// The `relative` offset is expressed as a fraction of the corresponding parent dimension for each component in `(x, y)`.
/// The `post_relative` offset is expressed as a fraction of the result of the `relative` and `real` calculated size from `FractionSize`.
/// The `real` offset is a concrete size in DPI pixels which is added onto the offset calculated from `relative`.
///
/// For example, `FractionalPosition { relative: (0.3, 0.1), post_relative: (-0.5, 0.0), real: Vector::new(5, 30) }` for a child with a computed size of `50 x 50`
/// placed within a parent of size `100 x 100` positioned at `(50, 50)` will result in an absolute position of `(60, 90)` (or relatively; `(10, 40)`).
/// Following the calculation; `100 * 0.3 = 30, + 50 = 80, + 5 = 85, - 0.5 * 50 = 60` and `100 * 0.1 = 10, + 50 = 60, + 30 = 90`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FractionalPosition {
    pub relative: (f32, f32),
    pub post_relative: (f32, f32),
    pub real: Vector,
}

/// A 2D size based on a relative size and an absolute size.
/// The `relative` size is expressed as a fraction of the corresponding parent dimension for each component in `(width, height)`.
/// The `post_relative` size is expressed as a fraction of the result of the `relative` and `real` calculated size.
/// The `real` size is a concrete size in DPI pixels which is added onto the size calculated from `relative`.
///
/// For example, `FractionalSize { relative: (0.5, 0.75), post_relative: (0.6, -0.2), real: Size::new(15, 10) }` placed within a parent
/// of size `100 x 100` will result in a size of `120 x 68`, because `100 * 0.5 = 50, + 15 = 75, + 0.6 * 75 = 120` and `100 * 0.75 = 75, + 10 = 85, - 0.2 * 85 = 68`.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FractionalSize {
    pub relative: (f32, f32),
    pub post_relative: (f32, f32),
    pub real: Size,
}

pub struct RelativeBoxItem {
    /// The offset from the `anchor`.
    pub offset: FractionalPosition,
    /// The size of the item.
    pub size: FractionalSize,
}

impl RelativeBoxItem {
    /// Returns a new `RelativeBoxItem` which will center the item in the parent.
    pub fn center(size: FractionalSize) -> Self {
        RelativeBoxItem {
            offset: FractionalPosition {
                relative: (0.5, 0.5),
                post_relative: (-0.5, -0.5),
                real: Default::default(),
            },
            size,
        }
    }

    /// Sets the `offset` value.
    pub fn offset(self, offset: FractionalPosition) -> RelativeBoxItem {
        RelativeBoxItem { offset, ..self }
    }

    /// Sets the `size` value.
    pub fn size(self, size: FractionalSize) -> RelativeBoxItem {
        RelativeBoxItem { size, ..self }
    }
}

struct ChildData {
    data: RelativeBoxItem,
    evq: BidirSingleEventQueue<AbsoluteRect, AbsoluteRect>,
    drop_listener: RcEventListener<base::DropEvent>,
    rect: AbsoluteRect,
    original_rect: AbsoluteRect,
    id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativeBox;

impl<U, G> ui::WidgetDataTarget<U, G> for RelativeBox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = RelativeBoxWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for RelativeBox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(_theme: &dyn draw::Theme) -> Self {
        RelativeBox
    }

    fn construct(self, _theme: &dyn draw::Theme, _u_aux: &mut U) -> Self::Target {
        let data = base::Observed::new(self);

        RelativeBoxWidgetBuilder {
            rect: Default::default(),
            graph: None,
            data,

            layout_child: None,
            dirty: true,
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<()> for RelativeBoxWidget<U, G>
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
    /// Powerful layout widget, allowing for simple relative screen positioning, independent of resolution.
    /// This is useful for;
    /// - Centering in the parent.
    /// - Aligning to a corner or edge of the parent.
    /// - Expanding to fill the parent on any or all axes.
    /// - Expanding to fill some fraction of the parent.
    /// - Anything else in which the relative metrics can be based on a fraction.
    pub struct RelativeBoxWidget {
        widget::MAX,

        <RelativeBox> State,

        {
            layout_child: Option<ChildData>,
            dirty: bool,
        }
    }
}

impl<U, G> base::Layout for RelativeBoxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type PushData = RelativeBoxItem;

    fn push(&mut self, data: Option<Self::PushData>, child: &mut impl base::LayableWidget) {
        if self.layout_child.is_some() {
            // TODO(jazzfool): should we return instead of panicking?
            panic!("RelativeBox only accepts 1 child at a time.");
        }

        self.dirty = true;

        let evq = BidirSingleEventQueue::new();
        child.listen_to_layout(base::WidgetLayoutEventsInner { id: 0, evq: evq.secondary() });

        let rect = child.abs_rect();

        self.layout_child = Some(ChildData {
            data: data.unwrap_or(RelativeBoxItem::center(Default::default())),
            evq,
            drop_listener: child.drop_event().listen(),
            rect,
            original_rect: rect,
            id: 0,
        });
    }

    fn remove(&mut self, child: &mut impl base::LayableWidget, restore_original: bool) {
        if let Some(data) = self.layout_child {
            child.listen_to_layout(None);
            if restore_original {
                child.set_ctxt_rect(data.original_rect);
            }
        }
    }
}

impl<U, G> Widget for RelativeBoxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;
}
