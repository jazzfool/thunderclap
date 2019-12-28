use {
    crate::{
        base::{self, Repaintable},
        draw::{self, state, ColorSwatch},
        pipe, ui,
    },
    reclutch::{
        display::{CommandGroup, DisplayCommand, GraphicsDisplay, Rect},
        event::RcEventQueue,
        prelude::*,
    },
    std::marker::PhantomData,
};

#[derive(PipelineEvent, Debug, Clone, PartialEq)]
#[reui_crate(crate)]
pub enum TextAreaEvent {
    /// The text area gained focus.
    #[event_key(focus)]
    Focus,
    /// The text area lost focus.
    #[event_key(blur)]
    Blur,
    /// The user modified text within the text area.
    #[event_key(user_modify)]
    UserModify(String),
}

pub fn text_area_terminal<T, U>() -> pipe::UnboundTerminal<T, U, base::WindowEvent>
where
    T: LogicalTextArea + ui::InteractiveWidget,
    U: base::UpdateAuxiliary + 'static,
{
    unbound_terminal! {
        T as obj,
        U as _aux,
        base::WindowEvent as event,

        text_input {
            if let Some(&c) = event.with(|_| obj.interaction().contains(state::InteractionState::FOCUSED)) {
                if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                    obj.push_char(c);
                }
            }
        }

        key_press {
            if let Some((key, _)) = event.with(|_| obj.interaction().contains(state::InteractionState::FOCUSED)) {
                match key {
                    base::KeyInput::Back => {
                        obj.remove_char();
                    }
                    base::KeyInput::Left => {
                        obj.move_cursor(-1);
                    }
                    base::KeyInput::Right => {
                        obj.move_cursor(1);
                    }
                    _ => {}
                }
            }
        }
    }
}

pub trait LogicalTextArea {
    /// Returns a mutable reference to the output event queue.
    fn event_queue(&mut self) -> &mut RcEventQueue<TextAreaEvent>;
    /// Add a character to the text.
    fn push_char(&mut self, c: char);
    /// Remove a character from the text.
    fn remove_char(&mut self);
    /// Move text cursor by an offset.
    fn move_cursor(&mut self, offset: isize);
}

#[derive(
    WidgetChildren, LayableWidget, DropNotifier, HasVisibility, Repaintable, Movable, Resizable,
)]
#[widget_children_trait(base::WidgetChildren)]
#[reui_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<TextAreaEvent>,
    pub data: base::Observed<TextAreaData>,

    pipe: Option<pipe::Pipeline<Self, U>>,
    painter: Box<dyn draw::Painter<state::TextAreaState>>,
    interaction: state::InteractionState,

    #[widget_rect]
    rect: Rect,
    #[widget_visibility]
    visibility: base::Visibility,
    #[repaint_target]
    command_group: CommandGroup,
    #[widget_drop_event]
    drop_event: RcEventQueue<base::DropEvent>,
    #[widget_layout]
    layout: base::WidgetLayoutEvents,

    phantom_g: PhantomData<G>,
}

impl<U, G> ui::InteractiveWidget for TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline]
    fn mouse_bounds(&self) -> Rect {
        self.painter.mouse_hint(self.rect)
    }

    #[inline]
    fn disabled(&self) -> bool {
        self.data.disabled
    }

    fn on_interaction_event(&mut self, event: ui::InteractionEvent) {
        match event {
            ui::InteractionEvent::Focus => {
                self.repaint();
                self.event_queue.emit_owned(TextAreaEvent::Focus);
            }
            ui::InteractionEvent::Blur => {
                self.repaint();
                self.event_queue.emit_owned(TextAreaEvent::Blur);
            }
            _ => {}
        }
    }
}

impl<U, G> LogicalTextArea for TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    #[inline(always)]
    fn event_queue(&mut self) -> &mut RcEventQueue<TextAreaEvent> {
        &mut self.event_queue
    }

    #[inline]
    fn push_char(&mut self, c: char) {
        {
            let cursor = self.data.cursor;
            self.data.text.insert(cursor, c);
        }
        self.repaint();
        self.data.cursor += 1;
    }

    #[inline]
    fn remove_char(&mut self) {
        self.repaint();
        if self.data.text.len() > 0 {
            {
                let cursor = self.data.cursor;
                self.data.text.remove(cursor - 1);
            }
            self.data.cursor -= 1;
        }
    }

    #[inline]
    fn move_cursor(&mut self, offset: isize) {
        self.repaint();
        let cursor = self.data.cursor as isize + offset;
        if cursor >= 0 && cursor <= self.data.text.len() as isize {
            self.data.cursor = cursor as _;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextAreaData {
    pub text: String,
    pub placeholder: String,
    pub typeface: draw::TypefaceStyle,
    pub color: ColorSwatch,
    pub placeholder_color: ColorSwatch,
    pub cursor_color: ColorSwatch,
    pub disabled: bool,
    pub cursor: usize,
}

impl TextAreaData {
    pub fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        TextAreaData {
            text: "".into(),
            placeholder: "".into(),
            typeface: data.typography.body.clone(),
            color: data.scheme.over_control_inset,
            placeholder_color: draw::ColorSwatch::generate(
                data.scheme.over_control_inset.weaken_500(data.contrast, 3),
                0.8,
            ),
            cursor_color: draw::ColorSwatch::generate(
                data.scheme.over_control_inset.strengthen_500(data.contrast, 3),
                0.8,
            ),
            disabled: false,
            cursor: 0,
        }
    }

    pub fn construct<U, G>(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        _g_aux: &mut G,
    ) -> TextArea<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        let mut pipe = pipeline! {
            TextArea<U, G> as obj,
            U as _aux,
            _ev in &data.on_change => { change { obj.repaint(); } }
        };

        pipe = pipe
            .add(ui::basic_interaction_terminal::<TextArea<U, G>, U>().bind(u_aux.window_queue()));
        pipe = pipe.add(text_area_terminal::<TextArea<U, G>, U>().bind(u_aux.window_queue()));

        let painter = theme.text_area();
        let rect = Rect::new(
            Default::default(),
            painter.size_hint(state::TextAreaState {
                rect: Default::default(),
                data: data.clone(),
                interaction: state::InteractionState::empty(),
            }),
        );

        TextArea {
            event_queue: Default::default(),
            data,

            pipe: pipe.into(),
            painter: theme.text_area(),
            interaction: state::InteractionState::empty(),

            rect,
            visibility: Default::default(),
            command_group: Default::default(),
            drop_event: Default::default(),
            layout: Default::default(),

            phantom_g: Default::default(),
        }
    }
}

impl<U, G> TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.rect);
    }

    fn derive_state(&self) -> state::TextAreaState {
        state::TextAreaState {
            rect: self.rect,
            data: self.data.clone(),
            interaction: self.interaction,
        }
    }
}

impl<U, G> Widget for TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect
    }

    fn update(&mut self, aux: &mut U) {
        let mut pipe = self.pipe.take().unwrap();
        pipe.update(self, aux);
        self.pipe = Some(pipe);

        if let Some(rect) = self.layout.receive() {
            self.rect = rect;
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(display, || painter.draw(state), None, None);
    }
}

impl<U, G> draw::HasTheme for TextArea<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {}
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Drop for TextArea<U, G> {
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
