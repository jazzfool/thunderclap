# Widget Rundown

This is a rundown of all the widgets, giving a brief overview of each widget.

**Note:** This is *not* an alternative to the API documentation.

## Component Widgets

### Button - `reui::ui::Button`

*A button which can be pressed and focused by the user. Suitable for simple user actions.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - `event_queue`: `ButtonEvent`
        - `ButtonEvent::Press`: The button has been pressed/released.
        - `ButtonEvent::MouseHover`: The cursor has entered/left the button boundaries.
        - `ButtonEvent::Focus`: The button has gained/lost focus.

### Label - `reui::ui::Label`

*Aligned text wrapped in a rectangle.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - *None*

## Abstract Widgets

### Vertical Stack - `reui::ui::VStack`

*Layout widget which arranges widgets vertically.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - *None*

### Container - `reui::ui::Container`

*Dynamically stores a list of widgets. This is useful if you don't need to access a child past initialization-time; essentially grouping it into a single child to minimize unused fields.*
*The children will still be rendered and receive updates.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ❌
- **Outgoing Event Queues:**
    - *None*
