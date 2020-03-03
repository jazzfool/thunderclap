use {
    crate::{
        base::{self, WidgetChildren},
        geom::*,
        ui,
    },
    reclutch::{display::DisplayCommand, prelude::*, verbgraph as vg},
};

use crate as thunderclap;
crate::widget! {
    #[doc = "Container which dynamically stores widgets."]
    #[doc = "If you don't need access to children past their creation then you can bundle them up in this."]
    #[doc = "Those children will still be rendered and receive updates."]
    pub struct ContainerWidget<C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static> {
        LayableWidget,
        DropNotifier,
        HasVisibility,
        Repaintable,
        Rectangular,
        OperatesVerbGraph,
        StoresParentPosition,

        {
            children: Vec<C>,
        }
    }
}

impl<
        U,
        G,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > ui::core::CoreWidget<()> for ContainerWidget<U, G, C>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) {}
}

impl<
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > ContainerWidget<U, G, C>
{
    /// Creates a new container widget, possibly with an existing list of dynamic children.
    pub fn new(children: Vec<C>) -> Self {
        ContainerWidgetBuilder {
            rect: Default::default(),
            graph: vg::VerbGraph::default().into(),

            children,
        }
        .build()
    }

    /// Moves a child into the container.
    pub fn push(&mut self, child: C) {
        self.children.push(child);
    }
}

impl<
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > Widget for ContainerWidget<U, G, C>
{
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

impl<
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > WidgetChildren for ContainerWidget<U, G, C>
{
    fn children(
        &self,
    ) -> Vec<
        &dyn base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand>,
    > {
        self.children.iter().map(|child| child as _).collect()
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
        self.children.iter_mut().map(|child| child as _).collect()
    }
}
