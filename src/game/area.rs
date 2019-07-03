use amethyst::{
    assets::{AssetStorage, Loader, Prefab, PrefabData, PrefabLoader, ProgressCounter, RonFormat},
    derive::PrefabData,
    ecs::prelude::{
        Component, DenseVecStorage, Entity, Read, ReadExpect, ReadStorage, VecStorage, Write,
        WriteExpect, WriteStorage,
    },
    Error,
};

use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use super::{
    character::CharacterPrefab,
    consts::{TILE_HEIGHT, TILE_WIDTH},
};

pub struct CurrentArea(pub Entity);

#[derive(Clone, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Area {
    pub dimensions: [u32; 2],
}

impl Component for Area {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

/// Translate from area grid position to world pixel coordinates for rendering entities
pub fn get_world_coordinates(x: u32, y: u32) -> (f32, f32) {
    ((x * TILE_WIDTH) as f32, (y * TILE_HEIGHT) as f32)
}
