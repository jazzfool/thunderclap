# Widget Rundown

This is a rundown of all the widgets, giving a brief overview of each widget.

**Note:** This is *not* an alternative to the API documentation.

## Component Widgets

### Button - `reui::ui::Button`

*A button which can be pressed and focused by the user. Suitable for simple user actions.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **`Layable....`** ✔️
- **Properties:**
    - `text`: Text shown in the button.
    - `typeface`: Typeface used in for the text.
    - `color`: Color of the text.
    - `background`: Background color of the text.
    - `focus`: Color used to indicate focus (usually in the form of a border).
    - `contrast`: Contrast mode of `background` and `color`.
    - `disabled`: Whether the button can be interacted with.
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
- **Properties:**
    - `text`: Text shown by the label.
    - `typeface`: Typeface of the text.
    - `color`: Color of the text.
    - `align`: Horizontal alignment of the text.
    - `wrap`: Whether text should be wrapped to fit in the rectangle.
- **Outgoing Event Queues:**
    - *None*

### Checkbox - `reui::ui::Checkbox`

*Toggled checkbox. Suitable for boolean inputs.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **`Layable....`** ✔️
- **Properties:**
    - `foreground`: Color of the checkmark.
    - `background`: Color of the checkbox.
    - `focus`: Color used to indicate focus (usually in the form of a border).
    - `checked`: Whether the checkbox is checked.
    - `disabled`: Whether the checkbox can be interacted with.
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

### Horizontal Stack - `reui::ui::HStack`

*Layout widget which arranges widget horizontally.*

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
