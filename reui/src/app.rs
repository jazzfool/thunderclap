use {
    crate::{base, draw, error::AppError},
    glutin::{
        event::{self, Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder, PossiblyCurrent, WindowedContext,
    },
    reclutch::{
        display::{
            skia, Color, CommandGroup, DisplayCommand, FontInfo, GraphicsDisplay, Point,
            ResourceData, ResourceDescriptor, ResourceReference, SharedData, Size, Vector,
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
    TF: FnOnce(&mut dyn GraphicsDisplay) -> T,
    RF: FnOnce(&mut UAux, &mut GAux, &T) -> R,
{
    let event_loop = EventLoop::new();

    let hidpi_factor = event_loop.primary_monitor().hidpi_factor();

    let wb = WindowBuilder::new().with_title(opts.name).with_inner_size(
        glutin::dpi::PhysicalSize::new(opts.window_size.width as _, opts.window_size.width as _)
            .to_logical(hidpi_factor),
    );

    let context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    let mut display =
        skia::SkiaGraphicsDisplay::new_gl_framebuffer(&skia::SkiaOpenGlFramebuffer {
            framebuffer_id: 0,
            size: (opts.window_size.width as _, opts.window_size.height as _),
        })?;

    let mut u_aux = UAux {
        window_queue: RcEventQueue::new(),
        cursor: Default::default(),
    };

    let mut g_aux = GAux {
        ui_font: {
            let resource = display.new_resource(ResourceDescriptor::Font(ResourceData::Data(
                SharedData::RefCount(std::sync::Arc::new(opts.ui_font.data().unwrap())),
            )))?;

            (resource, opts.ui_font)
        },
        semibold_font: {
            let resource = display.new_resource(ResourceDescriptor::Font(ResourceData::Data(
                SharedData::RefCount(std::sync::Arc::new(opts.semibold_font.data().unwrap())),
            )))?;

            (resource, opts.semibold_font)
        },
        scale: hidpi_factor as _,
    };

    let theme = theme(&mut display);
    let root = root(&mut u_aux, &mut g_aux, &theme);

    let mut app = App {
        root,
        background: opts.background,
        u_aux,
        g_aux,
        display,
        context,
        size: opts.window_size,
        event_loop,

        command_group_pre: CommandGroup::new(),
        command_group_post: CommandGroup::new(),
    };

    for _ in 0..opts.warmup {
        app.root.update(&mut app.u_aux);
        app.root.draw(&mut app.display, &mut app.g_aux);
    }

    Ok(app)
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
    /// The regular UI font.
    pub ui_font: FontInfo,
    /// The semibold UI font.
    pub semibold_font: FontInfo,
    /// Initial size of the app window.
    pub window_size: Size,
}

/// Reui/Reclutch based application.
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
    /// Graphical auxiliary.
    pub g_aux: GAux,
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
            mut g_aux,
            mut display,
            context,
            mut size,
            event_loop,

            mut command_group_pre,
            mut command_group_post,
        } = self;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    if display.size().0 != size.width as _ || display.size().1 != size.height as _ {
                        display.resize((size.width as _, size.height as _)).unwrap();
                    }

                    command_group_pre.push(
                        &mut display,
                        &[
                            DisplayCommand::Save,
                            DisplayCommand::Clear(background),
                            DisplayCommand::Scale(Vector::new(g_aux.scale, g_aux.scale)),
                        ],
                        false,
                        None,
                    );

                    root.draw(&mut display, &mut g_aux);

                    command_group_post.push(&mut display, &[DisplayCommand::Restore], false, None);

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
                    g_aux.scale = hidpi_factor as _;
                    let window_size = context.window().inner_size().to_physical(hidpi_factor);
                    size = Size::new(window_size.width as _, window_size.height as _);

                    command_group_pre.repaint();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(window_size),
                    ..
                } => {
                    let window_size = window_size.to_physical(g_aux.scale as _);
                    size = Size::new(window_size.width as _, window_size.height as _);
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let position = Point::new(position.x as _, position.y as _);

                    u_aux.cursor = position;

                    u_aux.window_queue.emit_owned(base::WindowEvent::MouseMove(
                        base::ConsumableEvent::new(position),
                    ));
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

                    u_aux.window_queue.emit_owned(base::WindowEvent::ClearFocus);

                    u_aux.window_queue.emit_owned(match state {
                        event::ElementState::Pressed => base::WindowEvent::MousePress(
                            base::ConsumableEvent::new((u_aux.cursor, mouse_button)),
                        ),
                        event::ElementState::Released => base::WindowEvent::MouseRelease(
                            base::ConsumableEvent::new((u_aux.cursor, mouse_button)),
                        ),
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(false),
                    ..
                } => {
                    u_aux.window_queue.emit_owned(base::WindowEvent::ClearFocus);
                }
                _ => return,
            }

            if let Some(cf) = f(event) {
                *control_flow = cf;
            }

            root.update(&mut u_aux);
            context.window().request_redraw();
        })
    }
}

/// Rudimentary update auxiliary.
pub struct UAux {
    pub window_queue: RcEventQueue<base::WindowEvent>,
    pub cursor: Point,
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
}

/// Rudimentary graphical auxiliary.
pub struct GAux {
    pub ui_font: (ResourceReference, FontInfo),
    pub semibold_font: (ResourceReference, FontInfo),
    pub scale: f32,
}

impl base::GraphicalAuxiliary for GAux {
    #[inline]
    fn ui_font(&self) -> (ResourceReference, FontInfo) {
        self.ui_font.clone()
    }

    #[inline]
    fn semibold_ui_font(&self) -> (ResourceReference, FontInfo) {
        self.semibold_font.clone()
    }

    #[inline]
    fn scaling(&self) -> f32 {
        self.scale
    }
}
