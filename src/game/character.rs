use amethyst::{
    assets::{PrefabData, ProgressCounter},
    core::Named,
    derive::PrefabData,
    ecs::prelude::{Component, DenseVecStorage, Entity, NullStorage, WriteStorage},
    Error,
};

use serde::{Deserialize, Serialize};

use super::area::Position;

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
pub struct CharacterPrefab {
    glyph: Glyph,
    position: Option<Position>,
    // color: Option<[f32; 4]>,
    variant: CharacterVariant,
}

// #[derive(Clone, Debug, Deserialize, Serialize, PrefabData)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum CharacterVariant {
    PlayerCharacter,
    NonPlayerCharacter { name: String },
}

impl<'a> PrefabData<'a> for CharacterVariant {
    type SystemData = (WriteStorage<'a, PlayerCharacter>, WriteStorage<'a, Named>);
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (player_characters, names): &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<Self::Result, Error> {
        match &self {
            CharacterVariant::PlayerCharacter => {
                player_characters
                    .insert(entity, PlayerCharacter)
                    .expect("could not add `PlayerCharacter` component to character");
            }
            CharacterVariant::NonPlayerCharacter { ref name } => {
                names
                    .insert(entity, Named::new(name.clone()))
                    .expect("could not add `Named` component to character");
            }
}

        Ok(())
    }
}
