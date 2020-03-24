use {
    crate::{base, draw, geom::*, ui},
    reclutch::{
        display::{DisplayCommand, Rect, Size},
        prelude::*,
        verbgraph as vg,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizedBox {
    pub position: RelativePoint,
    pub size: Size,
}

impl<U, G> ui::WidgetDataTarget<U, G> for SizedBox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = SizedBoxWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for SizedBox
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(_theme: &dyn draw::Theme) -> Self {
        SizedBox { position: RelativePoint::new(0.0, 0.0), size: Size::new(0.0, 0.0) }
    }

    fn construct(self, _theme: &dyn draw::Theme, _u_aux: &mut U) -> SizedBoxWidget<U, G> {
        let data = base::Observed::new(self);

        let graph = vg::verbgraph! {
            SizedBoxWidget<U, G> as obj,
            U as _aux,
            "rect" => _ev in &data.on_change => {
                change => {
                    obj.rect.origin = obj.data.position.cast_unit();
                    obj.rect.size = obj.data.size.cast_unit();
                }
            }
        };

        SizedBoxWidgetBuilder {
            rect: RelativeRect::new(data.get().position, data.get().size.cast_unit()),
            graph: graph.into(),
            data,
        }
        .build()
    }
}

impl<U, G> ui::core::CoreWidget<()> for SizedBoxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn derive_state(&self) {}
}

use crate as thunderclap;
crate::widget! {
    pub struct SizedBoxWidget {
        widget::MAX,
        <SizedBox> State,
    }
}

impl<U, G> base::Layout for SizedBoxWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type PushData = ();

    fn push(&mut self, _data: Option<()>, _child: &mut impl base::LayableWidget) {}
    fn remove(&mut self, _child: &mut impl base::LayableWidget, _restore_original: bool) {}
}

impl<U, G> Widget for SizedBoxWidget<U, G>
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
        }
    }
}
