<p align="left">
    <img src=".media/reui.png" width="150px"/>
</p>

## Themable GUI toolkit

---

<img style="float: right" src=".media/showcase.png" width="110px"/>

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

---

### You can see a rundown of all the widgets [here](Widgets.md).

## Theme List (so far)
- GitHub Primer

## Widget List (so far)
- Button
