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

## License

Reui is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](http://opensource.org/licenses/MIT)

at your choosing.
