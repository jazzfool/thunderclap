use {
    crate::{
        base::{self, WidgetChildren},
        draw,
        geom::*,
    },
    reclutch::{display::DisplayCommand, event::RcEventQueue, prelude::*, verbgraph as vg},
    std::marker::PhantomData,
};

lazy_widget! {
    generic ContainerWidget,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Container which dynamically stores widgets.
/// If you don't need access to children past their creation then you can bundle them up in this.
/// Those children will still be rendered and receive updates.
#[derive(Movable, Resizable, OperatesVerbGraph)]
#[thunderclap_crate(crate)]
pub struct ContainerWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    children: Vec<
        Box<
            dyn base::WidgetChildren<
                UpdateAux = U,
                GraphicalAux = G,
                DisplayObject = DisplayCommand,
            >,
        >,
    >,

    themed: draw::PhantomThemed,
    visibility: base::Visibility,
    drop_event: RcEventQueue<base::DropEvent>,
    parent_position: AbsolutePoint,

    #[widget_rect]
    rect: RelativeRect,

    graph: vg::OptionVerbGraph<Self, U>,
    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> ContainerWidget<U, G> {
    /// Creates a new container widget, possibly with an existing list of dynamic children.
    pub fn new(
        children: Vec<
            Box<
                dyn base::WidgetChildren<
                    UpdateAux = U,
                    GraphicalAux = G,
                    DisplayObject = DisplayCommand,
                >,
            >,
        >,
    ) -> Self {
        ContainerWidget {
            children,

            themed: Default::default(),
            visibility: Default::default(),
            drop_event: Default::default(),
            parent_position: Default::default(),

            rect: Default::default(),

            graph: None,
            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    /// Moves a child into the container.
    pub fn push(
        &mut self,
        child: impl base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand>
            + 'static,
    ) {
        self.children.push(Box::new(child));
    }
}

impl<U, G> vg::HasVerbGraph for ContainerWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn verb_graph(&mut self) -> &mut vg::OptionVerbGraph<Self, U> {
        &mut self.graph
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for ContainerWidget<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut U) {
        base::invoke_update(self, aux);

        // FIXME(jazzfool): only do this when a child's position changes.
        let mut rect: Option<AbsoluteRect> = None;
        for child in self.children() {
            if let Some(ref mut rect) = rect {
                *rect = rect.union(&child.abs_bounds());
            } else {
                rect = Some(child.abs_bounds());
            }
        }

        self.set_ctxt_rect(rect.unwrap_or_default());
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> WidgetChildren
    for ContainerWidget<U, G>
{
    fn children(
        &self,
    ) -> Vec<
        &dyn base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand>,
    > {
        self.children.iter().map(|child| child.as_ref() as _).collect()
    }

    fn children_mut(
        &mut self,
    ) -> Vec<
        &mut dyn base::WidgetChildren<
            UpdateAux = U,
            GraphicalAux = G,
            DisplayObject = DisplayCommand,
        >,
    > {
        self.children.iter_mut().map(|child| child.as_mut() as _).collect()
    }
}

impl<U, G> StoresParentPosition for ContainerWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint) {
        self.parent_position = parent_pos;
    }

    fn parent_position(&self) -> AbsolutePoint {
        self.parent_position
    }
}
