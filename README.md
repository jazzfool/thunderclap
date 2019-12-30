<p align="center">
    <img src=".media/reui.png" width="150px"/>
</p>

<h2 align="center">Rust UI Toolkit</h2>
<h3 align="center">Designed for anything from a quick and dirty GUI to a completely custom application.</h3>

<img align="right" src=".media/showcase.png" width="200px"/>

## Writing a counter

First we need to import some things.
```rust
use reui::{
    app, base,
    reclutch::display::Color,
    themes::Primer,
    ui::{Button, Label, VStack},
};
```

After that we need to create the widget; we use the `rooftop!` macro for this;
```rust
rooftop! {
    struct Counter {
        fn build() {}
    }
}
```

This may seem a little unusual; a struct method being declared outside `impl`, but you'll see why.
Just note that `fn build() {}` never declares a method.

Now we need to add our state; the counting number itself.

```rust
rooftop! {
    struct Counter {
        fn build(
            count = (0): i32,
        ) {}
    }
}
```

However, what good is this state if it never changes? To make it change, you need a UI.
If you've ever used Flutter or SwiftUI then this should come quite naturally.

```rust
rooftop! {
    struct Counter {
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
```

Although this emulates the syntax of Flutter and SwiftUI, it does things very differently. This `build` pseudo-method is only called once in the widget's lifetime (when the widget is created). Because of this, a `bind` syntax is introduced.

These widgets are declared as fields in the underlying code, hence the need to annotate the widget variable name. This has a distinct advantage of being able to reference widgets within event handlers, for example `widget.label.set_size(..)`.

We've made our counter, but we still need to make it show up on the screen.

Building with the `app` and `default-themes` features, this is quite simple:
```rust
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

That's it. In under 50 lines you can go from creating a layout, declaring a state, handling events, and showing it all in a window.

Keep in mind that `Counter` is now an actual widget. Somewhere else in your code it is completely valid to now do something along the lines of:
```rust
rooftop! {
    // ...
        fn build() {
            VStack() {
                Counter(
                    count: 0, // <-- optional but valid
                ),
            }
        }
    // ...
}
```

### Lack of control

Although the Fluuter/SwiftUI inspired syntax of the `rooftop!` macro is very useful for cobbling together something quickly, some applications need finer control. Thankfully, all the code that the `rooftop!` macro generates is regular Rust code. The code uses pipelines to simplify event queue handling (`reui::pipe::Pipeline`), all of which is equivalent to manually iterating over and matching events. You can peel away any of these abstractions and build something that still works with everything else.

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
