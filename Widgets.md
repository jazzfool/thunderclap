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
    - `foreground`: Color of the check mark.
    - `background`: Color of the checkbox.
    - `focus`: Color used to indicate focus (usually in the form of a border).
    - `contrast`: Contrast mode of `background` and `foreground`.
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

### Text Area - `reui::ui::TextArea`

*Accepts single line text input. Deliberately a visually bare-bones widget so that text input can be placed outside a textbox context. Suitable for string input.*

- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **`Layable....`** ✔️
- **Properties:**
    - `text`: Text within the text area.
    - `placeholder`: Placeholder text to appear when text is empty.
    - `typeface`: Typeface used for text.
    - `color`: Color of the text.
    - `placeholder_color`: Color of the placeholder text.
    - `cursor_color`: Color of text cursor/caret.
    - `disabled`: Whether the text area can be interacted with.
    - `cursor`: Text cursor/caret position.
- **Outgoing Event Queues:**
    - `event_queue`: `TextAreaEvent`
        - `focus`: The text area has gained focus.
        - `blur`: The text area has lost focus.
        - `user_modify`: The text area has been modified by the user.

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

### Margins - `reui::ui::Margins`

*Adds margins around the boundaries of it's children as a whole.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - *None*

### Max Fill - `reui::ui::MaxFill`

*Computes the rectangle fitting all it's children, then resizes all it's children to said rectangle.*

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
    - *None*
