use amethyst::{
    assets::{PrefabData, ProgressCounter},
    core::Named,
    derive::PrefabData,
    ecs::prelude::{Component, DenseVecStorage, Entity, NullStorage, VecStorage, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
pub struct PlayerCharacter;

impl Component for PlayerCharacter {
    type Storage = NullStorage<Self>;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
#[serde(deny_unknown_fields)]
pub struct Glyph(pub char);

impl Component for Glyph {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Debug, Deserialize, Serialize, PrefabData)]
#[serde(deny_unknown_fields)]
pub enum CharacterVariant {
    PlayerCharacter(PlayerCharacter),
    NonPlayerCharacter {
        name: Named,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PrefabData)]
#[serde(deny_unknown_fields)]
pub struct CharacterPrefab {
    glyph: Glyph,
    variant: CharacterVariant,
}
