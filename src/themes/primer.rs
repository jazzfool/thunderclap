use {
    super::Primer,
    crate::{
        base,
        draw::{self, state},
    },
    reclutch::display::{
        self, Color, DisplayCommand, DisplayListBuilder, Filter, Gradient, GraphicsDisplayPaint,
        GraphicsDisplayStroke, Size, StyleColor, TextDisplayItem, Vector,
    },
};

impl draw::Theme for Primer {
    fn button(&self) -> Box<dyn draw::Painter<state::ButtonState>> {
        Box::new(ButtonPainter)
    }
}

const DEFAULT_TEXT_SIZE: f32 = 12.0;

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
            size: state.text_size.unwrap_or(DEFAULT_TEXT_SIZE),
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

    fn draw(
        &mut self,
        state: state::ButtonState,
        aux: &dyn base::GraphicalAuxiliary,
    ) -> Vec<DisplayCommand> {
        let mut builder = DisplayListBuilder::new();

        let mut interaction_state = state::InteractionState::empty();

        let background = match state.state {
            state::ControlState::Normal(interaction) => {
                interaction_state = interaction;

                if interaction
                    .intersects(state::InteractionState::HOVERED | state::InteractionState::PRESSED)
                {
                    StyleColor::LinearGradient(Gradient {
                        start: state.rect.origin,
                        end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                        stops: vec![
                            (0.0, base::color_from_urgba(239, 243, 246, 1.0)),
                            (0.9, base::color_from_urgba(230, 235, 241, 1.0)),
                        ],
                    })
                } else {
                    StyleColor::LinearGradient(Gradient {
                        start: state.rect.origin,
                        end: state.rect.origin + Size::new(0.0, state.rect.size.height),
                        stops: vec![
                            (0.0, base::color_from_urgba(250, 251, 252, 1.0)),
                            (0.9, base::color_from_urgba(239, 243, 246, 1.0)),
                        ],
                    })
                }
            }
            state::ControlState::Disabled => base::color_from_urgba(239, 243, 246, 1.0).into(),
        };

        let border = match state.state {
            state::ControlState::Normal(interaction) => {
                if interaction
                    .intersects(state::InteractionState::HOVERED | state::InteractionState::PRESSED)
                {
                    base::color_from_urgba(27, 31, 35, 0.35)
                } else {
                    base::color_from_urgba(27, 31, 35, 0.2)
                }
            }
            state::ControlState::Disabled => base::color_from_urgba(27, 31, 35, 0.2),
        };

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
                thickness: (2.0 / 3.0),
                color: border.into(),
                ..Default::default()
            }),
            None,
        );

        // Text
        builder.push_text(
            ButtonPainter::make_text_item(
                &state,
                aux,
                base::color_from_urgba(36, 41, 46, 1.0),
                true,
            ),
            None,
        );

        if interaction_state.contains(state::InteractionState::FOCUSED)
            && !interaction_state.contains(state::InteractionState::PRESSED)
        {
            builder.push_round_rectangle(
                base::sharp_align(state.rect).inflate(1.5, 1.5),
                [3.5; 4],
                GraphicsDisplayPaint::Stroke(GraphicsDisplayStroke {
                    thickness: 3.5,
                    color: base::color_from_urgba(3, 102, 214, 0.3).into(),
                    ..Default::default()
                }),
                None,
            );
        }

        if interaction_state.contains(state::InteractionState::PRESSED) {
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
