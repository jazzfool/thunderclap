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

#[derive(WidgetChildren)]
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
    v_stack: ui::VStack<app::UAux, app::GAux>,

    visibility: base::Visibility,
    drop_event: RcEventQueue<base::DropEvent>,

    themed: draw::PhantomThemed,
}

impl Showcase {
    fn new(theme: &dyn draw::Theme, update_aux: &mut app::UAux, gfx_aux: &mut app::GAux) -> Self {
        let mut v_stack =
            ui::VStack::new(Rect::new(Point::new(50.0, 50.0), Size::new(200.0, 200.0)));

        let mut label =
            ui::simple_label("GitHub Primer".to_string(), theme, Default::default(), gfx_aux);

        let mut button_1 =
            ui::simple_button("Boring Button".to_string(), theme, None, None, update_aux, gfx_aux);

        let mut button_2 = ui::simple_button(
            "Important Button".to_string(),
            theme,
            Some(draw::state::ButtonType::Primary),
            None,
            update_aux,
            gfx_aux,
        );

        let mut button_3 = ui::simple_button(
            "Explode Computer".to_string(),
            theme,
            Some(draw::state::ButtonType::Danger),
            None,
            update_aux,
            gfx_aux,
        );

        let mut button_4 = ui::simple_button(
            "Outlined Button".to_string(),
            theme,
            Some(draw::state::ButtonType::Outline),
            None,
            update_aux,
            gfx_aux,
        );

        let mut checkbox =
            ui::Checkbox::new(false, false, Default::default(), theme, update_aux, gfx_aux);

        let v_stack_data =
            ui::VStackData { top_margin: 10.0, bottom_margin: 0.0, alignment: ui::Align::Begin };

        define_layout! {
            for v_stack => {
                v_stack_data.align(ui::Align::Stretch) => &mut label,
                v_stack_data => &mut button_1,
                v_stack_data.align(ui::Align::Middle) => &mut button_2,
                v_stack_data.align(ui::Align::End) => &mut button_3,
                v_stack_data.align(ui::Align::Stretch) => &mut button_4,
                v_stack_data => &mut checkbox
            }
        };

        Showcase {
            label,
            button_1,
            button_2,
            button_3,
            button_4,
            checkbox,
            v_stack,

            visibility: Default::default(),
            drop_event: Default::default(),

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

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut app::GAux) {
        base::invoke_draw(self, display, aux);
    }
}

lazy_widget! {
    Showcase,
    visibility: visibility,
    theme: themed,
    drop_event: drop_event
}

fn main() {
    let ui_font = FontInfo::from_data(
        std::sync::Arc::new(include_bytes!("../Inter-Regular.ttf").to_vec()),
        0,
    )
    .unwrap();

    let semibold_font = FontInfo::from_data(
        std::sync::Arc::new(include_bytes!("../Inter-SemiBold.ttf").to_vec()),
        0,
    )
    .unwrap();

    let app = app::create(
        |display| Primer::new(display).unwrap(),
        |u_aux, g_aux, theme| Showcase::new(theme, u_aux, g_aux),
        app::AppOptions {
            name: "Showcase".to_string(),
            warmup: 5,
            background: base::color_from_urgba(255, 255, 255, 1.0),
            ui_font,
            semibold_font,
            window_size: Size::new(500.0, 500.0),
        },
    )
    .unwrap();

    app.start(|_| None);
}
