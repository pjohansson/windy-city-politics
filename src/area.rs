use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity, VecStorage};

pub const TILE_HEIGHT: u32 = 24;
pub const TILE_WIDTH: u32 = 16;

pub struct CurrentArea(pub Entity);

#[derive(Clone, Debug)]
pub struct Area {
    pub dimensions: [u32; 2],
}

impl Component for Area {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

/// Translate from area grid position to screen coordinates for entities
pub fn get_screen_coordinates(x: u32, y: u32) -> (f32, f32) {
    ((x * TILE_WIDTH) as f32, (y * TILE_HEIGHT) as f32)
}
