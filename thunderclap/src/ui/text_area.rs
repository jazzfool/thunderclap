use {
    crate::{
        base::{self, Repaintable},
        draw::{self, state},
        geom::*,
        ui,
    },
    reclutch::{
        display::{Color, CommandGroup, DisplayCommand, GraphicsDisplay, Rect},
        event::RcEventQueue,
        prelude::*,
        verbgraph as vg,
    },
    std::marker::PhantomData,
};

#[derive(Event, Debug, Clone, PartialEq)]
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

pub fn text_area_handler<T, U>() -> vg::UnboundQueueHandler<T, U, base::WindowEvent>
where
    T: LogicalTextArea + ui::InteractiveWidget,
    U: base::UpdateAuxiliary + 'static,
{
    vg::unbound_queue_handler! {
        T as obj,
        U as _aux,
        base::WindowEvent as event,

        text_input => {
            if let Some(&c) = event.with(|_| obj.interaction().contains(state::InteractionState::FOCUSED)) {
                if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                    obj.push_char(c);
                }
            }
        }

        key_press => {
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
    WidgetChildren,
    LayableWidget,
    DropNotifier,
    HasVisibility,
    Repaintable,
    Movable,
    Resizable,
    OperatesVerbGraph,
)]
#[widget_children_trait(base::WidgetChildren)]
#[thunderclap_crate(crate)]
#[widget_transform_callback(on_transform)]
pub struct TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary + 'static,
    G: base::GraphicalAuxiliary + 'static,
{
    pub event_queue: RcEventQueue<TextAreaEvent>,
    pub data: base::Observed<TextArea>,

    graph: vg::OptionVerbGraph<Self, U>,
    painter: Box<dyn draw::Painter<state::TextAreaState>>,
    interaction: state::InteractionState,
    parent_position: AbsolutePoint,

    #[widget_rect]
    rect: RelativeRect,
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

impl<U, G> ui::InteractiveWidget for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline(always)]
    fn interaction(&mut self) -> &mut state::InteractionState {
        &mut self.interaction
    }

    #[inline]
    fn mouse_bounds(&self) -> RelativeRect {
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

impl<U, G> LogicalTextArea for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
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
        if self.data.text.len() > 0 && self.data.cursor > 0 {
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
pub struct TextArea {
    pub text: String,
    pub placeholder: String,
    pub typeface: draw::TypefaceStyle,
    pub color: Color,
    pub placeholder_color: Color,
    pub cursor_color: Color,
    pub disabled: bool,
    pub cursor: usize,
}

impl<U, G> ui::WidgetDataTarget<U, G> for TextArea
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type Target = TextAreaWidget<U, G>;
}

impl<U, G> ui::WidgetConstructor<U, G> for TextArea
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn from_theme(theme: &dyn draw::Theme) -> Self {
        let data = theme.data();
        TextArea {
            text: "".into(),
            placeholder: "".into(),
            typeface: data.typography.body.clone(),
            color: data.scheme.over_control_inset,
            placeholder_color: draw::weaken(data.scheme.over_control_inset, 0.5, data.contrast),
            cursor_color: draw::weaken(data.scheme.over_control_inset, 0.1, data.contrast),
            disabled: false,
            cursor: 0,
        }
    }

    fn construct(
        self,
        theme: &dyn draw::Theme,
        u_aux: &mut U,
        _g_aux: &mut G,
    ) -> TextAreaWidget<U, G>
    where
        U: base::UpdateAuxiliary + 'static,
        G: base::GraphicalAuxiliary + 'static,
    {
        let data = base::Observed::new(self);

        let mut graph = vg::verbgraph! {
            TextAreaWidget<U, G> as obj,
            U as _aux,
            "bind" => _ev in &data.on_change => { change => { obj.repaint(); } }
        };

        graph = graph.add(
            "interaction",
            ui::basic_interaction_handler::<TextAreaWidget<U, G>, U>().bind(u_aux.window_queue()),
        );
        graph = graph.add(
            "text_area",
            text_area_handler::<TextAreaWidget<U, G>, U>().bind(u_aux.window_queue()),
        );

        let painter = theme.text_area();
        let rect = RelativeRect::new(
            Default::default(),
            painter
                .size_hint(state::TextAreaState {
                    rect: Default::default(),
                    data: data.clone(),
                    interaction: state::InteractionState::empty(),
                })
                .cast_unit(),
        );

        TextAreaWidget {
            event_queue: Default::default(),
            data,

            graph: graph.into(),
            painter: theme.text_area(),
            interaction: state::InteractionState::empty(),
            parent_position: Default::default(),

            rect,
            visibility: Default::default(),
            command_group: Default::default(),
            drop_event: Default::default(),
            layout: Default::default(),

            phantom_g: Default::default(),
        }
    }
}

impl<U, G> TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn on_transform(&mut self) {
        self.repaint();
        self.layout.notify(self.abs_rect());
    }

    fn derive_state(&self) -> state::TextAreaState {
        state::TextAreaState {
            rect: self.abs_rect(),
            data: self.data.clone(),
            interaction: self.interaction,
        }
    }
}

impl<U, G> vg::HasVerbGraph for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn verb_graph(&mut self) -> &mut vg::OptionVerbGraph<Self, U> {
        &mut self.graph
    }
}

impl<U, G> Widget for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    type UpdateAux = U;
    type GraphicalAux = G;
    type DisplayObject = DisplayCommand;

    fn bounds(&self) -> Rect {
        self.rect.cast_unit()
    }

    fn update(&mut self, aux: &mut U) {
        let mut graph = self.graph.take().unwrap();
        graph.update_all(self, aux);
        self.graph = Some(graph);

        if let Some(rect) = self.layout.receive() {
            self.set_ctxt_rect(rect);
            self.command_group.repaint();
        }
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, _aux: &mut G) {
        let state = self.derive_state();
        let painter = &mut self.painter;
        self.command_group.push_with(
            display,
            || painter.draw(state),
            Default::default(),
            None,
            None,
        );
    }
}

impl<U, G> StoresParentPosition for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn set_parent_position(&mut self, parent_pos: AbsolutePoint) {
        self.parent_position = parent_pos;
        self.on_transform();
    }

    fn parent_position(&self) -> AbsolutePoint {
        self.parent_position
    }
}

impl<U, G> draw::HasTheme for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.painter
    }

    fn resize_from_theme(&mut self) {}
}

impl<U, G> ui::DefaultWidgetData<TextArea> for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn default_data(&mut self) -> &mut base::Observed<TextArea> {
        &mut self.data
    }
}

impl<U, G> ui::DefaultEventQueue<TextAreaEvent> for TextAreaWidget<U, G>
where
    U: base::UpdateAuxiliary,
    G: base::GraphicalAuxiliary,
{
    #[inline]
    fn default_event_queue(&self) -> &RcEventQueue<TextAreaEvent> {
        &self.event_queue
    }
}

impl<U: base::UpdateAuxiliary, G: base::GraphicalAuxiliary> Drop for TextAreaWidget<U, G> {
    fn drop(&mut self) {
        self.drop_event.emit_owned(base::DropEvent);
    }
}
