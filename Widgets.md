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
        - `press`: The button has been pressed.
        - `release`: The button has been released.
        - `begin_hover`: The cursor has entered the button boundaries.
        - `end_hover`: The cursor has left the button boundaries.
        - `focus`: The button has gained focus.
        - `blur`: The button has lost focus.

### Label - `reui::ui::Label`

*Aligned text wrapped in a rectangle.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - *None*

### Checkbox - `reui::ui::Checkbox`

*Toggled checkbox. Suitable for boolean inputs.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - `event_queue`: `CheckboxEvent`
        - `press`: The checkbox has been pressed.
        - `release`: The checkbox has been released.
        - `check`: The checkbox has been checked.
        - `uncheck`: The checkbox has been unchecked.
        - `begin_hover`: The cursor has entered the checkbox boundaries.
        - `end_hover`: The cursor has left the checkbox boundaries.
        - `focus`: The checkbox has gained focus.
        - `blur`: The checkbox has lost focus.

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
