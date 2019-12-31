use {
    crate::{draw, geom::*},
    reclutch::{
        display::{
            Color, CommandGroup, DisplayClip, DisplayCommand, GraphicsDisplay, Point, Rect, Size,
            Vector,
        },
        event::RcEventQueue,
        prelude::*,
        widget::Widget,
    },
    std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        rc::Rc,
        sync::Mutex,
    },
};

/// Naively implements `HasVisibility`, `Repaintable`, `HasTheme` and `DropEvent` (and hence `Drop`) for a widget.
///
/// # Example
/// ```ignore
/// struct LazyWidget {
///     visibility: Visibility,
///     themed: PhantomThemed,
///     drop_event: RcEventQueue<()>,
///     position: Position,
/// }
///
/// lazy_widget! {
///     LazyWidget,
///     visibility: visibility,
///     theme: themed,
///     drop_event: drop_event,
///     position: position
/// }
/// ```
///
/// This macro can also implement for generic widgets. Generic widgets within Reui follow a strict pattern:
/// ```ignore
/// // The name of the generics (U and G) are important.
/// struct GenericWidget<U: UpdateAuxiliary, G: GraphicalAuxiliary> { /* ... */ }
/// ```
/// Which then can be plugged into this macro like so:
/// ```ignore
/// lazy_widget! {
///     generic GenericWidget,
///     visibility: visibility,
///     theme: themed,
///     drop_event: drop_event,
///     position: position
/// }
/// ```
#[macro_export]
macro_rules! lazy_widget {
    ($name:ty,visibility:$vis:ident,theme:$thm:ident,drop_event:$de:ident) => {
        impl $crate::base::HasVisibility for $name {
            #[inline(always)]
            fn set_visibility(&mut self, visibility: $crate::base::Visibility) {
                self.$vis = visibility
            }

            #[inline(always)]
            fn visibility(&self) -> $crate::base::Visibility {
                self.$vis
            }
        }

        impl $crate::base::Repaintable for $name {
            #[inline]
            fn repaint(&mut self) {
                for child in $crate::base::WidgetChildren::children_mut(self) {
                    child.repaint();
                }
            }
        }

        impl $crate::draw::HasTheme for $name {
            #[inline(always)]
            fn theme(&mut self) -> &mut dyn $crate::draw::Themed {
                &mut self.$thm
            }

            #[inline(always)]
            fn resize_from_theme(&mut self) {}
        }

        impl $crate::base::DropNotifier for $name {
            #[inline(always)]
            fn drop_event(
                &self,
            ) -> &$crate::reclutch::event::RcEventQueue<$crate::base::DropEvent> {
                &self.$de
            }
        }

        impl Drop for $name {
            #[inline]
            fn drop(&mut self) {
                self.$de.emit_owned($crate::base::DropEvent);
            }
        }
    };
    (generic $name:tt,visibility:$vis:ident,theme:$thm:ident,drop_event:$de:ident) => {
        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::HasVisibility for $name<U, G>
        {
            #[inline(always)]
            fn set_visibility(&mut self, visibility: $crate::base::Visibility) {
                self.$vis = visibility
            }

            #[inline(always)]
            fn visibility(&self) -> $crate::base::Visibility {
                self.$vis
            }
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::Repaintable for $name<U, G>
        {
            #[inline]
            fn repaint(&mut self) {
                for child in $crate::base::WidgetChildren::children_mut(self) {
                    child.repaint();
                }
            }
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::draw::HasTheme for $name<U, G>
        {
            #[inline(always)]
            fn theme(&mut self) -> &mut dyn $crate::draw::Themed {
                &mut self.$thm
            }

            #[inline(always)]
            fn resize_from_theme(&mut self) {}
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::base::DropNotifier for $name<U, G>
        {
            #[inline(always)]
            fn drop_event(
                &self,
            ) -> &$crate::reclutch::event::RcEventQueue<$crate::base::DropEvent> {
                &self.$de
            }
        }

        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary> Drop
            for $name<U, G>
        {
            #[inline]
            fn drop(&mut self) {
                self.$de.emit_owned($crate::base::DropEvent);
            }
        }
    };
}

/// Most straight-forward implementation of `Widget`: `update` and `draw` are propagated to children.
///
/// # Example
/// ```ignore
/// struct MyWidget;
/// lazy_propagate! {
///     MyWidget,
///     update_aux: MyUpdateAux,
///     graphical_aux: MyGraphicalAux
/// }
/// ```
/// Rules for generic widgets are the same as the ones described in `lazy_widget!`:
/// ```ignore
/// lazy_propagate! {
///     generic MyGenericWidget
///     // notice we don't supply the aux types; that's the point of generic widgets.
/// }
/// ```
#[macro_export]
macro_rules! lazy_propagate {
    ($name:ty,update_aux:$ua:ty,graphical_aux:$ga:ty) => {
        impl $crate::reclutch::Widget for $name {
            type UpdateAux = $ua;
            type GraphicalAux = $ga;
            type DisplayObject = $crate::reclutch::display::DisplayCommand;

            fn update(&mut self, aux: &mut $ua) {
                $crate::base::invoke_update(self, aux);
            }

            fn draw(&mut self, display: $crate::reclutch::display::GraphicsDisplay, aux: &mut $ga) {
                $crate::base::invoke_draw(self, display, aux);
            }
        }
    };
    (generic $name:ty) => {
        impl<U: $crate::base::UpdateAuxiliary, G: $crate::base::GraphicalAuxiliary>
            $crate::reclutch::Widget for $name<U, G>
        {
            type UpdateAux = U;
            type GraphicalAux = G;
            type DisplayObject = $crate::reclutch::display::DisplayCommand;

            fn update(&mut self, aux: &mut U) {
                $crate::base::invoke_update(self, aux);
            }

            fn draw(&mut self, display: $crate::reclutch::display::GraphicsDisplay, aux: &mut G) {
                $crate::base::invoke_draw(self, display, aux);
            }
        }
    };
}

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
pub trait WidgetChildren:
    Widget + draw::HasTheme + Repaintable + HasVisibility + ContextuallyMovable
{
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
    fn set_position(&mut self, position: RelativePoint);
    /// Returns the current position of the widget.
    fn position(&self) -> RelativePoint;
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
    fn set_rect(&mut self, rect: RelativeRect);
    /// Returns the rectangular bounds.
    ///
    /// If `Rectangular` is a blanket implementation, then this is simply a constructor
    /// for `Rect` based on the values returned from `position()` and `size()`.
    fn rect(&self) -> RelativeRect;
}

impl<T> Rectangular for T
where
    T: Widget + Movable + Resizable,
{
    #[inline]
    fn set_rect(&mut self, rect: RelativeRect) {
        self.set_position(rect.origin);
        self.set_size(rect.size.cast_unit());
    }

    #[inline]
    fn rect(&self) -> RelativeRect {
        RelativeRect::new(self.position(), self.size().cast_unit())
    }
}

/// Describes the interactivity/visibility condition of a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Is rendered and receives updates.
    Normal,
    /// Receives updates but isn't rendered.
    Invisible,
    /// Is rendered but doesn't receive updates.
    Static,
    /// Is neither rendered nor updated.
    None,
}

impl Default for Visibility {
    #[inline]
    fn default() -> Self {
        Visibility::Normal
    }
}

/// Implemented by widgets which are capable of tracking visibility.
pub trait HasVisibility {
    fn set_visibility(&mut self, visibility: Visibility);
    fn visibility(&self) -> Visibility;
}

/// Trait required for any type passed as the `UpdateAux` type (seen as `U` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait UpdateAuxiliary: 'static {
    /// Returns the queue where window events (`WindowEvent`) are emitted, immutably.
    fn window_queue(&self) -> &RcEventQueue<WindowEvent>;
    /// Returns the queue where window events (`WindowEvent`) are emitted, mutably.
    fn window_queue_mut(&mut self) -> &mut RcEventQueue<WindowEvent>;
}

/// Trait required for any type passed as the `GraphicalAux` type (seen as `G` in the widget type parameters)
/// with accessors required for usage within Reui-implemented widgets.
pub trait GraphicalAuxiliary: 'static {
    /// Returns the HiDPI scaling factor.
    fn scaling(&self) -> f32;
}

#[derive(Clone, Debug, PartialEq)]
struct ConsumableEventInner<T> {
    marker: RefCell<bool>,
    data: T,
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
#[derive(Debug, PartialEq)]
pub struct ConsumableEvent<T>(Rc<ConsumableEventInner<T>>);

impl<T> ConsumableEvent<T> {
    /// Creates a unconsumed event, initialized with `val`.
    pub fn new(val: T) -> Self {
        ConsumableEvent(Rc::new(ConsumableEventInner { marker: RefCell::new(true), data: val }))
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
        let mut is_consumed = self.0.marker.borrow_mut();
        if *is_consumed && pred(&self.0.data) {
            *is_consumed = false;
            Some(&self.0.data)
        } else {
            None
        }
    }

    /// Returns the inner event data regardless of consumption.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.0.data
    }
}

impl<T> Clone for ConsumableEvent<T> {
    fn clone(&self) -> Self {
        ConsumableEvent(self.0.clone())
    }
}

/// An event related to the window, e.g. input.
#[derive(PipelineEvent, Debug, Clone, PartialEq)]
#[reui_crate(crate)]
pub enum WindowEvent {
    /// The user pressed a mouse button.
    #[event_key(mouse_press)]
    MousePress(ConsumableEvent<(AbsolutePoint, MouseButton, KeyModifiers)>),
    /// The user released a mouse button.
    /// This event complements `MousePress`, which means it realistically can only
    /// be emitted after `MousePress` has been emitted.
    #[event_key(mouse_release)]
    MouseRelease(ConsumableEvent<(AbsolutePoint, MouseButton, KeyModifiers)>),
    /// The user moved the cursor.
    #[event_key(mouse_move)]
    MouseMove(ConsumableEvent<(AbsolutePoint, KeyModifiers)>),
    /// Emitted when a text input is received.
    #[event_key(text_input)]
    TextInput(ConsumableEvent<char>),
    /// Emitted when a key is pressed.
    #[event_key(key_press)]
    KeyPress(ConsumableEvent<(KeyInput, KeyModifiers)>),
    /// Emitted when a key is released.
    #[event_key(key_release)]
    KeyRelease(ConsumableEvent<(KeyInput, KeyModifiers)>),
    /// Emitted immediately before an event which is capable of changing focus.
    /// If implementing a focus-able widget, to handle this event, simply clear
    /// the local "focused" flag (which should ideally be stored as `draw::state::InteractionState`).
    #[event_key(clear_focus)]
    ClearFocus,
}

// Most of these are copied from `winit`.
// We can't reuse the `winit` types because `winit` is an optional dependency (app feature).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

/// Button on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Key on a keyboard.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyInput {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

/// Information about a parent layout with a queue which receives updated rectangles.
#[derive(Debug)]
pub struct WidgetLayoutEventsInner {
    pub id: u64,
    pub evq: reclutch::event::bidir_single::Secondary<AbsoluteRect, AbsoluteRect>,
}

/// Helper layout over `WidgetLayoutEventsInner`; optionally stores information about a parent layout.
#[derive(Default, Debug)]
pub struct WidgetLayoutEvents(Option<WidgetLayoutEventsInner>);

impl WidgetLayoutEvents {
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates `WidgetLayoutEvents` from the given layout information.
    pub fn from_layout(layout: WidgetLayoutEventsInner) -> Self {
        WidgetLayoutEvents(Some(layout))
    }

    /// Possibly returns the inner associated layout ID.
    pub fn id(&self) -> Option<u64> {
        self.0.as_ref().map(|inner| inner.id)
    }

    /// Possibly updates the layout information.
    pub fn update(&mut self, layout: impl Into<Option<WidgetLayoutEventsInner>>) {
        self.0 = layout.into();
    }

    /// Notifies the layout that the widget rectangle has been updated from the widget side.
    pub fn notify(&mut self, rect: AbsoluteRect) {
        if let Some(inner) = &mut self.0 {
            inner.evq.emit_owned(rect);
        }
    }

    /// Returns the most up-to-date widget rectangle from the layout.
    pub fn receive(&mut self) -> Option<AbsoluteRect> {
        self.0.as_mut().and_then(|inner| inner.evq.retrieve_newest())
    }
}

/// Widget that is capable of listening to layout events.
pub trait LayableWidget: WidgetChildren + ContextuallyRectangular + DropNotifier {
    fn listen_to_layout(&mut self, layout: impl Into<Option<WidgetLayoutEventsInner>>);
    fn layout_id(&self) -> Option<u64>;
}

/// Widget which emits layout events to registered widgets.
pub trait Layout: WidgetChildren + Rectangular + Sized {
    type PushData;

    /// "Registers" a widget to the layout.
    fn push(&mut self, data: Self::PushData, child: &mut impl LayableWidget);

    /// De-registers a widget from the layout, optionally restoring the original widget rectangle.
    fn remove(&mut self, child: &mut impl LayableWidget, restore_original: bool);
}

/// Empty event indicating `Observed` data has changed.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reui_crate(crate)]
#[event_key(drop)]
pub struct DropEvent;

/// Widget which has an event queue where a single event is emitted when the widget is dropped.
pub trait DropNotifier: Widget {
    fn drop_event(&self) -> &RcEventQueue<DropEvent>;
}

/// Empty event indicating `Observed` data has changed.
#[derive(PipelineEvent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reui_crate(crate)]
#[event_key(change)]
pub struct ObservedEvent;

/// Wrapper which emits an event whenever the inner variable is changed.
#[derive(Debug)]
pub struct Observed<T: Sized> {
    pub on_change: RcEventQueue<ObservedEvent>,

    inner: T,
}

impl<T: Sized> Observed<T> {
    pub fn new(val: T) -> Self {
        Observed { on_change: RcEventQueue::new(), inner: val }
    }

    /// Updates the inner variable.
    /// Emits an event to `on_change` when invoked.
    #[inline]
    pub fn set(&mut self, val: T) {
        self.inner = val;
        self.on_change.emit_owned(ObservedEvent);
    }

    /// Returns an immutable reference to the inner variable.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the inner variable.
    /// Emits an event to `on_change` when invoked.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.on_change.emit_owned(ObservedEvent);
        &mut self.inner
    }
}

impl<T: Sized> std::ops::Deref for Observed<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: Sized> std::ops::DerefMut for Observed<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.on_change.emit_owned(ObservedEvent);
        &mut self.inner
    }
}

#[macro_export]
macro_rules! observe {
    ($($x:ident),*) => {
        $(let $x = $crate::base::Observed::new($x);)*
    };
}

/// Propagates `update` to the children of a widget.
pub fn invoke_update<U: UpdateAuxiliary, G>(
    widget: &mut dyn WidgetChildren<
        UpdateAux = U,
        GraphicalAux = G,
        DisplayObject = DisplayCommand,
    >,
    aux: &mut U,
) {
    // Iterate in reverse because most visually forefront widgets should get events first.
    for child in widget.children_mut().into_iter().rev() {
        if child.visibility() != Visibility::Static && child.visibility() != Visibility::None {
            child.update(aux);
        }
    }
}

lazy_static::lazy_static! {
    // Frame counter used by `invoke_draw`, resets back to 0 after 60 frames.
    // This is used to only clean up `CLIP_LIST` every 60 frames.
    static ref DRAW_COUNTER: Mutex<u8> = Mutex::new(0);
    // Map of pre/post command groups loosely linked to a widget by using the memory address as a unique identifier.
    static ref CLIP_LIST: Mutex<HashMap<usize, (CommandGroup, CommandGroup)>> =
        Mutex::new(HashMap::new());
}

fn invoke_draw_impl<U, G: GraphicalAuxiliary>(
    widget: &mut dyn WidgetChildren<
        UpdateAux = U,
        GraphicalAux = G,
        DisplayObject = DisplayCommand,
    >,
    display: &mut dyn GraphicsDisplay,
    aux: &mut G,
    clip_list: &mut HashMap<usize, (CommandGroup, CommandGroup)>,
    checked: &mut Option<HashSet<usize>>,
) {
    if widget.visibility() != Visibility::Invisible && widget.visibility() != Visibility::None {
        let id = widget as *const _ as *const usize as _;
        let (clip, restore) =
            clip_list.entry(id).or_insert_with(|| (CommandGroup::new(), CommandGroup::new()));
        let clip_rect = widget.abs_bounds();
        clip.repaint();
        restore.repaint();
        clip.push(
            display,
            &[
                DisplayCommand::Save,
                DisplayCommand::Clip(DisplayClip::Rectangle {
                    rect: clip_rect.cast_unit(),
                    antialias: true,
                }),
                DisplayCommand::Save,
            ],
            false,
            None,
        );

        widget.draw(display, aux);

        restore.push(display, &[DisplayCommand::Restore, DisplayCommand::Restore], false, None);

        if let Some(ref mut checked) = *checked {
            checked.insert(id);
        }
    }

    for child in widget.children_mut() {
        invoke_draw_impl(child, display, aux, clip_list, checked);
    }
}

/// Recursively invokes `draw`.
/// This will invoke draw (with some extra steps, see below)
/// for `widget`, then invoke `invoke_draw` all of `widget`s children.
///
/// Extra processing steps:
/// - Skip if widget visibility is `Invisible` or `None`.
/// - Clip to absolute widget bounds.
/// - Add widget position to auxiliary tracer.
pub fn invoke_draw<U, G: GraphicalAuxiliary>(
    widget: &mut dyn WidgetChildren<
        UpdateAux = U,
        GraphicalAux = G,
        DisplayObject = DisplayCommand,
    >,
    display: &mut dyn GraphicsDisplay,
    aux: &mut G,
) {
    let mut draw_counter = DRAW_COUNTER.lock().unwrap();
    let mut clip_list = CLIP_LIST.lock().unwrap();

    // Every 60 frames clean up CLIP_LIST.
    // To do so, gather information on which widget ptrs have been maintained.
    let mut checked = if *draw_counter >= 60 { Some(HashSet::new()) } else { None };

    invoke_draw_impl(widget, display, aux, &mut clip_list, &mut checked);

    // Perform cleanup (checked is only contains a value if on 60th frame).
    if let Some(checked) = checked {
        *draw_counter = 0;
        clip_list.retain(|widget_ptr, _| checked.contains(widget_ptr));
    }

    *draw_counter += 1;
}

/// Creates a color from 3 unsigned 8-bit components and an `f32` alpha.
/// This replicates CSS syntax (e.g. `rgba(28, 196, 54, 0.3)`).
pub fn color_from_urgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Aligns a rectangle with regards to Skia anti-aliasing.
pub fn sharp_align(rect: Rect) -> Rect {
    rect.round_in().inflate(0.5, 0.5)
}
