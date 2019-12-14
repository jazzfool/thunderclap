//! The main part of Reui; a widget toolkit built atop Reclutch.

pub mod button;
pub mod vstack;

pub use {button::*, vstack::*};

use {
    crate::{base, draw},
    reclutch::display,
};

#[macro_export]
macro_rules! define_layout {
    (for $layout:expr => {$($data:expr => $target:expr),*}) => {
        {
            use $crate::base::Layout;
            $(
                $layout.push($data, &mut $target);
            )*
        }
    }
}

#[macro_export]
macro_rules! define_layouts {
    ($(for $layout:expr => {$($data:expr => $target:expr),*}),*) => {
        {
            $(
                define_layout! {
                    for $layout => {
                        $($data => $target),*
                    }
                };
            )*
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggledEvent<T> {
    Start(T),
    Stop(T),
}

impl<T> ToggledEvent<T> {
    pub fn new(is_start: bool, data: T) -> Self {
        if is_start {
            ToggledEvent::Start(data)
        } else {
            ToggledEvent::Stop(data)
        }
    }

    #[inline]
    pub fn is_start(&self) -> bool {
        match self {
            ToggledEvent::Start(_) => true,
            ToggledEvent::Stop(_) => false,
        }
    }

    #[inline]
    pub fn data(&self) -> &T {
        match self {
            ToggledEvent::Start(ref x) | ToggledEvent::Stop(ref x) => x,
        }
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        match self {
            ToggledEvent::Start(ref mut x) | ToggledEvent::Stop(ref mut x) => x,
        }
    }

    #[inline]
    pub fn into_data(self) -> T {
        match self {
            ToggledEvent::Start(x) | ToggledEvent::Stop(x) => x,
        }
    }
}

/// How a child should be aligned within a layout.
/// On which axis the align applies to depends on the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// The child is aligned to the beginning of the layout.
    Begin,
    /// The child is centered.
    Middle,
    /// The child is aligned to the end of the layout.
    End,
    /// The child is stretched to fill the container.
    Stretch,
}

impl Default for Align {
    fn default() -> Self {
        Align::Begin
    }
}

pub fn simple_button<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary>(
    text: String,
    theme: &dyn draw::Theme,
    update_aux: &mut U,
    gfx_aux: &mut G,
) -> Button<U, G> {
    Button::new(
        display::DisplayText::Simple(text),
        display::Point::default(),
        None,
        draw::state::ButtonType::Normal,
        false,
        theme,
        update_aux,
        gfx_aux,
    )
}
