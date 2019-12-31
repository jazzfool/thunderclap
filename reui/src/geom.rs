//! Widget positioning module.

use crate::base;

/// Unit of absolute widget space.
pub struct AbsoluteUnit;

/// Point relative to the window instead of parent.
pub type AbsolutePoint = reclutch::euclid::Point2D<f32, AbsoluteUnit>;
/// Rectangle relative to the window instead of parent.
pub type AbsoluteRect = reclutch::euclid::Rect<f32, AbsoluteUnit>;

/// Unit of relative widget space.
pub struct RelativeUnit;

/// Point relative to the parent.
pub type RelativePoint = reclutch::euclid::Point2D<f32, RelativeUnit>;
/// Rectangle relative to the parent.
pub type RelativeRect = reclutch::euclid::Rect<f32, RelativeUnit>;

pub trait StoresParentPosition {
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint);
    fn parent_position(&self) -> AbsolutePoint;
}

pub enum AgnosticPoint {
    Relative(RelativePoint),
    Absolute(AbsolutePoint),
}

impl From<RelativePoint> for AgnosticPoint {
    fn from(pt: RelativePoint) -> Self {
        AgnosticPoint::Relative(pt)
    }
}

impl From<AbsolutePoint> for AgnosticPoint {
    fn from(pt: AbsolutePoint) -> Self {
        AgnosticPoint::Absolute(pt)
    }
}

pub enum AgnosticRect {
    Relative(RelativeRect),
    Absolute(AbsoluteRect),
}

impl From<RelativeRect> for AgnosticRect {
    fn from(rect: RelativeRect) -> Self {
        AgnosticRect::Relative(rect)
    }
}

impl From<AbsoluteRect> for AgnosticRect {
    fn from(rect: AbsoluteRect) -> Self {
        AgnosticRect::Absolute(rect)
    }
}

pub trait ContextuallyMovable: base::Movable + StoresParentPosition {
    fn set_ctxt_position(&mut self, position: AgnosticPoint);

    #[inline]
    fn abs_position(&self) -> AbsolutePoint {
        self.position().cast_unit() + self.parent_position().to_vector()
    }

    #[inline]
    fn abs_bounds(&self) -> AbsoluteRect {
        self.bounds().cast_unit().translate(self.parent_position().to_vector())
    }

    #[inline]
    fn abs_convert_pt(&self, pt: RelativePoint) -> AbsolutePoint {
        pt.cast_unit() + self.parent_position().to_vector()
    }

    #[inline]
    fn rel_convert_pt(&self, pt: AbsolutePoint) -> RelativePoint {
        pt.cast_unit() - self.parent_position().to_vector().cast_unit()
    }
}

impl<W: base::WidgetChildren> ContextuallyMovable for W {
    #[inline]
    fn set_ctxt_position(&mut self, position: AgnosticPoint) {
        self.set_position(match position {
            AgnosticPoint::Relative(rel_pt) => rel_pt,
            AgnosticPoint::Absolute(abs_pt) => {
                abs_pt.cast_unit() - self.parent_position().to_vector().cast_unit()
            }
        });
        update_parent_positions(self);
    }
}

pub trait ContextuallyRectangular: ContextuallyMovable + base::Rectangular {
    fn set_ctxt_rect(&mut self, rect: impl Into<AgnosticRect>);

    #[inline]
    fn abs_rect(&self) -> AbsoluteRect {
        self.rect().cast_unit().translate(self.parent_position().to_vector())
    }

    #[inline]
    fn abs_convert_rect(&self, mut rect: RelativeRect) -> AbsoluteRect {
        rect.origin = self.abs_convert_pt(rect.origin).cast_unit();
        rect.cast_unit()
    }

    #[inline]
    fn rel_convert_rect(&self, mut rect: AbsoluteRect) -> RelativeRect {
        rect.origin = self.rel_convert_pt(rect.origin).cast_unit();
        rect.cast_unit()
    }
}

impl<W: base::WidgetChildren + base::Rectangular> ContextuallyRectangular for W {
    fn set_ctxt_rect(&mut self, rect: impl Into<AgnosticRect>) {
        self.set_rect(match rect.into() {
            AgnosticRect::Relative(rel_rect) => rel_rect,
            AgnosticRect::Absolute(abs_rect) => {
                abs_rect.translate(-self.parent_position().to_vector()).cast_unit()
            }
        });
        update_parent_positions(self);
    }
}

fn update_parent_positions<U, G, D>(
    root: &mut dyn base::WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
) {
    let pos = root.abs_position();
    for child in root.children_mut() {
        child.set_parent_position(pos);
        update_parent_positions(child);
    }
}
