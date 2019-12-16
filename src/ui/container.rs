use {
    crate::{base, draw},
    reclutch::{
        display::{DisplayCommand, GraphicsDisplay},
        prelude::*,
    },
    std::marker::PhantomData,
};

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

    phantom_u: PhantomData<U>,
    phantom_g: PhantomData<G>,
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Container<U, G> {
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

            phantom_u: Default::default(),
            phantom_g: Default::default(),
        }
    }

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

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::HasVisibility
    for Container<U, G>
{
    #[inline]
    fn set_visibility(&mut self, visibility: base::Visibility) {
        self.visibility = visibility
    }

    #[inline]
    fn visibility(&self) -> base::Visibility {
        self.visibility
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> base::Repaintable for Container<U, G> {
    #[inline]
    fn repaint(&mut self) {}
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> draw::HasTheme for Container<U, G> {
    #[inline]
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    #[inline]
    fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
}
