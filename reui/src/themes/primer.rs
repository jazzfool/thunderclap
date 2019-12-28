use {
    super::Primer,
    crate::{
        base,
        draw::{self, state},
        error, ui,
    },
    reclutch::display::{
        self, Color, DisplayCommand, DisplayListBuilder, DisplayText, Filter, FontInfo, Gradient,
        GraphicsDisplay, GraphicsDisplayPaint, GraphicsDisplayStroke, ImageData, RasterImageFormat,
        RasterImageInfo, Rect, ResourceData, ResourceDescriptor, ResourceReference, SharedData,
        Size, StyleColor, TextDisplayItem, Vector,
    },
};

const BUTTON_TEXT_SIZE: f32 = 12.0;
const LABEL_TEXT_SIZE: f32 = 14.0;

impl Primer {
    /// Creates an instance of the GitHub Primer theme.
    pub fn new<G: base::GraphicalAuxiliary>(
        g_aux: &mut G,
        display: &mut dyn GraphicsDisplay,
    ) -> Result<Self, error::ThemeError> {
        let typeface = {
            let fonts = &[
                std::sync::Arc::new(include_bytes!("assets/Inter-Regular.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-Italic.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-SemiBold.ttf").to_vec()),
                std::sync::Arc::new(include_bytes!("assets/Inter-SemiBoldItalic.ttf").to_vec()),
            ];

            let fonts: Vec<(ResourceReference, FontInfo)> = fonts
                .into_iter()
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
                    background: draw::ColorSwatch::generate(
                        base::color_from_urgba(255, 255, 255, 1.0),
                        0.3,
                    ),
                    error: draw::ColorSwatch::generate(
                        base::color_from_urgba(211, 50, 63, 1.0),
                        0.3,
                    ),
                    focus: draw::ColorSwatch::generate(
                        base::color_from_urgba(3, 102, 214, 0.3),
                        0.3,
                    ),
                    primary: draw::ColorSwatch::generate(
                        base::color_from_urgba(46, 186, 78, 1.0),
                        0.3,
                    ),
                    control_outset: draw::ColorSwatch::generate(
                        base::color_from_urgba(244, 247, 249, 1.0),
                        0.1,
                    ),
                    control_inset: draw::ColorSwatch::generate(
                        base::color_from_urgba(255, 255, 255, 1.0),
                        0.3,
                    ),
                    over_error: draw::ColorSwatch::generate(
                        base::color_from_urgba(255, 255, 255, 1.0),
                        0.3,
                    ),
                    over_focus: draw::ColorSwatch::generate(
                        base::color_from_urgba(255, 255, 255, 1.0),
                        0.3,
                    ),
                    over_primary: draw::ColorSwatch::generate(
                        base::color_from_urgba(255, 255, 255, 1.0),
                        0.3,
                    ),
                    over_control_outset: draw::ColorSwatch::generate(
                        base::color_from_urgba(36, 41, 46, 1.0),
                        0.5,
                    ),
                    over_control_inset: draw::ColorSwatch::generate(
                        base::color_from_urgba(36, 41, 46, 1.0),
                        0.3,
                    ),
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
                        typeface: typeface.clone(),
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
            text: state.data.text.clone().into(),
            font: typeface.0,
            font_info: typeface.1,
            size: state.data.typeface.size,
            bottom_left: Default::default(),
            color,
        };

        text_item.set_top_left(if centered {
            display::center(text_item.bounds().unwrap().size, state.rect)
        } else {
            state.rect.origin
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
            .inflate(10.0, 6.0)
            .size
    }

    fn paint_hint(&self, rect: Rect) -> Rect {
        // account for focus border
        rect.inflate(3.25, 3.25)
    }

    fn mouse_hint(&self, rect: Rect) -> Rect {
        rect
    }

    fn draw(&mut self, state: state::ButtonState) -> Vec<DisplayCommand> {
        let (background, border, text, focus) = if state.data.disabled {
            (
                state.data.background.strengthen_500(state.data.contrast, 1).into(),
                state.data.color.weaken_500(state.data.contrast, 3).into(),
                state.data.color.weaken_500(state.data.contrast, 3).into(),
                state.data.focus[500].into(),
            )
        } else if state.interaction.contains(state::InteractionState::PRESSED) {
            let background = state.data.background.strengthen_500(state.data.contrast, 4);
            (
                background.into(),
                state.data.color.weaken_500(state.data.contrast, 3).into(),
                state.data.color[500].into(),
                state.data.focus[500].into(),
            )
        } else if state.interaction.contains(state::InteractionState::HOVERED) {
            let background = draw::ColorSwatch::generate(
                state.data.background.strengthen_500(state.data.contrast, 2),
                0.1,
            );

            (
                StyleColor::LinearGradient(Gradient {
                    start: state.rect.origin,
                    end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                    stops: vec![(0.0, background[50]), (0.9, background[900])],
                }),
                state.data.color.weaken_500(state.data.contrast, 3).into(),
                state.data.color[500].into(),
                state.data.focus[500].into(),
            )
        } else {
            (
                StyleColor::LinearGradient(Gradient {
                    start: state.rect.origin,
                    end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                    stops: vec![
                        (0.0, state.data.background[50]),
                        (0.9, state.data.background[900]),
                    ],
                }),
                state.data.color.weaken_500(state.data.contrast, 3).into(),
                state.data.color[500].into(),
                state.data.focus[500].into(),
            )
        };

        let text_item = self.make_text_item(&state, text, true);

        let mut builder = DisplayListBuilder::new();

        // Background
        builder.push_round_rectangle(
            base::sharp_align(state.rect),
            [3.5; 4],
            GraphicsDisplayPaint::Fill(background),
            None,
        );

        // Border
        builder.push_round_rectangle(
            base::sharp_align(state.rect),
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
                base::sharp_align(state.rect).inflate(1.5, 1.5),
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
            builder.push_round_rectangle_clip(base::sharp_align(state.rect), [3.5; 4]);
            builder.push_round_rectangle(
                state.rect.inflate(10.0, 10.0).translate(Vector::new(0.0, 7.0)),
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

    fn paint_hint(&self, rect: Rect) -> Rect {
        rect.inflate(3.25, 3.25)
    }

    fn mouse_hint(&self, rect: Rect) -> Rect {
        Rect::new(rect.origin, Size::new(20.0, 20.0))
    }

    fn draw(&mut self, mut state: state::CheckboxState) -> Vec<DisplayCommand> {
        state.rect.size = Size::new(20.0, 20.0);
        vec![]
    }
}

struct TextAreaPainter;

impl TextAreaPainter {
    fn make_text_item(&self, state: &state::TextAreaState, color: StyleColor) -> TextDisplayItem {
        let typeface = state.data.typeface.typeface.pick(state.data.typeface.style);

        let mut text_item = TextDisplayItem {
            text: if state.data.text.is_empty() {
                state.data.text.clone()
            } else {
                state.data.placeholder.clone()
            }
            .into(),
            font: typeface.0,
            font_info: typeface.1,
            size: state.data.typeface.size,
            bottom_left: Default::default(),
            color,
        };

        text_item.set_top_left(state.rect.origin);

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
    fn paint_hint(&self, rect: Rect) -> Rect {
        rect
    }

    #[inline]
    fn mouse_hint(&self, rect: Rect) -> Rect {
        rect
    }

    fn draw(&mut self, state: state::TextAreaState) -> Vec<DisplayCommand> {
        let text = if state.data.text.is_empty() {
            state.data.placeholder_color[500]
        } else {
            state.data.color[500]
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

        builder.push_rectangle_clip(state.rect, true);

        if let Some((a, b)) = cursor {
            builder.push_line(
                a + Size::new(1.0, 0.0),
                b + Size::new(1.0, 0.0),
                GraphicsDisplayStroke {
                    thickness: 1.0,
                    color: state.data.cursor_color[500].into(),
                    ..Default::default()
                },
                None,
            );
        }

        builder.push_text(text_item, None);

        builder.build()
    }
}
