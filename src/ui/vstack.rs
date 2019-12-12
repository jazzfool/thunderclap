use {
    crate::{base, draw},
    indexmap::IndexMap,
    reclutch::{
        display::{self, DisplayCommand, Point, Rect, Size},
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

pub struct VStackChildData {
    data: VStackData,
    id: u64,
}

#[derive(WidgetChildren, Debug, Clone)]
#[widget_children_trait(base::WidgetChildren)]
pub struct VStack<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    rect: Rect,
    rects: IndexMap<u64, Rect>,
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
    type ChildData = VStackChildData;

    fn push<T: base::WidgetChildren + base::Rectangular>(
        &mut self,
        data: VStackData,
        child: T,
    ) -> base::LayedOut<T, Self> {
        self.dirty = true;

        let id = self.next_rect_id;
        self.next_rect_id += 1;

        self.rects.insert(id, child.rect());

        base::LayedOut::new(child, VStackChildData { data, id })
    }

    fn remove<T: base::WidgetChildren + base::Rectangular>(
        &mut self,
        child: base::LayedOut<T, Self>,
        restore_original: bool,
    ) -> T {
        self.dirty = true;

        child.decompose().0
    }

    fn update_layout(
        &mut self,
        children: Vec<
            base::ActivelyLayedOut<
                '_,
                Self::UpdateAux,
                Self::GraphicalAux,
                Self::DisplayObject,
                Self,
            >,
        >,
    ) {
        if !self.dirty {
            for child in &children {
                if let Some(rect) = self.rects.get(&child.data.id) {
                    if *rect != child.widget.rect() {
                        self.dirty = true;
                    }
                } else {
                    panic!("invalid layout child ID");
                }
            }
        }

        if self.dirty {
            let mut advance = self.rect.origin.y;
            for child in children {
                advance += child.data.data.top_margin;

                let mut rect = child.widget.rect();
                rect.origin.y = advance;

                match child.data.data.alignment {
                    VStackAlignment::Left => rect.origin.x = self.rect.origin.x,
                    VStackAlignment::Middle => {
                        rect.origin.x = display::center_horizontally(rect, self.rect).x
                    }
                    VStackAlignment::Right => {
                        rect.origin.x = self.rect.origin.x + self.rect.size.width - rect.size.width
                    }
                    VStackAlignment::Stretch => {
                        rect.origin.x = self.rect.origin.x;
                        rect.size.width = self.rect.size.width;
                    }
                }

                child.widget.set_rect(rect);
                *self.rects.get_mut(&child.data.id).unwrap() = rect;

                advance += rect.size.height + child.data.data.bottom_margin;
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
