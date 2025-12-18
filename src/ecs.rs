use std::{any::{Any, TypeId}, collections::HashMap, rc::Rc};

use glutin::{config::Config, context::PossiblyCurrentContext, surface::{WindowSurface}};
use skia_safe::{Canvas, Color4f, Image, Paint, Rect, Surface, Vector, gpu::DirectContext};
use winit::{dpi::PhysicalPosition, event::Modifiers, window::Window};

use crate::InteractableState;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Entity(pub u32);

pub struct World {
    pub entities: Vec<Entity>,
    pub storages: HashMap<TypeId, Box<dyn Any>>,
}

/*
pub struct World {
    entities: Vec<u32>,
    bounds: HashMap<Entity, Bounds>,
    quads: HashMap<Entity, Quad>,
    history: HashMap<Entity, History>,
    drawing_state: HashMap<Entity, DrawingState>,
    paint: HashMap<Entity, Paint>,
    canvas_surface: HashMap<Entity, CanvasSurface>,
}
*/

pub struct Resources {
    pub gpu_state: Option<GpuState>,
    pub keyboard_state: KeyboardState,
    pub mouse_state: MouseState,
}

pub struct GpuState {
    pub gl_context: PossiblyCurrentContext,
    pub gl_config: Config,
    pub gl_surface: glutin::surface::Surface<WindowSurface>,
    pub gr_context: DirectContext,
    pub skia_surface: Option<Surface>,
    pub window: Rc<Window>,
}

pub struct KeyboardState {
    modifiers: Modifiers,
}

pub struct MouseState {
    prev_cursor_pos: PhysicalPosition<f32>,
}

pub struct Bounds {
    pub rect: Rect,
}

pub struct Quad {
    pub rect: Rect,
    pub color: Color4f,
}

struct CanvasSurface {
    surface: Surface,
}

struct History {
    history: Vec<Image>,
    history_index: usize,
    max_history: usize,
}

struct DrawingState {
    is_drawing: bool,
}

pub struct Storage<T> {
    pub data: HashMap<Entity, T>,
}

pub struct DirtyVisual;

pub struct Interactable {
    pub state: InteractableState,
}

pub struct Parallax;

pub struct Transform {
    translation: Vector,
    scale: Vector,
    z: f32,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: vec!(),
            storages: HashMap::new(),
        }
    }

    pub fn spawn(self: &mut Self) -> Entity {
        let next = self.entities.last().unwrap_or(&Entity(0)).0 + 1;
        let new_entity = Entity(next);
        self.entities.push(new_entity);
        new_entity
    }

    pub fn insert<T: 'static>(self: &mut Self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();

        let storage = self.storages
            .entry(type_id)
            .or_insert_with(|| {
                Box::new(Storage::<T> {
                    data: HashMap::new(),
                })
            });

        let storage = storage.downcast_mut::<Storage<T>>().unwrap();

        storage.data.insert(entity, component);
    }

    pub fn storage<T: 'static>(&self) -> Option<&Storage<T>> {
        self.storages
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<Storage<T>>())
    }

    pub fn storage_mut<T: 'static>(&mut self) -> Option<&mut Storage<T>> {
        self.storages
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<Storage<T>>())
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let store = self.storage::<T>();

        store.into_iter()
            .flat_map(move |t| {
                t.data.iter().filter_map(|(entity, component)| {
                    Some((*entity, component))
                })
            })
    }

    pub fn query2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        let a_store = self.storage::<A>();
        let b_store = self.storage::<B>();

        a_store
            .into_iter()
            .flat_map(move |a| {
                b_store.into_iter().flat_map(move |b| {
                    a.data.iter().filter_map(|(entity, a_component)| {
                        let b_component = b.data.get(entity)?;
                        Some((*entity, a_component, b_component))
                    })
                })
            })
    }
    pub fn query3<A: 'static, B: 'static, C: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B, &C)> {
        let a_store = self.storage::<A>();
        let b_store = self.storage::<B>();
        let c_store = self.storage::<C>();

        a_store.into_iter().flat_map(move |a| {
            let b = b_store.unwrap();
            let c = c_store.unwrap();

            a.data.iter().filter_map(|(entity, a_component)| {
                let b_component = b.data.get(entity)?;
                let c_component = c.data.get(entity)?;
                Some((*entity, a_component, b_component, c_component))
            })
        })
    }
}

impl Resources {
    pub fn new(gpu_state: GpuState) -> Self {
        Resources { 
            gpu_state: Some(gpu_state),
            keyboard_state: KeyboardState { modifiers: Modifiers::default() },
            mouse_state: MouseState { prev_cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 } } 
        }
    }
}

pub fn render_quads(world: &World, canvas: &Canvas) {
    for (_, quad) in world.query::<Quad>() {
        let paint = Paint::new(
            quad.color,
            None
        );
        canvas.draw_rect(quad.rect, &paint);
    }
}
