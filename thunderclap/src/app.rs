use {
    crate::{base, draw, error::AppError, geom::*},
    glutin::{
        event::{self, DeviceEvent, Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder, PossiblyCurrent, WindowedContext,
    },
    reclutch::{
        display::{
            self, skia, Color, CommandGroup, DisplayCommand, GraphicsDisplay, Point, Size, Vector,
        },
        event::RcEventQueue,
        prelude::*,
    },
};

/// Creates an application with a given theme and root widget.
/// The application uses the Skia OpenGL graphics backend.
/// Small details of app creation can be controlled with `AppOptions`.
pub fn create<R, T, TF, RF>(theme: TF, root: RF, opts: AppOptions) -> Result<App<R>, AppError>
where
    R: base::WidgetChildren<UpdateAux = UAux, GraphicalAux = GAux, DisplayObject = DisplayCommand>,
    T: draw::Theme,
    TF: FnOnce(&mut GAux, &mut dyn GraphicsDisplay) -> T,
    RF: FnOnce(&mut UAux, &T) -> R,
{
    let event_loop = EventLoop::new();

    let hidpi_factor = event_loop.primary_monitor().scale_factor();

    let wb = WindowBuilder::new().with_title(opts.name).with_inner_size(
        glutin::dpi::PhysicalSize::new(
            opts.window_size.width as f64,
            opts.window_size.width as f64,
        )
        .to_logical::<f64>(hidpi_factor),
    );

    let context = ContextBuilder::new().with_vsync(true).build_windowed(wb, &event_loop).unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let mut display =
        skia::SkiaGraphicsDisplay::new_gl_framebuffer(&skia::SkiaOpenGlFramebuffer {
            framebuffer_id: 0,
            size: (opts.window_size.width as _, opts.window_size.height as _),
        })?;

    let g_aux = GAux { scale: hidpi_factor as _ };
    let mut u_aux = UAux { window_queue: RcEventQueue::new(), cursor: Default::default(), g_aux };

    let theme = theme(&mut u_aux.g_aux, &mut display);
    let root = root(&mut u_aux, &theme);

    let mut app = App {
        root,
        background: opts.background,
        u_aux,
        display,
        context,
        size: opts.window_size,
        event_loop,

        command_group_pre: CommandGroup::new(),
        command_group_post: CommandGroup::new(),
    };

    for _ in 0..opts.warmup {
        app.root.update(&mut app.u_aux);
        app.root.draw(&mut app.display, &mut app.u_aux.g_aux);
    }

    Ok(app)
}

fn convert_modifiers(modifiers: event::ModifiersState) -> base::KeyModifiers {
    base::KeyModifiers {
        shift: modifiers.shift(),
        ctrl: modifiers.ctrl(),
        alt: modifiers.alt(),
        logo: modifiers.logo(),
    }
}

/// Settings on how an app should be created.
#[derive(Debug, Clone)]
pub struct AppOptions {
    /// The name of the application; usually translates to the window title.
    pub name: String,
    /// The number warmup cycles (i.e. the amount of times `update` and `draw` should be called offscreen).
    pub warmup: u32,
    /// The background color of the window.
    pub background: Color,
    /// Initial size of the app window.
    pub window_size: Size,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            name: "Thunderclap App".into(),
            warmup: 2,
            background: Color::new(1.0, 1.0, 1.0, 1.0),
            window_size: Size::new(500.0, 500.0),
        }
    }
}

/// Thunderclap/Reclutch based application.
pub struct App<R>
where
    R: base::WidgetChildren<UpdateAux = UAux, GraphicalAux = GAux, DisplayObject = DisplayCommand>,
{
    /// Root widget.
    pub root: R,
    /// Background color.
    pub background: Color,
    /// Update auxiliary.
    pub u_aux: UAux,
    /// Graphics display (Skia backend).
    pub display: skia::SkiaGraphicsDisplay,
    /// OpenGL context/window.
    pub context: WindowedContext<PossiblyCurrent>,
    size: Size,
    event_loop: EventLoop<()>,

    command_group_pre: CommandGroup,
    command_group_post: CommandGroup,
}

impl<R> App<R>
where
    R: base::WidgetChildren<UpdateAux = UAux, GraphicalAux = GAux, DisplayObject = DisplayCommand>,
{
    /// Starts the event loop.
    pub fn start<F>(self, mut f: F) -> !
    where
        F: 'static + FnMut(Event<()>) -> Option<ControlFlow>,
        R: 'static,
    {
        let App {
            mut root,
            background,
            mut u_aux,
            mut display,
            context,
            mut size,
            event_loop,

            mut command_group_pre,
            mut command_group_post,
        } = self;

        let mut modifiers =
            base::KeyModifiers { shift: false, ctrl: false, alt: false, logo: false };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::MainEventsCleared => context.window().request_redraw(),
                Event::RedrawRequested(..) => {
                    if display.size().0 != size.width as _ || display.size().1 != size.height as _ {
                        display.resize((size.width as _, size.height as _)).unwrap();
                    }

                    command_group_pre.push(
                        &mut display,
                        &[
                            DisplayCommand::Save,
                            DisplayCommand::Clear(background),
                            DisplayCommand::Scale(Vector::new(
                                u_aux.g_aux.scale,
                                u_aux.g_aux.scale,
                            )),
                        ],
                        display::ZOrder(std::i32::MIN),
                        false,
                        None,
                    );

                    base::invoke_draw(&mut root, &mut display, &mut u_aux.g_aux);

                    command_group_post.push(
                        &mut display,
                        &[DisplayCommand::Restore],
                        display::ZOrder(std::i32::MAX),
                        false,
                        None,
                    );

                    display.present(None).unwrap();

                    context.swap_buffers().unwrap();
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::ScaleFactorChanged { scale_factor: hidpi_factor, .. },
                    ..
                } => {
                    u_aux.g_aux.scale = hidpi_factor as _;
                    let window_size = context.window().inner_size();
                    size = Size::new(window_size.width as _, window_size.height as _);

                    command_group_pre.repaint();
                }
                Event::WindowEvent { event: WindowEvent::Resized(window_size), .. } => {
                    size = Size::new(window_size.width as _, window_size.height as _);
                }
                Event::DeviceEvent {
                    event: DeviceEvent::ModifiersChanged(key_modifiers), ..
                } => {
                    modifiers = convert_modifiers(key_modifiers);
                }
                Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                    let position = position.to_logical::<f64>(u_aux.g_aux.scale as f64);
                    let position = Point::new(position.x as _, position.y as _);

                    u_aux.cursor = position.cast_unit();

                    u_aux.window_queue.emit_owned(base::WindowEvent::MouseMove(
                        base::ConsumableEvent::new((position.cast_unit(), modifiers)),
                    ));
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. }, ..
                } => {
                    let mouse_button = match button {
                        event::MouseButton::Left => base::MouseButton::Left,
                        event::MouseButton::Middle => base::MouseButton::Middle,
                        event::MouseButton::Right => base::MouseButton::Right,
                        _ => base::MouseButton::Left,
                    };

                    u_aux.window_queue.emit_owned(base::WindowEvent::ClearFocus);

                    u_aux.window_queue.emit_owned(match state {
                        event::ElementState::Pressed => base::WindowEvent::MousePress(
                            base::ConsumableEvent::new((u_aux.cursor, mouse_button, modifiers)),
                        ),
                        event::ElementState::Released => base::WindowEvent::MouseRelease(
                            base::ConsumableEvent::new((u_aux.cursor, mouse_button, modifiers)),
                        ),
                    });
                }
                Event::WindowEvent { event: WindowEvent::ReceivedCharacter(character), .. } => {
                    u_aux.window_queue.emit_owned(base::WindowEvent::TextInput(
                        base::ConsumableEvent::new(character),
                    ));
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input: event::KeyboardInput { virtual_keycode, state, .. },
                            ..
                        },
                    ..
                } => {
                    if let Some(virtual_keycode) = virtual_keycode {
                        let key_input: base::KeyInput = virtual_keycode.into();

                        u_aux.window_queue.emit_owned(match state {
                            event::ElementState::Pressed => base::WindowEvent::KeyPress(
                                base::ConsumableEvent::new((key_input, modifiers)),
                            ),
                            event::ElementState::Released => base::WindowEvent::KeyRelease(
                                base::ConsumableEvent::new((key_input, modifiers)),
                            ),
                        });
                    }
                }
                Event::WindowEvent { event: WindowEvent::Focused(false), .. } => {
                    u_aux.window_queue.emit_owned(base::WindowEvent::ClearFocus);
                }
                _ => return,
            }

            if let Some(cf) = f(event) {
                *control_flow = cf;
            }

            root.update(&mut u_aux);
        })
    }
}

/// Rudimentary update auxiliary.
pub struct UAux {
    pub window_queue: RcEventQueue<base::WindowEvent>,
    pub cursor: AbsolutePoint,
    pub g_aux: GAux,
}

impl base::UpdateAuxiliary for UAux {
    #[inline]
    fn window_queue(&self) -> &RcEventQueue<base::WindowEvent> {
        &self.window_queue
    }

    #[inline]
    fn window_queue_mut(&mut self) -> &mut RcEventQueue<base::WindowEvent> {
        &mut self.window_queue
    }

    #[inline]
    fn graphical(&self) -> &dyn base::GraphicalAuxiliary {
        &self.g_aux
    }

    #[inline]
    fn graphical_mut(&mut self) -> &mut dyn base::GraphicalAuxiliary {
        &mut self.g_aux
    }
}

/// Rudimentary graphical auxiliary.
pub struct GAux {
    pub scale: f32,
}

impl base::GraphicalAuxiliary for GAux {
    #[inline]
    fn scaling(&self) -> f32 {
        self.scale
    }
}
