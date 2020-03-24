# Widget Rundown

This is a rundown of all the widgets, giving a brief overview of each widget.

**Note:** This is _not_ an alternative to the API documentation.

## Component Widgets

### Button - `thunderclap::ui::Button`

_A button which can be pressed and focused by the user. Suitable for simple user actions._

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

### Label - `thunderclap::ui::Label`

_Aligned text wrapped in a rectangle._

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
  - _None_

### Checkbox - `thunderclap::ui::Checkbox`

_Toggled checkbox. Suitable for boolean inputs._

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

### Text Area - `thunderclap::ui::TextArea`

_Accepts single line text input. Deliberately a visually bare-bones widget so that text input can be placed outside a textbox context. Suitable for string input._

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

### Scroll Bar - `thunderclap::ui::ScrollBar`

_Scrolling track with an interactive bar. Suitable for overflowing content._

- **`Themed.....`** ✔️
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Properties:**
  - `lock_width`: Whether the width/height parameter (depending on orientation) is taken into account.
  - `document_length`: Full length of the overflowing content.
  - `page_length`: Visible length of overflowing content.
  - `background`: Color of the scrolling track.
  - `foreground`: Color of the interactive bar.
  - `contrast`: Contrast mode of `background` and `foreground`.
- **Outgoing Event Queues:**
  - `event_queue`: `TextAreaEvent`
    - `begin_scroll`: The bar has been grabbed by the cursor.
    - `end_scroll`: The bar has been released by the cursor.
    - `scroll`: The bar has been moved, either by the cursor or scroll wheel.

## Abstract Widgets

### Vertical Stack - `thunderclap::ui::VStack`

_Layout widget which arranges widgets vertically._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Horizontal Stack - `thunderclap::ui::HStack`

_Layout widget which arranges widget horizontally._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Container Widget - `thunderclap::ui::Container`

_Dynamically stores a list of widgets. This is useful if you don't need to access a child past initialization-time; essentially grouping it into a single child to minimize unused fields._
_The children will still be rendered and receive updates._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ❌
- **Outgoing Event Queues:**
  - _None_

### Margins - `thunderclap::ui::Margins`

_Adds margins around the boundaries of it's children as a whole._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Max Fill - `thunderclap::ui::MaxFill`

_Computes the rectangle fitting all it's children, then resizes all it's children to said rectangle._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Relative Box - `thunderclap::ui::RelativeBox`

_Positions and sizes its only child relative to its parent._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Sized Box - `thunderclap::ui::SizedBox`

_Fixed size/position empty box._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_

### Option Widget - `thunderclap::ui::OptionWidget`

_Stores a child in an `Option`, allowing for dynamic swapping or removal of a child._

- **`Themed.....`** ❌
- **`Focusable..`** ❌
- **`Layable....`** ✔️
- **Outgoing Event Queues:**
  - _None_
