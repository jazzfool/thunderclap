use {
    super::{recolor_icon, Primer},
    crate::{
        base,
        draw::{self, state},
        error,
    },
    reclutch::display::{
        self, Color, DisplayCommand, DisplayListBuilder, Filter, Gradient, GraphicsDisplay,
        GraphicsDisplayPaint, GraphicsDisplayStroke, ImageData, RasterImageFormat, RasterImageInfo,
        Rect, ResourceData, ResourceDescriptor, ResourceReference, SharedData, Size, StyleColor,
        TextDisplayItem, Vector,
    },
};

const BUTTON_TEXT_SIZE: f32 = 12.0;
const LABEL_TEXT_SIZE: f32 = 14.0;

impl Primer {
    pub fn new(display: &mut dyn GraphicsDisplay) -> Result<Self, error::ThemeError> {
        let mut img: image::RgbaImage =
            image::load_from_memory(include_bytes!("checkmark.png"))?.to_rgba();

        recolor_icon(
            &mut img,
            Color::new(1.0, 1.0, 1.0, 1.0),
            Color::new(0.0, 0.0, 0.0, 1.0),
        );

        let checkmark = display.new_resource(ResourceDescriptor::Image(ImageData::Raw(
            ResourceData::Data(SharedData::RefCount(std::sync::Arc::new(
                img.clone().into_raw(),
            ))),
            RasterImageInfo {
                size: (128, 128),
                format: RasterImageFormat::Rgba8,
            },
        )))?;

        recolor_icon(
            &mut img,
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 1.0, 1.0),
        );

        Ok(Primer { checkmark })
    }
}

impl draw::Theme for Primer {
    fn button(&self) -> Box<dyn draw::Painter<state::ButtonState>> {
        Box::new(ButtonPainter)
    }

    fn checkbox(&self) -> Box<dyn draw::Painter<state::CheckboxState>> {
        Box::new(CheckboxPainter {
            checkmark: self.checkmark.clone(),
        })
    }

    fn label_color(&self) -> StyleColor {
        base::color_from_urgba(36, 41, 46, 1.0).into()
    }

    fn default_text_size(&self) -> f32 {
        LABEL_TEXT_SIZE
    }
}

struct ButtonPainter;

impl ButtonPainter {
    fn make_text_item(
        state: &state::ButtonState,
        aux: &dyn base::GraphicalAuxiliary,
        color: Color,
        centered: bool,
    ) -> TextDisplayItem {
        let font = aux.semibold_ui_font();

        let mut text = TextDisplayItem {
            text: state.text.clone(),
            font: font.0,
            font_info: font.1,
            size: state.text_size.unwrap_or(BUTTON_TEXT_SIZE),
            bottom_left: Default::default(),
            color: color.into(),
        };

        text.set_top_left(if centered {
            display::center(text.bounds().unwrap().size, state.rect)
        } else {
            state.rect.origin
        });

        text
    }
}

impl draw::Painter<state::ButtonState> for ButtonPainter {
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::ButtonState>> {
        theme.button()
    }

    fn size_hint(&self, state: state::ButtonState, aux: &dyn base::GraphicalAuxiliary) -> Size {
        ButtonPainter::make_text_item(&state, aux, Color::default(), false)
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

    fn draw(
        &mut self,
        state: state::ButtonState,
        aux: &dyn base::GraphicalAuxiliary,
    ) -> Vec<DisplayCommand> {
        let mut interaction_state = state::InteractionState::empty();

        let (background, border, text, focus) = match state.button_type {
            state::ButtonType::Normal => match state.state {
                state::ControlState::Normal(interaction) => {
                    interaction_state = interaction;

                    if interaction.intersects(
                        state::InteractionState::HOVERED | state::InteractionState::PRESSED,
                    ) {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(239, 243, 246, 1.0)),
                                    (0.9, base::color_from_urgba(230, 235, 241, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(27, 31, 35, 0.35).into(),
                            base::color_from_urgba(36, 41, 46, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 0.3).into(),
                        )
                    } else {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(250, 251, 252, 1.0)),
                                    (0.9, base::color_from_urgba(239, 243, 246, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(27, 31, 35, 0.35).into(),
                            base::color_from_urgba(36, 41, 46, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 0.3).into(),
                        )
                    }
                }
                state::ControlState::Disabled => (
                    base::color_from_urgba(239, 243, 246, 1.0).into(),
                    base::color_from_urgba(27, 31, 35, 0.2).into(),
                    base::color_from_urgba(36, 41, 46, 0.4).into(),
                    base::color_from_urgba(3, 102, 214, 0.3).into(),
                ),
            },
            state::ButtonType::Primary => match state.state {
                state::ControlState::Normal(interaction) => {
                    interaction_state = interaction;

                    if interaction.contains(state::InteractionState::PRESSED) {
                        (
                            base::color_from_urgba(39, 159, 67, 1.0).into(),
                            base::color_from_urgba(27, 31, 35, 0.5).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(46, 200, 82, 0.5).into(),
                        )
                    } else if interaction.contains(state::InteractionState::HOVERED) {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(46, 200, 82, 1.0)),
                                    (0.9, base::color_from_urgba(38, 159, 66, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(27, 31, 35, 0.5).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(46, 200, 82, 0.5).into(),
                        )
                    } else {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(52, 208, 88, 1.0)),
                                    (0.9, base::color_from_urgba(40, 167, 69, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(27, 31, 35, 0.5).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(46, 200, 82, 0.5).into(),
                        )
                    }
                }
                state::ControlState::Disabled => (
                    base::color_from_urgba(148, 211, 162, 1.0).into(),
                    base::color_from_urgba(27, 31, 35, 0.5).into(),
                    base::color_from_urgba(255, 255, 255, 0.75).into(),
                    base::color_from_urgba(46, 200, 82, 0.5).into(),
                ),
            },
            state::ButtonType::Danger => match state.state {
                state::ControlState::Normal(interaction) => {
                    interaction_state = interaction;

                    if interaction.contains(state::InteractionState::PRESSED) {
                        (
                            base::color_from_urgba(181, 32, 44, 1.0).into(),
                            base::color_from_urgba(104, 32, 40, 1.0).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(181, 32, 44, 0.4).into(),
                        )
                    } else if interaction.contains(state::InteractionState::HOVERED) {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(221, 66, 78, 1.0)),
                                    (0.9, base::color_from_urgba(203, 36, 49, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(104, 32, 40, 1.0).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(181, 32, 44, 0.4).into(),
                        )
                    } else {
                        (
                            StyleColor::LinearGradient(Gradient {
                                start: state.rect.origin,
                                end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                                stops: vec![
                                    (0.0, base::color_from_urgba(250, 251, 252, 1.0)),
                                    (0.9, base::color_from_urgba(239, 243, 246, 1.0)),
                                ],
                            }),
                            base::color_from_urgba(27, 31, 35, 0.35).into(),
                            base::color_from_urgba(203, 36, 49, 1.0).into(),
                            base::color_from_urgba(181, 32, 44, 0.4).into(),
                        )
                    }
                }
                state::ControlState::Disabled => (
                    base::color_from_urgba(239, 243, 246, 1.0).into(),
                    base::color_from_urgba(27, 31, 35, 0.2).into(),
                    base::color_from_urgba(203, 36, 49, 0.4).into(),
                    base::color_from_urgba(3, 102, 214, 0.3).into(),
                ),
            },
            state::ButtonType::Outline => match state.state {
                state::ControlState::Normal(interaction) => {
                    interaction_state = interaction;

                    if interaction.intersects(
                        state::InteractionState::PRESSED | state::InteractionState::HOVERED,
                    ) {
                        (
                            base::color_from_urgba(3, 102, 214, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 1.0).into(),
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 0.5).into(),
                        )
                    } else if interaction.contains(state::InteractionState::FOCUSED) {
                        (
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 0.5).into(),
                        )
                    } else {
                        (
                            base::color_from_urgba(255, 255, 255, 1.0).into(),
                            base::color_from_urgba(27, 31, 35, 0.35).into(),
                            base::color_from_urgba(3, 102, 214, 1.0).into(),
                            base::color_from_urgba(3, 102, 214, 0.5).into(),
                        )
                    }
                }
                state::ControlState::Disabled => (
                    base::color_from_urgba(255, 255, 255, 1.0).into(),
                    base::color_from_urgba(27, 31, 35, 0.35).into(),
                    base::color_from_urgba(36, 41, 46, 0.4).into(),
                    base::color_from_urgba(3, 102, 214, 0.5).into(),
                ),
            },
        };

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
                thickness: 2.0 / 3.0,
                color: border,
                ..Default::default()
            }),
            None,
        );

        // Text
        builder.push_text(ButtonPainter::make_text_item(&state, aux, text, true), None);

        // Focus rect
        if (interaction_state.contains(state::InteractionState::FOCUSED)
            && !interaction_state.contains(state::InteractionState::PRESSED))
            || (state.button_type == state::ButtonType::Outline
                && interaction_state.contains(state::InteractionState::PRESSED))
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
        if state.button_type != state::ButtonType::Outline
            && interaction_state.contains(state::InteractionState::PRESSED)
        {
            builder.push_round_rectangle_clip(base::sharp_align(state.rect), [3.5; 4]);
            builder.push_round_rectangle(
                state
                    .rect
                    .inflate(10.0, 10.0)
                    .translate(Vector::new(0.0, 7.0)),
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

struct CheckboxPainter {
    checkmark: ResourceReference,
}

impl draw::Painter<state::CheckboxState> for CheckboxPainter {
    fn invoke(&self, theme: &dyn draw::Theme) -> Box<dyn draw::Painter<state::CheckboxState>> {
        theme.checkbox()
    }

    fn size_hint(&self, _state: state::CheckboxState, _aux: &dyn base::GraphicalAuxiliary) -> Size {
        Size::new(20.0, 20.0)
    }

    fn paint_hint(&self, rect: Rect) -> Rect {
        rect.inflate(3.25, 3.25)
    }

    fn mouse_hint(&self, rect: Rect) -> Rect {
        Rect::new(rect.origin, Size::new(20.0, 20.0))
    }

    fn draw(
        &mut self,
        mut state: state::CheckboxState,
        _aux: &dyn base::GraphicalAuxiliary,
    ) -> Vec<DisplayCommand> {
        state.rect.size = Size::new(20.0, 20.0);

        let (background, checkmark_alpha, interaction) = match state.state {
            state::ControlState::Normal(interaction) => {
                if state.checked {
                    (
                        if interaction.contains(state::InteractionState::PRESSED) {
                            base::color_from_urgba(203, 208, 214, 1.0).into()
                        } else {
                            base::color_from_urgba(239, 243, 246, 1.0).into()
                        },
                        1.0,
                        interaction,
                    )
                } else if interaction.contains(state::InteractionState::PRESSED) {
                    (
                        base::color_from_urgba(203, 208, 214, 1.0).into(),
                        0.3,
                        interaction,
                    )
                } else if interaction.contains(state::InteractionState::HOVERED) {
                    (
                        base::color_from_urgba(239, 243, 246, 1.0).into(),
                        0.1,
                        interaction,
                    )
                } else {
                    (
                        base::color_from_urgba(239, 243, 246, 1.0).into(),
                        0.0,
                        interaction,
                    )
                }
            }
            _ => todo!(),
        };

        let border: StyleColor = base::color_from_urgba(27, 31, 35, 0.35).into();
        let focus: StyleColor = base::color_from_urgba(3, 102, 214, 0.3).into();

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
                thickness: 2.0 / 3.0,
                color: border,
                ..Default::default()
            }),
            None,
        );

        // Focus rect
        if interaction.contains(state::InteractionState::FOCUSED) {
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

        builder.save_layer(checkmark_alpha);

        // Checkmark
        builder.push_image(
            None,
            base::sharp_align(state.rect).inflate(-2.0, -2.0),
            self.checkmark.clone(),
            None,
        );

        builder.build()
    }
}
