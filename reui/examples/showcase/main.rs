use reui::{
    app, base,
    reclutch::display::Color,
    themes::Primer,
    ui::{Button, Label, VStack},
};

#[macro_use]
extern crate reclutch;
#[macro_use]
extern crate reui;

rooftop! {
    struct Counter {
        fn build(
            count = (0): i32,
            btn_color = (theme.data().scheme.control_outset): Color
        ) {
            VStack(top_margin=5.0): v_stack {
                Label(
                    text=bind(format!("Count: {}", bind.count).into()),
                    wrap=false,
                ): count_label,
                //HStack(): button_list {
                    Button(text="Count Up".to_string().into(), background=bind(bind.btn_color)): count_up
                        @press {
                            widget.data.count += 1;
                        },
                    Button(text="Count Down".to_string().into(), background=bind(bind.btn_color)): count_down
                        @press {
                            widget.data.count -= 1;
                        },
                //},
            }
        }
    }
}

fn main() {
    let app = app::create(
        |_g_aux, display| Primer::new(display).unwrap(),
        |u_aux, g_aux, theme| {
            Counter { ..Counter::from_theme(theme) }.construct(theme, u_aux, g_aux)
        },
        app::AppOptions { name: "Showcase".to_string(), ..Default::default() },
    )
    .unwrap();
    app.start(|_| None);
}
