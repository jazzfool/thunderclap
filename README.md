<p align="center">
    <img src=".media/reui.png" width="150px"/>
</p>

## <p align="center">Themable GUI toolkit</p>

<img align="right" src=".media/showcase.png" width="200px"/>

## Using Reui
The only notable difference between Reclutch on how you should use widgets is
that Reui defines it's own version of `WidgetChildren`, this is so that the widget
tree is enforced to be theme-able (implements `HasTheme`).
You can still `derive` this `WidgetChildren` like so:
```rust
#[derive(WidgetChildren)]
#[widget_children_trait(reui::base::WidgetChildren)]
struct MyWidget;
```

### Layout
Layout is simple and idiomatic with the provided macros:
```rust
let v_stack_data = VStackData::default().align(Align::Middle);

define_layout! {
    for v_stack => {
        v_stack_data => &mut button,
        v_stack_data => define_layout! {
            for another_v_stack => {
                v_stack_data => &mut nested_button
            }
        }
    }
}
```

### Events
Reui introduces an additional (optional) event handling layer to achieve more traditional callbacks.

The managing type here is `reui::update::Pipeline`. Typically it is one pipeline to one widget.
Each pipeline has a list of "terminals" `reui::update::Terminal`. One terminal goes to one event queue.
Then, each terminal has a list of handlers (callbacks) which use a static string as a key.
This key is used to cherry-pick events from an event queue.

All event data types must implement `reui::update::Event`. This trait has a method which enables event
cherry-picking.

Example of a pipeline for a simple counter:
```rust
let pipe = pipeline! {
    Counter as obj,
    UpdateAux as _aux,
    _event in &count_up.event_queue => { // first terminal
        press { obj.count += 1; } // handler
    }
    _event in &count_down.event_queue => { // second terminal
        press { obj.count -= 1; } // handler
    }
};
```
The above code would be found where `Counter` is being created.

Then when it comes time to update we simply defer to the pipeline.
```rust
impl Widget for Counter {
    // ...
    fn update(&mut self, aux: &mut UpdateAux) {
        let mut pipe = self.pipe.take().unwrap(); // we must move the pipeline out first or else the borrow checker will complain.
        pipe.update(self, aux);
        self.pipe = Some(pipe); // we move it back in when we're done.
    }
}
```

That's it! This is a much nicer solution than manually matching events, and best of all we can now separate the logic into a single variable (pipelines), or even modularize parts of it (terminals)! Further, this is a *layer* on event queues, which means you can still get more granular control over the queue/listeners if you want to.

The `pipeline!` macro roughly (it doesn't chain methods directly) translates to the following:
```rust
Pipeline::new()
    .add(Terminal::new(&count_up.event_queue).on(
        "press",
        |obj: &mut Counter, _aux: &mut UpdateAux, _event| {
            obj.count += 1;
        },
    ))
    .add(Terminal::new(&count_down.event_queue).on(
        "press",
        |obj: &mut Counter, _aux: &mut UpdateAux, _event| {
            obj.count -= 1;
        },
    ));
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

## License

Reui is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](http://opensource.org/licenses/MIT)

at your choosing.
