pub mod implementations;
pub mod canvas;
pub mod ecs;

use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};

use skia_safe::gpu::backend_render_targets::make_gl;
use skia_safe::gpu::surfaces::wrap_backend_render_target;
use skia_safe::gpu::{direct_contexts, BackendRenderTarget, Budgeted, DirectContext, Protected, SurfaceOrigin};
use skia_safe::gpu::gl::{Format, FramebufferInfo, Interface};
use skia_safe::{Canvas, Color, Color4f, ColorType, Image, Matrix, Paint, Point, Rect};
use winit::dpi::{LogicalPosition, PhysicalPosition, PhysicalSize};
use winit::error::EventLoopError;
use winit::event::{ElementState, Ime, KeyEvent, Modifiers, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersKeyState, PhysicalKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::raw_window_handle::{self, HasRawWindowHandle, HasWindowHandle};
use winit::window::{Window, WindowAttributes, WindowId};

use std::ffi::CString;
use std::num::NonZeroU32;
use std::rc::Rc;

use crate::ecs::{Bounds, Entity, GpuState, Interactable, Quad, Resources, Transform, World, render_quads};

#[derive(PartialEq, Eq, Clone)]
enum InteractableState {
    DEFAULT,
    HOVERED,
    PRESSED,
}

impl InteractableState {
    fn color(&self) -> Color4f {
        match self {
            InteractableState::DEFAULT => Color4f::new(0.5, 0.5, 0.5, 1.0),
            InteractableState::HOVERED => Color4f::new(0.6, 0.6, 0.6, 1.0),
            InteractableState::PRESSED => Color4f::new(0.3, 0.3, 0.3, 1.0),
        }
    }
}

struct App {
    world: World,
    resources: Resources,
}

fn create_canvas_skia_surface(gr_context: &mut DirectContext, rect: Rect) -> skia_safe::Surface {
    let size = rect.size().to_floor();
    let image_info = skia_safe::ImageInfo::new((size.width, size.height), ColorType::N32, skia_safe::AlphaType::Premul, None);
    skia_safe::gpu::surfaces::render_target(gr_context, Budgeted::Yes, &image_info, None, SurfaceOrigin::TopLeft, None, None, false).unwrap()
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.resources.gpu_state.is_none() {
            let attrs = WindowAttributes::default().with_title("gamer");
            match event_loop.create_window(attrs) {
                Ok(window) => {
                },
                Err(e) => eprintln!("Failed to create window: {:?}", e),
            }
        }
    }
    
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut gpu_state) = self.resources.gpu_state else { return; };
        if window_id != gpu_state.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let width = NonZeroU32::new(size.width).unwrap_or(NonZeroU32::new(1).unwrap());
                let height = NonZeroU32::new(size.height).unwrap_or(NonZeroU32::new(1).unwrap());

                gpu_state.gl_surface.resize(&gpu_state.gl_context, width, height);

                gpu_state.create_skia_surface(size);
                gpu_state.window.request_redraw();
            }
            WindowEvent::CursorMoved { device_id, position } => {
                let x = position.x as f32;
                let y = position.y as f32;
                let should_update = hover_system(&mut self.world, x, y);
                if should_update { gpu_state.window.request_redraw(); }
            }
            WindowEvent::MouseInput { device_id, state, button } => {
            }
            WindowEvent::RedrawRequested => {
                if gpu_state.skia_surface.is_none() {
                    gpu_state.create_skia_surface(gpu_state.window.inner_size());
                }
                if let Some(surface) = &mut gpu_state.skia_surface {
                    let canvas = surface.canvas();
                    render_system(&self.world, &canvas);
                    gpu_state.gr_context.flush_and_submit();
                    gpu_state.gl_surface.swap_buffers(&gpu_state.gl_context).unwrap();
                }
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
            }
            WindowEvent::ModifiersChanged(modifiers) => {
            }
            _ => {}
        }
    }
    
    // Handle window destruction for cleanup (though not strictly necessary 
    // for this simple example as the fields are Option)
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.resources.gpu_state = None;
    }
}

fn render_system(world: &World, canvas: &Canvas) {
    canvas.clear(Color::from_rgb(200, 200, 200));
    render_quads(world, canvas);
}

fn hover_system(world: &mut World, x: f32, y: f32) -> bool {
    let hovered = hover_detect(world, x, y);
    hover_update(world, &hovered);
    hovered.len() > 0
}

fn hover_detect(world: &World, x: f32, y: f32) -> Vec<Entity> {
    let mut results = Vec::new();
    world.query2::<Bounds, Interactable, _>(|entity, bounds, _| {
            let rect = bounds.rect;

            let hovered =
                x >= rect.left() && x <= rect.right() &&
                y >= rect.top()  && y <= rect.bottom();

            if hovered { results.push(entity) }
    });
    results
}

fn hover_update(world: &mut World, hovered: &[Entity]) {
    let mut interactable_storage = world.storage_mut::<Interactable>().unwrap();
    for entity in hovered {
        println!("{}", entity.0);
        let interactable = Some(interactable_storage.data.get_mut(entity).unwrap());
        interactable.unwrap().state = InteractableState::HOVERED;
    }
    let mut quad_storage = world.storage_mut::<Quad>().unwrap();
    for entity in hovered {
        let quad = quad_storage.data.get_mut(entity).unwrap();
        quad.color = InteractableState::HOVERED.color();
    }
}

fn main() -> Result<(), EventLoopError> {
    let initial_button_rect = Rect::from_xywh(30.0, 30.0, 30.0, 30.0);
    let event_loop = EventLoop::new().unwrap();
    
    let initial_attrs = WindowAttributes::default()
        .with_title("gamer")
        .with_inner_size(PhysicalSize::new(400, 400));

    let config_template_builder = ConfigTemplateBuilder::new();
    let display_builder = DisplayBuilder::new().with_window_attributes(Some(initial_attrs));

    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |configs| {
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
            .unwrap()
            })
    .unwrap();

    let window = Rc::new(window.unwrap());
    let raw_window_handle = window.window_handle().unwrap().as_raw();

    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
    let not_current_gl_context = unsafe {
        gl_display.create_context(&gl_config, &context_attributes).expect("couldnt make gl display create context")
    };
    let inner_size = window.inner_size();
    let width = NonZeroU32::new(inner_size.width).unwrap();
    let height = NonZeroU32::new(inner_size.height).unwrap();
    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(raw_window_handle, width, height);
    let gl_surface = unsafe {
        gl_display.create_window_surface(&gl_config, &surface_attributes).expect("could not make surface")
    };
    let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(&symbol).cast()
    });

    let gl_interface = Interface::new_load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(&symbol).cast()
    }).expect("Skia failed to load required GL functions!"); 

    let mut gr_context = direct_contexts::make_gl(gl_interface, None).expect("failed to create gr_context");

    let canvas_rect = Rect::from_wh(800.0, 800.0);
    let mut canvas_skia_surface = create_canvas_skia_surface(&mut gr_context, canvas_rect);
    let mut canvas_paint = Paint::new(Color4f::new(0.0, 0.0, 0.0, 1.0), None);
    canvas_paint.set_stroke_cap(skia_safe::PaintCap::Round);

    let gpu_state = GpuState { 
        gl_context,
        gl_config,
        gl_surface,
        gr_context,
        skia_surface: None,
        window: window.clone(),
    };

    let mut world = World::new();
    let button_entity = world.spawn();
    world.insert(button_entity, Bounds { rect: initial_button_rect });
    world.insert(button_entity, Quad { color: InteractableState::DEFAULT.color(), rect: initial_button_rect } );
    world.insert(button_entity, Interactable { state: InteractableState::DEFAULT } );
    world.insert(button_entity, Transform { local_to_parent: Matrix::new_identity(), z: 0.0 } );
    println!("{}", button_entity.0);

    let resources = Resources::new(gpu_state);

    let mut app = App {
        world,
        resources,
    };

    let mut canvas_history = Vec::new();
    canvas_history.push(canvas_skia_surface.image_snapshot());

    window.set_visible(true);
    window.request_redraw();

    let _ = event_loop.run_app(&mut app);

    Ok(())
}
