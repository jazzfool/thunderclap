use reui::{
    app, base,
    prelude::*,
    reclutch::display::Color,
    themes::Primer,
    ui::{self, Button, HStack, Label, VStack},
};

#[macro_use]
extern crate reclutch;
#[macro_use]
extern crate reui;

rooftop! {
    struct Counter: () {
        fn build(
            count: i32 = 0,
            btn_color: Color = theme.data().scheme.control_outset,
        ) {
           VStack(top_margin=5.0) {
                Label(
                    text=bind(format!("Count: {}", bind.count).into()),
                    wrap=false,
                ),
                HStack(left_margin=5.0) {
                    Button(
                        text=ui::txt("Count Up"),
                        background=bind(bind.btn_color)
                    )
                        @press {
                            widget.data.count += 1;
                        },
                    Button(
                        text=ui::txt("Count Down"),
                        background=bind(bind.btn_color)
                    )
                        @press {
                            widget.data.count -= 1;
                        },
                },
                Label(
                    text=bind(format!("Count: {}", bind.count).into()),
                    wrap=false,
                ),
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
