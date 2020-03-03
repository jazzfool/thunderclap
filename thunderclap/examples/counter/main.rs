use thunderclap::{
    app, base,
    reclutch::display::Color,
    themes::Primer,
    ui::{self, Button, HStack, Label, Margins, ScrollBar, SideMargins, TextArea, VStack},
};

#[macro_use]
extern crate reclutch;
#[macro_use]
extern crate thunderclap;

rooftop! {
    struct Counter: () {
        fn build(
            count: i32 = 0,
            btn_color: Color = theme.data().scheme.control_outset,
        ) {
            Margins(margins=SideMargins::new_all_same(10.0)) {
                ScrollBar(),
                VStack(bottom_margin=5.0) {
                    Label(
                        text=bind(format!("Count: {}", bind.count).into()),
                        wrap=false,
                    ),
                    HStack(left_margin=5.0) {
                        Button(
                            text="Count Up".into(),
                            background=bind(bind.btn_color)
                        )
                            @press {
                                widget.data.count += 1;
                            },
                        Button(
                            text="Count Down".into(),
                            background=bind(bind.btn_color)
                        )
                            @press {
                                widget.data.count -= 1;
                            },
                    },
                    TextArea(
                        placeholder="placeholder text!".into(),
                    ),
                },
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
