use {
    crate::draw::HasTheme,
    reclutch::{
        display::{Color, FontInfo, GraphicsDisplay, Point, Rect, ResourceReference, Size},
        event::RcEventQueue,
        widget::Widget,
    },
    std::{cell::RefCell, rc::Rc},
};

/// A custom widget children trait with additional bounds.
/// This is used as an alternative to `reclutch::widget::WidgetChildren`.
///
/// You can still use this with the derive macro as follows:
/// ```rust
/// #[derive(WidgetChildren)]
/// #[widget_children_trait(reui::base::WidgetChildren)]
/// struct MyWidget;
/// ```
pub trait WidgetChildren: Widget + HasTheme {
    /// Returns a list of all the children as a vector of immutable `dyn WidgetChildren`.
    fn children(
        &self,
    ) -> Vec<
        &dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }

    /// Returns a list of all the children as a vector of mutable `dyn WidgetChildren`.
    fn children_mut(
        &mut self,
    ) -> Vec<
        &mut dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        Vec::new()
    }
}

/// Implemented by all widgets that can be moved/positioned.
pub trait Movable: Widget {
    /// Changes the current position of the widget.
    fn set_position(&mut self, position: Point);
    /// Returns the current position of the widget.
    fn position(&self) -> Point;
}

/// Implemented by all widgets that can be resized.
pub trait Resizable: Widget {
    /// Changes the current size of the widget.
    fn set_size(&mut self, size: Size);
    /// Returns the current size of the widget.
    fn size(&self) -> Size;
}

/// Implemented by all widgets that can be moved and resized.
///
/// There's no need to implement this manually, as long as `Movable` and `Resizable`
/// have been implemented, this will be automatically implemented alongside them.
pub trait Rectangular: Widget + Movable + Resizable {
    /// Changes the rectangular bounds.
    ///
    /// If `Rectangular` is a blanket implementation, then this simply becomes
    /// `set_position()` and `set_size()`.
    fn set_rect(&mut self, rect: Rect);
    /// Returns the rectangular bounds.
    ///
    /// If `Rectangular` is a blanket implementation, then this is simply a constructor
    /// for `Rect` based on the values returned from `position()` and `size()`.
    fn rect(&self) -> Rect;
}

impl<T> Rectangular for T
where
    T: Widget + Movable + Resizable,
{
    #[inline]
    fn set_rect(&mut self, rect: Rect) {
        self.set_position(rect.origin);
        self.set_size(rect.size);
    }

    #[inline]
    fn rect(&self) -> Rect {
        Rect::new(self.position(), self.size())
    }
}

/// Trait required for any type passed as the `UpdateAux` type (seen as `U` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait UpdateAuxiliary {
    /// Returns the queue where window events (`WindowEvent`) are emitted, immutably.
    fn window_queue(&self) -> &RcEventQueue<WindowEvent>;
    /// Returns the queue where window events (`WindowEvent`) are emitted, mutably.
    fn window_queue_mut(&mut self) -> &mut RcEventQueue<WindowEvent>;
}

/// Trait required for any type passed as the `GraphicalAux` type (seen as `G` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait GraphicalAuxiliary {
    /// Returns the UI font.
    fn ui_font(&self) -> (ResourceReference, FontInfo);
    /// Returns the UI font in semi-bold variant.
    /// This may be used over `ui_font` stylistically by a theme.
    fn semibold_ui_font(&self) -> (ResourceReference, FontInfo);
    /// Returns the HiDPI scaling factor.
    fn scaling(&self) -> f32;
}

/// Event data that can be "consumed". This is needed for events such as clicking and typing.
/// Those kinds of events aren't typically received by multiple widgets.
///
/// As an example of this, say you have multiple buttons stacked atop each other.
/// When you click that stack of buttons, only the one on top should receive the click event,
/// as in, the event is *consumed*.
///
/// Note that this primitive isn't very strict. The consumption conditions can be bypassed
/// in case the data needs to be accessed regardless of state, and the predicate can be
/// exploited to use the data without consuming it.
///
/// Also note that the usage of "consume" is completely unrelated to the consume/move
/// semantics of Rust. In fact, nothing is actually consumed in this implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumableEvent<T>(Rc<RefCell<bool>>, T);

impl<T> ConsumableEvent<T> {
    /// Creates a unconsumed event, initialized with `val`.
    pub fn new(val: T) -> Self {
        ConsumableEvent(Rc::new(RefCell::new(true)), val)
    }

    /// Returns the event data as long as **both** the following conditions are satisfied:
    /// 1. The event hasn't been consumed yet.
    /// 2. The predicate returns true.
    ///
    /// The point of the predicate is to let the caller see if the event actually applies
    /// to them before consuming needlessly.
    pub fn with<P: FnMut(&T) -> bool>(&self, mut pred: P) -> Option<&T> {
        if *self.0.borrow() {
            if pred(&self.1) {
                *self.0.borrow_mut() = false;
                return Some(&self.1);
            }
        }
        None
    }

    /// Returns the inner event data regardless of consumption.
    pub fn get(&self) -> &T {
        &self.1
    }
}

/// An event related to the window, e.g. input.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// The user pressed a mouse button.
    MousePress(ConsumableEvent<(Point, MouseButton)>),
    /// The user released a mouse button.
    /// This event complements `MousePress`, which means it realistically can only
    /// be emitted after `MousePress` has been emitted.
    MouseRelease(ConsumableEvent<(Point, MouseButton)>),
    /// The user moved the cursor.
    MouseMove(ConsumableEvent<Point>),
    /// Emitted immediately before an event which is capable of changing focus.
    /// If implementing a focus-able widget, to handle this event, simply clear
    /// the local "focused" flag (which should ideally be stored as `draw::state::InteractionState`).
    ClearFocus,
}

/// Button on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Propagates `update` for the children of a widget.
pub fn invoke_update<U, G, D>(
    widget: &mut impl WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    aux: &mut U,
) {
    // Iterate in reverse because most visually forefront widgets should get events first.
    for child in widget.children_mut().into_iter().rev() {
        child.update(aux);
    }
}

/// Propagates `draw` for the children of a widget.
pub fn invoke_draw<U, G, D>(
    widget: &mut impl WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    display: &mut dyn GraphicsDisplay<D>,
    aux: &mut G,
) {
    for child in widget.children_mut() {
        child.draw(display, aux);
    }
}

/// Creates a color from 3 unsigned 8-bit components and an `f32` alpha.
pub fn color_from_urgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Aligns a rectangle with regards to anti-aliasing.
///
/// Use this if you have, for example, a 1px stroke and want it to look sharp without losing curve anti-aliasing.
pub fn sharp_align(rect: Rect) -> Rect {
    rect.round_in().inflate(-0.5, -0.5)
}
