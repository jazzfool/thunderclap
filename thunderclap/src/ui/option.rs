//! `Option` but for widgets.

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
    /// Much like `Container`, this stores child widgets.
    /// Unlike `Container`, this can only store one child widget, optionally.
    /// This is useful if you want to "delete" a widget.
    ///
    /// This is considered an "invisible" widget; everything delegates to the child widget.
    pub struct OptionWidget<C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static> {
        LayableWidget,
        DropNotifier,
        HasVisibility,
        Repaintable,
        Rectangular,
        OperatesVerbGraph,
        StoresParentPosition,

        {
            child: Option<C>,
        }
    }
}

impl<
        U,
        G,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > ui::core::CoreWidget<()> for OptionWidget<U, G, C>
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
    > OptionWidget<U, G, C>
{
    /// Creates a new container widget, possibly with an existing list of dynamic children.
    pub fn new(child: Option<C>) -> Self {
        OptionWidgetBuilder {
            rect: Default::default(),
            graph: vg::VerbGraph::default().into(),

            child,
        }
        .build()
    }

    /// Changes the child, returning the old child, if there was one.
    #[inline]
    pub fn set(&mut self, child: C) -> Option<C> {
        self.child.replace(child)
    }

    /// Removes/deletes the child if there was one and returns it.
    #[inline]
    pub fn remove(&mut self) -> Option<C> {
        self.child.take()
    }

    /// Returns the child immutably.
    #[inline]
    pub fn get(&self) -> Option<&C> {
        self.child.as_ref()
    }

    /// Returns the child mutably.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut C> {
        self.child.as_mut()
    }
}

impl<
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > Widget for OptionWidget<U, G, C>
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut U) {
        base::invoke_update(self, aux);

        self.set_ctxt_rect(child.map(|child| child.abs_bounds()).unwrap_or_default());
    }
}

impl<
        U: base::UpdateAuxiliary,
        G: base::GraphicalAuxiliary,
        C: WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand> + 'static,
    > WidgetChildren for OptionWidget<U, G, C>
{
    fn children(
        &self,
    ) -> Vec<
        &dyn base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand>,
    > {
        if let Some(child) = &self.child {
            vec![child]
        } else {
            vec![]
        }
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
        if let Some(child) = &mut self.child {
            vec![child]
        } else {
            vec![]
        }
    }
}
