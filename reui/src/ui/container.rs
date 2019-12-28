use {
    crate::{
        base::{self, WidgetChildren},
        draw,
    },
    reclutch::{
        display::{DisplayCommand, Rect},
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
#[derive(Movable)]
#[reui_crate(crate)]
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
    #[widget_rect]
    rect: Rect,

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
            rect: Default::default(),

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

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Widget for Container<U, G> {
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut U) {
        base::invoke_update(self, aux);

        // FIXME(jazzfool): only do this when a child position changes.
        let mut rect: Option<Rect> = None;
        for child in self.children() {
            if let Some(ref mut rect) = rect {
                *rect = rect.union(&child.bounds());
            } else {
                rect = Some(child.bounds());
            }
        }
        self.rect = rect.unwrap_or_default();
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> WidgetChildren for Container<U, G> {
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
