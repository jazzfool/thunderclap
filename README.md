# Thunderclap

A Rust toolkit to write decomposable and fast user interfaces. It is:

- **Event-driven:** Thunderclap builds efficient abstractions over the Reclutch event system to avoid unnecessary computations.
- **Simple:** Thunderclap provides a suite of widgets alongside various infrastructures to simplify writing your own widgets.
- **Customizable:** There isn't a single line of hard-coded widget rendering; trivial widgets are fully parameterized and non-trivial widgets delegate to a provided theme.

<img align="right" src=".media/showcase.png" width="200px"/>

## Overview

Thunderclap aims to take the traditional widget hierarchy model from bulletproof libraries (e.g. Qt) and combine it with the cutting-edge simplicity of modern toolkits (e.g. Flutter).
To accomplish this it provides three primary high-level components:
- A widget library that fill the need for boilerplate UI components.
- A theme API with a verbose typography and color scheme protocol.
- A macro to emulate a declarative UI syntax for widget creation.

## Example

There's also [an in-depth overview of the code below](https://github.com/jazzfool/thunderclap/wiki/Making-a-counter).

```rust
use thunderclap::{
    app, base,
    themes::Primer,
    ui::{Button, Label, VStack},
};

rooftop! {
    // The empty tuple can be replaced with an event so that this widget can emit events.
    struct Counter: () {
        fn build(
            // Declare our state with a default value
            count: i32 = 0,
        ) {
            // Display the widgets in a vertical list
            VStack() {
                Label(
                    // Bind the text in order to keep it up-to-date.
                    text=bind(format!("Count: {}", bind.count).into()),
                    // We don't want to wrap the text.
                    wrap=false,
                ),
                Button(text="Count Up")
                    @press {
                        // Increment the count when this button is pressed.
                        widget.data.count += 1;
                    },
                Button(text="Count Down")
                    @press {
                        // Then decrement when this button is pressed.
                        widget.data.count -= 1;
                    },
            }
        }
    }
}

fn main() {
    let app = app::create(
        // This closure creates a theme to use
        |_, display| Primer::new(display).unwrap(),
        |u_aux, g_aux, theme| {
            Counter {
                // Perhaps we want to start counting from 5 instead of 0
                count: 5,
                ..Counter::from_theme(theme)
            }.construct(theme, u_aux, g_aux)
        },
        app::AppOptions {
            name: "Counter App".into(),
            ..Default::default()
        },
    ).unwrap();
    app.start(|_| None);
}
```

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

Thunderclap is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](http://opensource.org/licenses/MIT)

at your choosing.
