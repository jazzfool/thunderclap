# Widget Rundown

This is a rundown of all the widgets, giving a brief overview of each widget.

**Note:** This is *not* an alternative to the API documentation.

## Control Widgets

### Button - `reui::ui::Button`

*A button which can be pressed and focused by the user. Suitable for simple user actions.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **Outgoing Event Queues:**
    - `event_queue`: `ButtonEvent`
        - `ButtonEvent::Press`: The button has been pressed/released.
        - `ButtonEvent::MouseHover`: The cursor has entered/left the button boundaries.
        - `ButtonEvent::Focus`: The button has gained/lost focus.

## Abstract Widgets

### Vertical Stack - `reui::ui::VStack`

*Layout widget which arranges widgets vertically.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **Outgoing Event Queues:**
    - *None*
