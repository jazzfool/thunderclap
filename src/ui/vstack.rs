use {
    crate::{base, draw},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Point, Rect, Size},
        event::{RcEventListener, RcEventQueue},
        prelude::*,
    },
    std::marker::PhantomData,
};

/// How a child should be aligned horizontally within a `VStack`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VStackAlignment {
    /// The child is align to the left side.
    Left,
    /// The child is centered.
    Middle,
    /// The child is align to the right side.
    Right,
    /// The width of the child is stretched to fill the container.
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VStackData {
    pub top_margin: f32,
    pub bottom_margin: f32,
    pub alignment: VStackAlignment,
}

#[derive(Debug)]
struct ChildData {
    data: VStackData,
    output: RcEventQueue<Rect>,
    input: RcEventListener<Rect>,
    rect: Rect,
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

        let mut output = RcEventQueue::new();
        let input_q = RcEventQueue::new();

        let input = input_q.listen();

        child.listen_to_layout(base::LayoutEvents {
            id,
            rcv: &mut output,
            notify: input_q,
        });

        self.rects.insert(
            id,
            ChildData {
                data,
                output,
                input,
                rect: child.rect(),
            },
        );
    }

    /// De-registers a widget from the layout, optionally restoring the original widget rectangle.
    fn remove(&mut self, child: &mut impl base::LayableWidget, _restore_original: bool) {
        // TODO(jazzfool): `restore_original` support.
        child.listen_to_layout(None);
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
        {
            let dirty = &mut self.dirty;
            for (_, data) in &mut self.rects {
                let rect = &mut data.rect;
                data.input.with(|events| {
                    for event in events {
                        *dirty = true;
                        *rect = *event;
                    }
                });
            }
        }

        if self.dirty {
            let mut advance = self.rect.origin.y;
            for (_, data) in &mut self.rects {
                advance += data.data.top_margin;

                let mut rect = data.rect;
                rect.origin.y = advance;
                rect.origin.x = match data.data.alignment {
                    VStackAlignment::Left => self.rect.origin.x,
                    VStackAlignment::Middle => display::center_horizontally(rect, self.rect).x,
                    VStackAlignment::Right => {
                        self.rect.origin.x + self.rect.size.width - rect.size.width
                    }
                    VStackAlignment::Stretch => {
                        rect.size.width = self.rect.size.width;
                        self.rect.origin.x
                    }
                };

                data.output.emit_owned(rect);
                data.rect = rect;

                advance += rect.size.height + data.data.bottom_margin;
            }

            self.dirty = false;
        }
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Repaintable for VStack<U, G> {
    fn repaint(&mut self) {}
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Movable for VStack<U, G> {
    fn set_position(&mut self, position: Point) {
        self.rect.origin = position;
    }

    fn position(&self) -> Point {
        self.rect.origin
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Resizable for VStack<U, G> {
    fn set_size(&mut self, size: Size) {
        self.rect.size = size;
    }

    fn size(&self) -> Size {
        self.rect.size
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> draw::HasTheme for VStack<U, G> {
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
}
