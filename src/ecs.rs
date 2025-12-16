use std::collections::HashMap;

use skia_safe::{Color4f, Image, Paint, Rect, Surface};

struct Entity(u32);

pub struct World {
    entities: Vec<u32>,
    bounds: HashMap<Entity, Bounds>,
    quads: HashMap<Entity, Quad>,
    history: HashMap<Entity, History>,
    drawing_state: HashMap<Entity, DrawingState>,
    paint: HashMap<Entity, Paint>,
    canvas_surface: HashMap<Entity, CanvasSurface>,
}

struct Bounds {
    rect: Rect,
}

struct Quad {
    color: Color4f,
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
