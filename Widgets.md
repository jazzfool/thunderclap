# Widget Rundown

## Control Widgets

### Button - `reui::ui::Button`
- **`Themed.....`** ✔️
- **`Focusable..`** ✔️
- **Outgoing Event Queues:**
    - `on_press`: The button was pressed.
    - `on_release`: The button was released. Complements `on_press`.
    - `on_mouse_enter`: The cursor begin overlapping the button.
    - `on_mouse_leave`: The cursor stopped overlapping the button. Complements `on_mouse_enter`.
    - `on_focus`: The button gained focus.
    - `on_blur`: The button lost focus. Complements `on_focus`.