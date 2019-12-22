use {
    crate::{base, draw},
    reclutch::{
        display::{DisplayCommand, GraphicsDisplay},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

lazy_widget! {
    generic Container,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

/// Container which dynamically stores widgets.
/// If you don't need access to children past their creation then you can bundle them up in this.
/// Those children will still be rendered and receive updates.
pub struct Container<U, G>
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

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Container<U, G> {
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
        Container {
            children,

            themed: Default::default(),
            visibility: Default::default(),
            drop_event: Default::default(),

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

    /// Moves a child into the container.
    pub fn push(
        &mut self,
        child: Box<
            dyn base::WidgetChildren<
                UpdateAux = U,
                GraphicalAux = G,
                DisplayObject = DisplayCommand,
            >,
        >,
    ) {
        self.children.push(child);
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for Container<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut U) {
        base::invoke_update(self, aux);
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut G) {
        base::invoke_draw(self, display, aux);
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::WidgetChildren
    for Container<U, G>
{
    fn children(
        &self,
    ) -> Vec<
        &dyn base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = DisplayCommand>,
    > {
        self.children
            .iter()
            .map(|child| child.as_ref() as _)
            .collect()
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
        self.children
            .iter_mut()
            .map(|child| child.as_mut() as _)
            .collect()
    }
}
