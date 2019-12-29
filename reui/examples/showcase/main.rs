use reui::{
    app, base, draw,
    reclutch::{
        display::{DisplayCommand, FontInfo, GraphicsDisplay, Point, Rect, Size},
        event::RcEventQueue,
        prelude::*,
    },
    themes::Primer,
    ui,
};

#[macro_use]
extern crate reclutch;

#[macro_use]
extern crate reui;

#[derive(WidgetChildren, Movable)]
#[widget_children_trait(base::WidgetChildren)]
struct Showcase {
    #[widget_child]
    label: ui::Label<app::UAux, app::GAux>,
    #[widget_child]
    button_1: ui::Button<app::UAux, app::GAux>,
    #[widget_child]
    button_2: ui::Button<app::UAux, app::GAux>,
    #[widget_child]
    button_3: ui::Button<app::UAux, app::GAux>,
    #[widget_child]
    button_4: ui::Button<app::UAux, app::GAux>,
    #[widget_child]
    checkbox: ui::Checkbox<app::UAux, app::GAux>,
    #[widget_child]
    text_area: ui::TextArea<app::UAux, app::GAux>,
    #[widget_child]
    v_stack: ui::VStack<app::UAux, app::GAux>,

    visibility: base::Visibility,
    drop_event: RcEventQueue<base::DropEvent>,
    #[widget_position]
    position: Point,

    themed: draw::PhantomThemed,
}

impl Showcase {
    fn new(theme: &dyn draw::Theme, u_aux: &mut app::UAux, g_aux: &mut app::GAux) -> Self {
        let mut v_stack =
            ui::VStack::new(Rect::new(Point::new(50.0, 50.0), Size::new(200.0, 200.0)));

        let mut label = ui::LabelData {
            text: "GitHub Primer".to_string().into(),
            ..ui::LabelData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let mut button_1 = ui::ButtonData {
            text: "Outlined Button".to_string().into(),
            ..ui::ButtonData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let mut button_2 = ui::ButtonData {
            text: "Outlined Button".to_string().into(),
            ..ui::ButtonData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let mut button_3 = ui::ButtonData {
            text: "Outlined Button".to_string().into(),
            ..ui::ButtonData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let mut button_4 = ui::ButtonData {
            text: "Outlined Button".to_string().into(),
            ..ui::ButtonData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let mut checkbox = ui::CheckboxData { ..ui::CheckboxData::from_theme(theme) }
            .construct(theme, u_aux, g_aux);

        let mut text_area = ui::TextAreaData {
            placeholder: "click me and start typing!".into(),
            ..ui::TextAreaData::from_theme(theme)
        }
        .construct(theme, u_aux, g_aux);

        let v_stack_data =
            ui::VStackData { top_margin: 10.0, bottom_margin: 0.0, alignment: ui::Align::Begin };

        define_layout! {
            for v_stack => {
                v_stack_data.align(ui::Align::Stretch) => &mut label,
                v_stack_data => &mut button_1,
                v_stack_data.align(ui::Align::Middle) => &mut button_2,
                v_stack_data.align(ui::Align::End) => &mut button_3,
                v_stack_data.align(ui::Align::Stretch) => &mut button_4,
                v_stack_data => &mut checkbox,
                v_stack_data.align(ui::Align::Stretch) => &mut text_area
            }
        };

        Showcase {
            label,
            button_1,
            button_2,
            button_3,
            button_4,
            checkbox,
            text_area,
            v_stack,

            visibility: Default::default(),
            drop_event: Default::default(),
            position: Default::default(),

            themed: draw::PhantomThemed,
        }
    }
}

impl Widget for Showcase {
    type UpdateAux = app::UAux;
    type GraphicalAux = app::GAux;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut app::UAux) {
        base::invoke_update(self, aux);
    }
}

lazy_widget! {
    Showcase,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

fn main() {
    let app = app::create(
        |g_aux, display| Primer::new(g_aux, display).unwrap(),
        |u_aux, g_aux, theme| Showcase::new(theme, u_aux, g_aux),
        app::AppOptions {
            name: "Showcase".to_string(),
            warmup: 5,
            background: base::color_from_urgba(255, 255, 255, 1.0),
            window_size: Size::new(500.0, 500.0),
        },
    )
    .unwrap();

    app.start(|_| None);
}
