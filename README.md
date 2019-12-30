<p align="center">
    <img src=".media/reui.png" width="150px"/>
</p>

<h2 align="center">Rust UI Toolkit</h2>
<h3 align="center">Designed for anything from a quick and dirty GUI to a completely custom application.</h3>

<img align="right" src=".media/showcase.png" width="200px"/>

## Features
- Completely event-driven.
- Flutter/SwiftUI layout syntax without the need for diffing.
- 100% themable. 

## "Rooftop" Syntax

### Click [here](https://github.com/jazzfool/reui/wiki/Making-a-counter) to see how the code below works.

```rust
use reui::{
    app, base,
    reclutch::display::Color,
    themes::Primer,
    ui::{Button, Label, VStack},
};

rooftop! {
    struct Counter: () // <-- output event
    {
        fn build(
            count = (0): i32,
        ) {
            VStack(): v_stack {
                Label(
                    text=bind(format!("Count: {}", bind.count).into()), // <-- runtime bindings
                    wrap=false, // <-- properties
                ): count_label, // <-- variable name
                Button(text="Count Up"): count_up
                    @press { // <-- event handling
                        widget.data.count += 1;
                    },
                Button(text="Count Down"): count_down
                    @press {
                        widget.data.count -= 1;
                    },
            }
        }
    }
}

fn main() {
    let app = app::create(
        |_, display| Primer::new(display).unwrap(), // <-- create our theme
        |u_aux, g_aux, theme| {
            Counter {
                count: 5, // <-- we want to start counting from 5 instead of 0
                ..Counter::from_theme(theme)
            }.construct(theme, u_aux, g_aux) // <-- create the root widget
        },
        app::AppOptions {
            name: "Counter App",
            ..Default::default()
        },
    ).unwrap();
    app.start(|_| None);
}
```

### Lack of control

Although the Flutter/SwiftUI inspired syntax of the `rooftop!` macro is very useful for cobbling together something quickly, some applications need finer control. Thankfully, all the code that the `rooftop!` macro generates is regular Rust code. The code uses pipelines to simplify event queue handling (`reui::pipe::Pipeline`), all of which is equivalent to manually iterating over and matching events. You can peel away any of these abstractions and build something that still works with everything else.

---

### You can see a rundown of all the widgets [here](Widgets.md).

## Theme List (so far)

- GitHub Primer

## Widget List (so far)

- Button
- Vertical Stack
- Container
- Label
- Checkbox
- Horizontal Stack
- Text area

## License

Reui is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](http://opensource.org/licenses/MIT)

at your choosing.
