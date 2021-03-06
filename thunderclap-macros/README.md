# `derive`

All of these derives accept a `thunderclap_crate` attribute to specify the name of the Thunderclap crate;

```rust
use thunderclap as alternative_thunderclap;

#[derive(SomeThunderclapDerive)]
#[thunderclap_crate(alternative_thunderclap)] // <--
struct Foo // ...
```

Realistically, there's no need to use this. This mainly used within Thunderclap so that internal types can `derive` where the only handle to the crate root is `crate::`.

## `PipelineEvent`

```rust
#[derive(PipelineEvent, Clone, Copy, PartialEq)]
enum MyEvent {
    #[event_key(stop)]
    Stop,
    #[event_key(play)]
    Play(f32),
    #[event_key(rewind)]
    Rewind {
        seconds: u32,
        play: bool,
    },
}
```

Which resolves down to:
```rust
impl thunderclap::pipe::Event for MyEvent {
    fn get_key(&self) -> &'static str {
        match self {
            MyEvent::Stop => "stop",
            MyEvent::Play(..) => "play",
            MyEvent::Rewind{..} => "rewind",
        }
    }
}

impl MyEvent { // These are automatically called by `pipeline!` to "cast" the event.
    pub fn unwrap_as_stop(self) -> Option<()> {
        if let MyEvent::Stop = self { Some(()) } else { None }
    }

    pub fn unwrap_as_play(self) -> Option<(f32)> {
        if let MyEvent::Play(x0) = self { Some(x0) } else { None }
    }

    pub fn unwrap_as_rewind(self) -> Option<(u32, bool)> {
        if let MyEvent::Rewind{seconds, play} = self { Some((seconds, play)) } else { None }
    }
}
```

## `LayableWidget`

```rust
#[derive(LayableWidget)]
struct MyWidget {
    #[widget_layout]
    layout: WidgetLayoutEvents,
}
```

Expands to...

```rust
impl thunderclap::base::LayableWidget for MyWidget {
    #[inline]
    fn listen_to_layout(&mut self, layout: impl Into<Option<thunderclap::base::WidgetLayoutEventsInner>>) {
        self.layout.update(layout);
    }

    #[inline]
    fn layout_id(&self) -> Option<u64> {
        self.layout.id()
    }
}
```

## `DropNotifier`

```rust
#[derive(DropNotifier)]
struct MyWidget {
    #[widget_drop_event]
    drop_event: RcEventQueue<DropEvent>,
}
```

Expands to...

```rust
impl thunderclap::base::DropNotifier for MyWidget {
    #[inline(always)]
    fn drop_event(&self) -> &thunderclap::reclutch::event::RcEventQueue<thunderclap::base::DropEvent> {
        &self.drop_event
    }
}
```

Note that you'll still have to appropriately implement `Drop` to emit into `drop_event`;

```rust
// Manually implemented
impl Drop for MyWidget {
    fn drop(&mut self) {
        self.drop_event.emit_owned(DropEvent);
    }
}
```

## `HasVisibility`

```rust
#[derive(HasVisibility)]
struct MyWidget {
    #[widget_visibility]
    visibility: Visibility,
}
```

Expands to...

```rust
impl thunderclap::base::HasVisibility {
    #[inline]
    fn set_visibility(&mut self, visibility: thunderclap::base::Visibility) {
        self.visibility = visibility;
    }

    #[inline]
    fn visibility(&self) -> thunderclap::base::Visibility {
        self.visibility
    }
}
```

TL;DR: setter and getter.

## `Repaintable`

```rust
#[derive(Repaintable)]
struct MyWidget {
    #[repaint_target]
    a: CommandGroup,

    #[repaint_target]
    b: CommandGroup,

    #[widget_child]
    #[repaint_target]
    c: AnotherWidget, // <-- assuming this has a method called `repaint`.
}
```

Expands to...

```rust
impl thunderclap::base::Repaintable for MyWidget {
    #[inline]
    fn repaint(&mut self) {
        self.a.repaint();
        self.b.repaint();
        self.c.repaint();

        for child in thunderclap::base::WidgetChildren::children_mut(self) {
            child.repaint();
        }
    }
}
```

## `Movable` and `Resizable`

Both these derives accept an attribute `widget_transform_callback`.

In the case of deriving both `Movable` and `Resizable`, note that "overlapping" derive attributes are valid, so in many scenarios you can write the attribute once for it to be applied to both derives.

Assume `<a/b>` means "interchangeable", since these two derives are almost identical.

```rust
#[derive(<Movable/Resizable>)]
#[widget_transform_callback(on_transform)]
struct MyWidget {
    #[widget_rect]
    rect: RelativeRect
    // -- OR --
    #[widget_<position/size>]
    x: <RelativePoint/Size>,
}
```

Expands to...

```rust
impl thunderclap::base::<Movable/Resizable> for MyWidget {
    fn set_<position/size>(&mut self, <position/size>: thunderclap::reclutch::display::<RelativePoint/Size>) {
        self.rect.<origin/size> = <position/size>;
        // -- OR --
        self.x = <position/size>;

        thunderclap::base::Repaintable::repaint(self);
        self.on_transform();
    }

    #[inline]
    fn <position/size>(&self) -> thunderclap::reclutch::display::<RelativePoint/Size> {
        self.rect.<origin/size>
        // -- OR --
        self.x
    }
}
```

Here the `// -- OR --` denotes that the derive can operate on either a point/size field or a rectangle field.
