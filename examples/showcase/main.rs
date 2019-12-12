use {
    glutin::{
        event::{self, Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
    },
    reui::{
        base, draw,
        prelude::*,
        reclutch::{
            display::{
                self, Color, CommandGroup, DisplayCommand, DisplayText, FontInfo, GraphicsDisplay,
                Point, Rect, ResourceData, ResourceDescriptor, ResourceReference, SharedData, Size,
                Vector,
            },
            event::RcEventQueue,
            prelude::*,
        },
        themes::Primer,
        ui,
    },
};

#[macro_use]
extern crate reclutch;

struct UpdateAux {
    window_queue: RcEventQueue<base::WindowEvent>,
    cursor: Point,
}

impl base::UpdateAuxiliary for UpdateAux {
    fn window_queue(&self) -> &RcEventQueue<base::WindowEvent> {
        &self.window_queue
    }

    fn window_queue_mut(&mut self) -> &mut RcEventQueue<base::WindowEvent> {
        &mut self.window_queue
    }
}

struct GraphicalAux {
    font: (ResourceReference, FontInfo),
    semibold_font: (ResourceReference, FontInfo),
    scale: Vector,
}

impl base::GraphicalAuxiliary for GraphicalAux {
    fn ui_font(&self) -> (ResourceReference, FontInfo) {
        self.font.clone()
    }

    fn semibold_ui_font(&self) -> (ResourceReference, FontInfo) {
        self.semibold_font.clone()
    }

    fn scaling(&self) -> f32 {
        self.scale.x
    }
}

#[derive(WidgetChildren)]
#[widget_children_trait(base::WidgetChildren)]
struct Showcase {
    #[widget_child]
    button_1:
        base::LayedOut<ui::Button<UpdateAux, GraphicalAux>, ui::VStack<UpdateAux, GraphicalAux>>,
    // er, this could be a little less ugly probably somehow maybe...
    #[widget_child]
    button_2:
        base::LayedOut<ui::Button<UpdateAux, GraphicalAux>, ui::VStack<UpdateAux, GraphicalAux>>,
    #[widget_child]
    button_3:
        base::LayedOut<ui::Button<UpdateAux, GraphicalAux>, ui::VStack<UpdateAux, GraphicalAux>>,
    #[widget_child]
    button_4:
        base::LayedOut<ui::Button<UpdateAux, GraphicalAux>, ui::VStack<UpdateAux, GraphicalAux>>,
    #[widget_child]
    v_stack: ui::VStack<UpdateAux, GraphicalAux>,

    command_group_pre: CommandGroup,
    command_group_post: CommandGroup,

    themed: draw::PhantomThemed,
}

impl Showcase {
    fn new(
        theme: &dyn draw::Theme,
        update_aux: &mut UpdateAux,
        gfx_aux: &mut GraphicalAux,
    ) -> Self {
        let mut v_stack =
            ui::VStack::new(Rect::new(Point::new(50.0, 50.0), Size::new(200.0, 200.0)));

        let v_stack_data = ui::VStackData {
            top_margin: 10.0,
            bottom_margin: 0.0,
            alignment: ui::VStackAlignment::Left,
        };

        let button_1 = v_stack.push(
            v_stack_data,
            ui::simple_button("Button 1".to_string(), theme, update_aux, gfx_aux),
        );
        let button_2 = v_stack.push(
            ui::VStackData {
                alignment: ui::VStackAlignment::Middle,
                ..v_stack_data
            },
            ui::simple_button("Button 2".to_string(), theme, update_aux, gfx_aux),
        );
        let button_3 = v_stack.push(
            ui::VStackData {
                alignment: ui::VStackAlignment::Right,
                ..v_stack_data
            },
            ui::simple_button("Button 3".to_string(), theme, update_aux, gfx_aux),
        );
        let button_4 = v_stack.push(
            ui::VStackData {
                alignment: ui::VStackAlignment::Stretch,
                ..v_stack_data
            },
            ui::simple_button("VStacks!".to_string(), theme, update_aux, gfx_aux),
        );

        Showcase {
            button_1,
            button_2,
            button_3,
            button_4,
            v_stack,

            command_group_pre: CommandGroup::new(),
            command_group_post: CommandGroup::new(),

            themed: draw::PhantomThemed,
        }
    }
}

impl Widget for Showcase {
    type UpdateAux = UpdateAux;
    type GraphicalAux = GraphicalAux;
    type DisplayObject = DisplayCommand;

    fn update(&mut self, aux: &mut UpdateAux) {
        base::invoke_update(self, aux);

        self.v_stack.update_layout(vec![
            self.button_1.activate(),
            self.button_2.activate(),
            self.button_3.activate(),
            self.button_4.activate(),
        ]);
    }

    fn draw(&mut self, display: &mut dyn GraphicsDisplay, aux: &mut GraphicalAux) {
        self.command_group_pre.push(
            display,
            &[
                DisplayCommand::Save,
                DisplayCommand::Scale(aux.scale),
                DisplayCommand::Clear(Color::new(1.0, 1.0, 1.0, 1.0)),
            ],
            false,
        );

        base::invoke_draw(self, display, aux);

        self.command_group_post
            .push(display, &[DisplayCommand::Restore], false);
    }
}

impl base::Repaintable for Showcase {
    fn repaint(&mut self) {
        self.command_group_pre.repaint();
        self.command_group_post.repaint();
    }
}

impl draw::HasTheme for Showcase {
    fn theme(&mut self) -> &mut dyn draw::Themed {
        &mut self.themed
    }

    fn resize_from_theme(&mut self, _aux: &dyn base::GraphicalAuxiliary) {}
}

fn main() {
    let mut window_size = (500u32, 500u32);

    let event_loop = EventLoop::new();
    let hidpi_factor = event_loop.primary_monitor().hidpi_factor();

    let wb = glutin::window::WindowBuilder::new()
        .with_title("Reui Showcase")
        .with_inner_size(
            glutin::dpi::PhysicalSize::new(window_size.0 as _, window_size.1 as _)
                .to_logical(hidpi_factor),
        );

    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let mut display = display::skia::SkiaGraphicsDisplay::new_gl_framebuffer(
        &display::skia::SkiaOpenGlFramebuffer {
            framebuffer_id: 0,
            size: (window_size.0 as _, window_size.1 as _),
        },
    )
    .unwrap();

    let mut update_aux = UpdateAux {
        window_queue: RcEventQueue::new(),
        cursor: Default::default(),
    };

    let mut gfx_aux = {
        let font_info = FontInfo::from_data(
            std::sync::Arc::new(include_bytes!("../Inter-Regular.ttf").to_vec()),
            0,
        )
        .unwrap();

        let font = display
            .new_resource(ResourceDescriptor::Font(ResourceData::Data(
                SharedData::RefCount(std::sync::Arc::new(font_info.data().unwrap())),
            )))
            .unwrap();

        let semibold_font_info = FontInfo::from_data(
            std::sync::Arc::new(include_bytes!("../Inter-SemiBold.ttf").to_vec()),
            0,
        )
        .unwrap();

        let semibold_font = display
            .new_resource(ResourceDescriptor::Font(ResourceData::Data(
                SharedData::RefCount(std::sync::Arc::new(semibold_font_info.data().unwrap())),
            )))
            .unwrap();

        GraphicalAux {
            font: (font, font_info),
            semibold_font: (semibold_font, semibold_font_info),
            scale: Vector::new(hidpi_factor as _, hidpi_factor as _),
        }
    };

    let theme = Primer;

    let mut showcase = Showcase::new(&theme, &mut update_aux, &mut gfx_aux);

    showcase.update(&mut update_aux);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if display.size().0 != window_size.0 as _ || display.size().1 != window_size.1 as _
                {
                    display
                        .resize((window_size.0 as _, window_size.1 as _))
                        .unwrap();
                }

                showcase.draw(&mut display, &mut gfx_aux);
                display.present(None).unwrap();
                context.swap_buffers().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::HiDpiFactorChanged(hidpi_factor),
                ..
            } => {
                gfx_aux.scale = Vector::new(hidpi_factor as _, hidpi_factor as _);
                let size = context.window().inner_size().to_physical(hidpi_factor);
                window_size = (size.width as _, size.height as _);
                showcase.repaint();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = size.to_physical(gfx_aux.scale.x as _);
                window_size = (size.width as _, size.height as _);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let position = Point::new(position.x as _, position.y as _);

                update_aux.cursor = position;

                update_aux
                    .window_queue
                    .emit_owned(base::WindowEvent::MouseMove(base::ConsumableEvent::new(
                        position,
                    )));
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                let mouse_button = match button {
                    event::MouseButton::Left => base::MouseButton::Left,
                    event::MouseButton::Middle => base::MouseButton::Middle,
                    event::MouseButton::Right => base::MouseButton::Right,
                    _ => base::MouseButton::Left,
                };

                update_aux
                    .window_queue
                    .emit_owned(base::WindowEvent::ClearFocus);

                update_aux.window_queue.emit_owned(match state {
                    event::ElementState::Pressed => base::WindowEvent::MousePress(
                        base::ConsumableEvent::new((update_aux.cursor, mouse_button)),
                    ),
                    event::ElementState::Released => base::WindowEvent::MouseRelease(
                        base::ConsumableEvent::new((update_aux.cursor, mouse_button)),
                    ),
                });
            }
            _ => return,
        }

        showcase.update(&mut update_aux);
        context.window().request_redraw();
    });
}
