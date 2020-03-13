//! Widget positioning module.

use crate::base;

/// Unit of absolute widget space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbsoluteUnit;

/// Point relative to the window instead of parent.
pub type AbsolutePoint = reclutch::euclid::Point2D<f32, AbsoluteUnit>;
/// Rectangle relative to the window instead of parent.
pub type AbsoluteRect = reclutch::euclid::Rect<f32, AbsoluteUnit>;

/// Unit of relative widget space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativeUnit;

/// Point relative to the parent.
pub type RelativePoint = reclutch::euclid::Point2D<f32, RelativeUnit>;
/// Rectangle relative to the parent.
pub type RelativeRect = reclutch::euclid::Rect<f32, RelativeUnit>;

/// Getter/setter for widgets which store their parent's position.
pub trait StoresParentPosition {
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint);
    fn parent_position(&self) -> AbsolutePoint;
}

/// A point that can be either relative or absolute.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// A rectangle that can be either relative or absolute.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Getters/setters for context-aware 2D translation.
pub trait ContextuallyMovable: base::Movable + StoresParentPosition {
    /// Changes the position to an agnostic point (i.e. accepts relative or absolute points).
    fn set_ctxt_position(&mut self, position: AgnosticPoint);

    /// Returns the position relative to the window.
    #[inline]
    fn abs_position(&self) -> AbsolutePoint {
        self.position().cast_unit() + self.parent_position().to_vector()
    }

    /// Returns the bounds with the position relative to the window.
    #[inline]
    fn abs_bounds(&self) -> AbsoluteRect {
        self.bounds().cast_unit().translate(self.parent_position().to_vector())
    }

    /// Converts a point relative to this widget to an absolute point (relative to the window).
    #[inline]
    fn abs_convert_pt(&self, pt: RelativePoint) -> AbsolutePoint {
        pt.cast_unit() + self.parent_position().to_vector()
    }

    /// Converts an absolute point (relative to the window) to a point relative to this widget.
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

/// Getters/setters for context-aware 2D rectangle translation.
pub trait ContextuallyRectangular: ContextuallyMovable + base::Rectangular {
    /// Changes the rectangle to an agnostic rectangle (i.e. accepts relative and absolute coordinates).
    fn set_ctxt_rect(&mut self, rect: impl Into<AgnosticRect>);

    /// Returns the relative rectangle in absolute coordinates (i.e. relative to the window).
    #[inline]
    fn abs_rect(&self) -> AbsoluteRect {
        self.rect().cast_unit().translate(self.parent_position().to_vector())
    }

    /// Converts a rectangle relative to this widget into absolute coordinates (i.e. relative to the window).
    #[inline]
    fn abs_convert_rect(&self, mut rect: RelativeRect) -> AbsoluteRect {
        rect.origin = self.abs_convert_pt(rect.origin).cast_unit();
        rect.cast_unit()
    }

    /// Converts an absolute rectangle (i.e. relative to the window) into relative coordinates.
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
