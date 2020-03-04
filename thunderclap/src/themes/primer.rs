use {
    super::Primer,
    crate::{
        base,
        draw::{self, state},
        error,
        geom::*,
    },
    reclutch::display::{
        self, Color, DisplayCommand, DisplayListBuilder, Filter, FontInfo, Gradient,
        GraphicsDisplay, GraphicsDisplayPaint, GraphicsDisplayStroke, Rect, ResourceData,
        ResourceDescriptor, ResourceReference, SharedData, Size, StyleColor, TextDisplayItem,
        Vector, VectorPath, VectorPathBuilder,
    },
};

fn check_mark_icon(rect: Rect) -> VectorPath {
    let mut builder = VectorPathBuilder::new();

    // start at top-right
    builder.move_to(rect.origin + Size::new(rect.size.width, 0.0));
    // line to bottom-middle (but a bit to the left)
    builder.line_to(rect.origin + Size::new((rect.size.width / 2.0) - 2.0, rect.size.height));
    // line to left-middle
    builder.line_to(rect.origin + Size::new(0.0, rect.size.height / 2.0));

    builder.build()
}

impl Primer {
    /// Creates an instance of the GitHub Primer theme.
    pub fn new(display: &mut dyn GraphicsDisplay) -> Result<Self, error::ThemeError> {
        let typeface = {
            let fonts = &[
                std::sync::Arc::new(include_bytes!("assets/Inter-Regular.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-Italic.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-SemiBold.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-SemiBoldItalic.ttf").to_vec()),
            ];

            let fonts: Vec<(ResourceReference, FontInfo)> = fonts
                .iter()
                .map(|font| -> Result<(ResourceReference, FontInfo), error::ThemeError> {
                    let font_info = FontInfo::from_data(font.clone(), 0)?;
                    let font_resource = display.new_resource(ResourceDescriptor::Font(
                        ResourceData::Data(SharedData::RefCount(font.clone())),
                    ))?;

                    Ok((font_resource, font_info))
                })
                .collect::<Result<Vec<_>, _>>()?;

            draw::Typeface {
                regular: fonts[0].clone(),
                italic: fonts[1].clone(),
                bold: fonts[2].clone(),
                bold_italic: fonts[3].clone(),
            }
        };

        Ok(Primer {
            data: draw::ThemeData {
                scheme: draw::ColorScheme {
                    background: base::color_from_urgba(255, 255, 255, 1.0),
                    error: base::color_from_urgba(211, 50, 63, 1.0),
                    focus: base::color_from_urgba(3, 102, 214, 0.3),
                    primary: base::color_from_urgba(46, 186, 78, 1.0),
                    control_outset: base::color_from_urgba(244, 247, 249, 1.0),
                    control_inset: base::color_from_urgba(255, 255, 255, 1.0),
                    over_error: base::color_from_urgba(255, 255, 255, 1.0),
                    over_focus: base::color_from_urgba(255, 255, 255, 1.0),
                    over_primary: base::color_from_urgba(255, 255, 255, 1.0),
                    over_control_outset: base::color_from_urgba(36, 41, 46, 1.0),
                    over_control_inset: base::color_from_urgba(36, 41, 46, 1.0),
                },
                typography: draw::Typography {
                    header: draw::TypefaceStyle {
                        typeface: typeface.clone(),
                        size: 32.0,
                        style: draw::TextStyle::Bold,
                    },
                    sub_header: draw::TypefaceStyle {
                        typeface: typeface.clone(),
                        size: 24.0,
                        style: draw::TextStyle::Bold,
                    },
                    body: draw::TypefaceStyle {
                        typeface: typeface.clone(),
                        size: 16.0,
                        style: draw::TextStyle::Regular,
                    },
                    button: draw::TypefaceStyle {
                        typeface,
                        size: 12.0,
                        style: draw::TextStyle::Bold,
                    },
                },
                contrast: draw::ThemeContrast::Light,
            },
        })
    }
}

impl draw::Theme for Primer {
    fn button(&self) -> Box<dyn draw::Painter<state::ButtonState>> {
        Box::new(ButtonPainter)
    }

    fn checkbox(&self) -> Box<dyn draw::Painter<state::CheckboxState>> {
        Box::new(CheckboxPainter)
    }

    fn text_area(&self) -> Box<dyn draw::Painter<state::TextAreaState>> {
        Box::new(TextAreaPainter)
    }

    fn scroll_bar(&self) -> Box<dyn draw::Painter<state::ScrollBarState>> {
        Box::new(ScrollBarPainter)
    }

    fn data(&self) -> &draw::ThemeData {
        &self.data
    }
}

struct ButtonPainter;

impl ButtonPainter {
    fn make_text_item(
        &self,
        state: &state::ButtonState,
        color: StyleColor,
        centered: bool,
    ) -> TextDisplayItem {
        let typeface = state.data.typeface.typeface.pick(state.data.typeface.style);
        let mut text_item = TextDisplayItem {
            text: state.data.text.clone(),
            font: typeface.0,
            font_info: typeface.1,
            size: state.data.typeface.size,
            bottom_left: Default::default(),
            color,
        };

        text_item.set_top_left(if centered {
            display::center(text_item.bounds().unwrap().size, state.rect.cast_unit())
        } else {
            state.rect.origin.cast_unit()
        });

        text_item
    }
}

impl draw::Painter<state::ButtonState> for ButtonPainter {
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::ButtonState>> {
        theme.button()
    }

    fn size_hint(&self, state: state::ButtonState) -> Size {
        self.make_text_item(&state, Color::default().into(), false)
            .bounds()
            .unwrap()
            .inflate(10.0, 5.0)
            .size
    }

    fn paint_hint(&self, rect: RelativeRect) -> RelativeRect {
        // account for focus border
        rect.inflate(3.25, 3.25)
    }

    fn mouse_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect
    }

    fn draw(&mut self, state: state::ButtonState) -> Vec<DisplayCommand> {
        let (background, border, text, focus) = if state.data.disabled {
            (
                draw::strengthen(state.data.background, 0.2, state.data.contrast).into(),
                draw::weaken(state.data.color, 0.4, state.data.contrast).into(),
                draw::weaken(state.data.color, 0.4, state.data.contrast).into(),
                state.data.focus.into(),
            )
        } else if state.interaction.contains(state::InteractionState::PRESSED) {
            let background = draw::strengthen(state.data.background, 0.2, state.data.contrast);
            (
                background.into(),
                draw::weaken(state.data.color, 0.3, state.data.contrast).into(),
                state.data.color.into(),
                state.data.focus.into(),
            )
        } else if state.interaction.contains(state::InteractionState::HOVERED) {
            let background = draw::strengthen(state.data.background, 0.1, state.data.contrast);

            (
                StyleColor::LinearGradient(Gradient {
                    start: state.rect.origin.cast_unit(),
                    end: state.rect.origin.cast_unit() + Size::new(0.0, state.rect.size.height),
                    stops: vec![
                        (0.0, draw::lighten(background, 0.1)),
                        (0.9, draw::darken(background, 0.1)),
                    ],
                }),
                draw::weaken(state.data.color, 0.3, state.data.contrast).into(),
                state.data.color.into(),
                state.data.focus.into(),
            )
        } else {
            (
                StyleColor::LinearGradient(Gradient {
                    start: state.rect.origin.cast_unit(),
                    end: state.rect.origin.cast_unit() + Size::new(0.0, state.rect.size.height),
                    stops: vec![
                        (0.0, draw::lighten(state.data.background, 0.1)),
                        (0.9, draw::darken(state.data.background, 0.1)),
                    ],
                }),
                draw::weaken(state.data.color, 0.4, state.data.contrast).into(),
                state.data.color.into(),
                state.data.focus.into(),
            )
        };

        let text_item = self.make_text_item(&state, text, true);

        let mut builder = DisplayListBuilder::new();

        // Background
        builder.push_round_rectangle(
            base::sharp_align(state.rect.cast_unit()),
            [3.5; 4],
            GraphicsDisplayPaint::Fill(background),
            None,
        );

        // Border
        builder.push_round_rectangle(
            base::sharp_align(state.rect.cast_unit()),
            [3.5; 4],
            GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                thickness: 1.0 / 3.0,
                color: border,
                ..Default::default()
            }),
            None,
        );

        // Text
        builder.push_text(text_item, None);

        // Focus rect
        if state.interaction.contains(state::InteractionState::FOCUSED)
            && !state.interaction.contains(state::InteractionState::PRESSED)
        {
            builder.push_round_rectangle(
                base::sharp_align(state.rect.cast_unit()).inflate(1.5, 1.5),
                [3.5; 4],
                GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                    thickness: 3.5,
                    color: focus,
                    ..Default::default()
                }),
                None,
            );
        }

        // Pressed inset shadow
        if state.interaction.contains(state::InteractionState::PRESSED) {
            builder.push_round_rectangle_clip(base::sharp_align(state.rect.cast_unit()), [3.5; 4]);
            builder.push_round_rectangle(
                state.rect.cast_unit().inflate(10.0, 10.0).translate(Vector::new(0.0, 7.0)),
                [10.0; 4],
                GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                    thickness: 10.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.2).into(),
                    ..Default::default()
                }),
                Some(Filter::Blur(3.0, 3.0)),
            );
        }

        builder.build()
    }
}

struct CheckboxPainter;

impl draw::Painter<state::CheckboxState> for CheckboxPainter {
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::CheckboxState>> {
        theme.checkbox()
    }

    fn size_hint(&self, _state: state::CheckboxState) -> Size {
        Size::new(20.0, 20.0)
    }

    fn paint_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect.inflate(3.25, 3.25)
    }

    fn mouse_hint(&self, rect: RelativeRect) -> RelativeRect {
        RelativeRect::new(rect.origin, Size::new(20.0, 20.0).cast_unit())
    }

    fn draw(&mut self, mut state: state::CheckboxState) -> Vec<DisplayCommand> {
        state.rect.size = Size::new(20.0, 20.0).cast_unit();
        state.rect = base::sharp_align(state.rect.cast_unit()).cast_unit();

        let (background, foreground, border, focus) = if state.data.checked {
            (
                state.data.background,
                draw::weaken(state.data.foreground, 0.1, state.data.contrast).into(),
                draw::weaken(state.data.foreground, 0.4, state.data.contrast).into(),
                state.data.focus.into(),
            )
        } else if state.interaction.contains(state::InteractionState::HOVERED) {
            (
                draw::strengthen(state.data.background, 0.05, state.data.contrast),
                base::color_from_urgba(0, 0, 0, 0.0).into(),
                draw::weaken(state.data.foreground, 0.4, state.data.contrast).into(),
                state.data.focus.into(),
            )
        } else {
            (
                state.data.background,
                base::color_from_urgba(0, 0, 0, 0.0).into(),
                draw::weaken(state.data.foreground, 0.4, state.data.contrast).into(),
                state.data.focus.into(),
            )
        };

        let background = if state.interaction.contains(state::InteractionState::PRESSED) {
            draw::strengthen(background, 0.2, state.data.contrast)
        } else {
            background
        }
        .into();

        let mut builder = DisplayListBuilder::new();

        // Background
        builder.push_round_rectangle(
            state.rect.cast_unit(),
            [3.5; 4],
            GraphicsDisplayPaint::Fill(background),
            None,
        );

        // Border
        builder.push_round_rectangle(
            state.rect.cast_unit(),
            [3.5; 4],
            GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                thickness: 1.0 / 3.0,
                color: border,
                ..Default::default()
            }),
            None,
        );

        // Foreground (check mark)
        builder.push_path(
            check_mark_icon(state.rect.cast_unit().inflate(-4.0, -4.0)),
            false,
            GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                thickness: 2.5,
                color: foreground,
                ..Default::default()
            }),
            None,
        );

        // Focus rect
        if state.interaction.contains(state::InteractionState::FOCUSED)
            && !state.interaction.contains(state::InteractionState::PRESSED)
        {
            builder.push_round_rectangle(
                state.rect.cast_unit().inflate(1.5, 1.5),
                [3.5; 4],
                GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                    thickness: 3.5,
                    color: focus,
                    ..Default::default()
                }),
                None,
            );
        }

        builder.build()
    }
}

struct TextAreaPainter;

impl TextAreaPainter {
    fn make_text_item(&self, state: &state::TextAreaState, color: StyleColor) -> TextDisplayItem {
        let typeface = state.data.typeface.typeface.pick(state.data.typeface.style);

        let mut text_item = TextDisplayItem {
            text: if state.data.text.is_empty() {
                state.data.placeholder.clone()
            } else {
                state.data.text.clone()
            }
            .into(),
            font: typeface.0,
            font_info: typeface.1,
            size: state.data.typeface.size,
            bottom_left: Default::default(),
            color,
        };

        text_item.set_top_left(state.rect.origin.cast_unit());

        text_item
    }
}

impl draw::Painter<state::TextAreaState> for TextAreaPainter {
    #[inline]
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::TextAreaState>> {
        theme.text_area()
    }

    #[inline]
    fn size_hint(&self, state: state::TextAreaState) -> Size {
        self.make_text_item(&state, Color::default().into()).bounds().unwrap().size
    }

    #[inline]
    fn paint_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect
    }

    #[inline]
    fn mouse_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect
    }

    fn draw(&mut self, state: state::TextAreaState) -> Vec<DisplayCommand> {
        let text = if state.data.text.is_empty() {
            state.data.placeholder_color
        } else {
            state.data.color
        }
        .into();

        let text_item = self.make_text_item(&state, text);

        let cursor = if state.interaction.contains(state::InteractionState::FOCUSED) {
            let bounds = text_item.limited_bounds(state.data.cursor).unwrap();
            Some((bounds.origin + Size::new(bounds.size.width, 0.0), bounds.origin + bounds.size))
        } else {
            None
        };

        let mut builder = DisplayListBuilder::new();

        builder.push_rectangle_clip(state.rect.cast_unit(), true);

        if let Some((a, b)) = cursor {
            builder.push_line(
                a + Size::new(1.0, 0.0),
                b + Size::new(1.0, 0.0),
                GraphicsDisplayStroke {
                    thickness: 1.0,
                    color: state.data.cursor_color.into(),
                    ..Default::default()
                },
                None,
            );
        }

        builder.push_text(text_item, None);

        builder.build()
    }
}

struct ScrollBarPainter;

impl draw::Painter<state::ScrollBarState> for ScrollBarPainter {
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::ScrollBarState>> {
        theme.scroll_bar()
    }

    fn size_hint(&self, state: state::ScrollBarState) -> Size {
        state.rect.size.cast_unit()
    }

    fn paint_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect
    }

    fn mouse_hint(&self, rect: RelativeRect) -> RelativeRect {
        rect
    }

    fn draw(&mut self, mut state: state::ScrollBarState) -> Vec<DisplayCommand> {
        state.rect = base::sharp_align(state.rect.cast_unit()).cast_unit();
        state.scroll_bar = base::sharp_align(state.scroll_bar.cast_unit()).cast_unit();

        let foreground = if state.interaction.contains(state::InteractionState::HOVERED) {
            draw::strengthen(state.data.foreground, 0.2, state.data.contrast)
        } else {
            state.data.foreground
        };

        let border = draw::weaken(state.data.foreground, 0.4, state.data.contrast);

        let mut builder = DisplayListBuilder::new();

        // Background blur
        builder.push_round_rectangle_backdrop(
            state.rect.cast_unit(),
            [3.5; 4],
            Filter::Blur(10.0, 10.0),
        );

        // Scroll track (the background)
        builder.push_round_rectangle(
            state.rect.cast_unit(),
            [3.5; 4],
            GraphicsDisplayPaint::Fill(draw::with_opacity(state.data.background, 0.75).into()),
            None,
        );

        // Border
        builder.push_round_rectangle(
            state.rect.cast_unit(),
            [3.5; 4],
            GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                thickness: 1.0 / 3.0,
                color: border.into(),
                ..Default::default()
            }),
            None,
        );

        // Scroll bar
        builder.push_round_rectangle(
            state.rect.cast_unit(),
            [3.5; 4],
            GraphicsDisplayPaint::Fill(foreground.into()),
            None,
        );

        builder.build()
    }
}
