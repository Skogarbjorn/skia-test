use std::{any::{Any, TypeId, type_name}, cell::{Ref, RefCell, RefMut}, collections::HashMap, rc::Rc};

use glutin::{config::Config, context::PossiblyCurrentContext, surface::{WindowSurface}};
use skia_safe::{Canvas, Color4f, Image, Matrix, Paint, Rect, Surface, Vector, gpu::DirectContext};
use winit::{dpi::PhysicalPosition, event::Modifiers, window::Window};

use crate::InteractableState;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Entity(pub u32);

pub struct World {
    pub entities: Vec<Entity>,
    pub storages: HashMap<TypeId, RefCell<Box<dyn Any>>>,
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

pub struct Parallax {
    pub strength: f32,
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub local_to_parent: Matrix,
    pub z: f32,
}

pub struct View<'a, T> {
    storage: Ref<'a, Storage<T>>,
}
pub struct ViewMut<'a, T> {
    storage: RefMut<'a, Storage<T>>,
}
impl<'a, T> View<'a, T> {
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.storage.data.get(&entity)
    }
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.storage.data.iter().map(|(e, c)| (*e, c))
    }
}
impl<'a, T> ViewMut<'a, T> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.storage.data.get_mut(&entity)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.storage.data.iter_mut().map(|(e, c)| (*e, c))
    }
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

        let cell = self.storages.entry(type_id).or_insert_with(|| {
            RefCell::new(Box::new(Storage::<T> {
                data: HashMap::new(),
            }))
        });

        let mut storage_any = cell.borrow_mut();
        let storage = storage_any.downcast_mut::<Storage<T>>().unwrap();
        storage.data.insert(entity, component);
    }

    pub fn storage<T: 'static>(&self) -> Option<Ref<Storage<T>>> {
        let cell = self.storages.get(&TypeId::of::<T>())?;

        Some(Ref::map(cell.borrow(), |boxed| {
            boxed.downcast_ref::<Storage<T>>().unwrap()
        }))
    }

    pub fn storage_mut<T: 'static>(&self) -> Option<RefMut<Storage<T>>> {
        let cell = self.storages.get(&TypeId::of::<T>())?;

        Some(RefMut::map(cell.borrow_mut(), |boxed| {
            boxed.downcast_mut::<Storage<T>>().unwrap()
        }))
    }

    pub fn view<T: 'static>(&self) -> View<T> {
        println!("{}", type_name::<T>());
        View {
            storage: self.storage::<T>().expect("Storage not initialized")
        }
    }
    pub fn view_mut<T: 'static>(&self) -> ViewMut<T> {
        ViewMut { 
            storage: self.storage_mut::<T>().expect("Storage not initialized (mut version)")
        }
    }

    pub fn query<T: 'static, F>(&self, mut f: F) 
    where 
        F: FnMut(Entity, &T) 
    {
        if let Some(store) = self.storage::<T>() {
            for (entity, component) in store.data.iter() {
                f(*entity, component);
            }
        }
    }

    pub fn query2<A: 'static, B: 'static, F>(&self, mut f: F)
    where
        F: FnMut(Entity, &A, &B)
    {
        let a_store = self.storage::<A>();
        let b_store = self.storage::<B>();

        if let (Some(a), Some(b)) = (a_store, b_store) {
            for (entity, a_comp) in a.data.iter() {
                if let Some(b_comp) = b.data.get(entity) {
                    f(*entity, a_comp, b_comp);
                }
            }
        }
    }

    pub fn query3<A: 'static, B: 'static, C: 'static, F>(&self, mut f: F)
    where
        F: FnMut(Entity, &A, &B, &C)
    {
        let a_store = self.storage::<A>();
        let b_store = self.storage::<B>();
        let c_store = self.storage::<C>();

        if let (Some(a), Some(b), Some(c)) = (a_store, b_store, c_store) {
            for (entity, a_comp) in a.data.iter() {
                if let Some(b_comp) = b.data.get(entity) {
                    if let Some(c_comp) = c.data.get(entity) {
                        f(*entity, a_comp, b_comp, c_comp);
                    }
                }
            }
        }
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
    let mut q_view = world.view_mut::<Quad>();
    let t_view = world.view::<Transform>();

    for (entity, quad) in q_view.iter_mut() {
        canvas.save();
        if let Some(transform) = t_view.storage.data.get(&entity) {
            canvas.concat(&transform.local_to_parent);
        }
        let paint = Paint::new(quad.color, None);
        canvas.draw_rect(quad.rect, &paint);
        canvas.restore();
    }
}
