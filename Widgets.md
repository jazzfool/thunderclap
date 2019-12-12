# Widget Rundown

This is a rundown of all the widgets, giving a brief overview of each widget.

**Note:** This is *not* an alternative to the API documentation.

## Control Widgets

### Button - `reui::ui::Button`

*A button which can be pressed and focused by the user. Suitable for simple user actions.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **Outgoing Event Queues:**
    - `on_press`: The button was pressed.
    - `on_release`: The button was released. Complements `on_press`.
    - `on_mouse_enter`: The cursor began overlapping the button.
    - `on_mouse_leave`: The cursor stopped overlapping the button. Complements `on_mouse_enter`.
    - `on_focus`: The button gained focus.
    - `on_blur`: The button lost focus. Complements `on_focus`.

## Abstract Widgets

### Vertical Stack - `reui::ui::VStack`

*Layout widget which arranges widgets vertically.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **Outgoing Event Queues:**
    - *None*
