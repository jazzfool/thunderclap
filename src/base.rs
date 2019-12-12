use {
    crate::draw,
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
/// ```ignore
/// use reclutch::WidgetChildren;
/// #[derive(WidgetChildren)]
/// #[widget_children_trait(reui::base::WidgetChildren)]
/// struct MyWidget;
/// ```
pub trait WidgetChildren: Widget + draw::HasTheme + Repaintable {
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

/// Implemented by widgets that can be repainted.
pub trait Repaintable: Widget {
    /// Repaints the widget (typically means invoking `repaint` on the inner command group).
    fn repaint(&mut self);
}

/// Implemented by widgets that can be moved/positioned.
pub trait Movable: Widget {
    /// Changes the current position of the widget.
    fn set_position(&mut self, position: Point);
    /// Returns the current position of the widget.
    fn position(&self) -> Point;
}

/// Implemented by widgets that can be resized.
pub trait Resizable: Widget {
    /// Changes the current size of the widget.
    fn set_size(&mut self, size: Size);
    /// Returns the current size of the widget.
    fn size(&self) -> Size;
}

/// Implemented by widgets that can be moved and resized.
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
    pub fn with<P>(&self, mut pred: P) -> Option<&T>
    where
        P: FnMut(&T) -> bool,
    {
        let mut is_consumed = self.0.borrow_mut();
        if *is_consumed && pred(&self.1) {
            *is_consumed = false;
            Some(&self.1)
        } else {
            None
        }
    }

    /// Returns the inner event data regardless of consumption.
    #[inline(always)]
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

/// Attachment of layout data to a widget, which is then used to update the widget
/// rectangle based on a layout.
pub trait Layout: WidgetChildren + Rectangular + Sized {
    type PushData;
    type ChildData: Sized;

    /// "Registers" a widget to the layout and returns the registration information as `LayedOut`.
    fn push<T: WidgetChildren + Rectangular>(
        &mut self,
        data: Self::PushData,
        child: T,
    ) -> LayedOut<T, Self>;
    /// De-registers a widget from the layout and returns the original widget, stripped of additional data.
    ///
    /// If `restore_original` is `true`, then the original `Rect` (when the widget was `push`ed) will be restored.
    fn remove<T: WidgetChildren + Rectangular>(
        &mut self,
        child: LayedOut<T, Self>,
        restore_original: bool,
    ) -> T;
    /// Updates the layout of a list of children.
    fn update_layout(
        &mut self,
        children: Vec<
            ActivelyLayedOut<'_, Self::UpdateAux, Self::GraphicalAux, Self::DisplayObject, Self>,
        >,
    );
}

/// A widget with extra attached information required for a layout.
#[derive(WidgetChildren)]
#[widget_children_trait(WidgetChildren)]
pub struct LayedOut<T: WidgetChildren + Rectangular, L: Layout> {
    widget: T,
    data: L::ChildData,
}

impl<T: WidgetChildren + Rectangular, L: Layout> LayedOut<T, L> {
    /// Creates a new `LayedOut`.
    pub fn new(widget: T, data: L::ChildData) -> Self {
        LayedOut { widget, data }
    }

    /// Returns `self` as a tuple.
    pub fn decompose(self) -> (T, L::ChildData) {
        (self.widget, self.data)
    }

    /// Dynamically and mutably borrows the inner widget and layout data.
    ///
    /// Required for `update_layout`.
    pub fn activate(
        &mut self,
    ) -> ActivelyLayedOut<'_, T::UpdateAux, T::GraphicalAux, T::DisplayObject, L> {
        ActivelyLayedOut {
            widget: &mut self.widget,
            data: &mut self.data,
        }
    }

    /// Returns the attached layout data immutably.
    pub fn data(&self) -> &L::ChildData {
        &self.data
    }

    /// Returns the attached layout data mutably.
    pub fn data_mut(&mut self) -> &mut L::ChildData {
        &mut self.data
    }
}

impl<T: WidgetChildren + Rectangular, L: Layout> std::ops::Deref for LayedOut<T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<T: WidgetChildren + Rectangular, L: Layout> std::ops::DerefMut for LayedOut<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

impl<T: WidgetChildren + Rectangular, L: Layout> Widget for LayedOut<T, L> {
    type UpdateAux = T::UpdateAux;
    type GraphicalAux = T::GraphicalAux;
    type DisplayObject = T::DisplayObject;

    fn bounds(&self) -> Rect {
        self.widget.bounds()
    }

    fn update(&mut self, aux: &mut T::UpdateAux) {
        self.widget.update(aux)
    }

    fn draw(
        &mut self,
        display: &mut dyn GraphicsDisplay<T::DisplayObject>,
        aux: &mut T::GraphicalAux,
    ) {
        self.widget.draw(display, aux)
    }
}

impl<T: WidgetChildren + Rectangular, L: Layout> draw::HasTheme for LayedOut<T, L> {
    fn theme(&mut self) -> &mut dyn draw::Themed {
        self.widget.theme()
    }

    fn resize_from_theme(&mut self, aux: &dyn GraphicalAuxiliary) {
        self.widget.resize_from_theme(aux)
    }
}

impl<T: WidgetChildren + Rectangular, L: Layout> Repaintable for LayedOut<T, L> {
    fn repaint(&mut self) {
        self.widget.repaint()
    }
}

/// Mutable and dynamic borrow of the inner content of `LayedOut`.
pub struct ActivelyLayedOut<'a, U, G, D, L: Layout> {
    pub widget: &'a mut dyn Rectangular<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    pub data: &'a mut L::ChildData,
}

/// Propagates `update` for the children of a widget.
pub fn invoke_update<U, G, D>(
    widget: &mut dyn WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
    aux: &mut U,
) {
    // Iterate in reverse because most visually forefront widgets should get events first.
    for child in widget.children_mut().into_iter().rev() {
        child.update(aux);
    }
}

/// Propagates `draw` for the children of a widget.
pub fn invoke_draw<U, G, D>(
    widget: &mut dyn WidgetChildren<UpdateAux = U, GraphicalAux = G, DisplayObject = D>,
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
