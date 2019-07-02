use amethyst::{
    assets::{PrefabLoaderSystem, Processor},
    core::SystemBundle,
    error::Error,
    renderer::{sprite_visibility::SpriteVisibilitySortingSystem, SpriteSheet},
    shred::DispatcherBuilder,
};

use crate::{
    systems::{
        CameraMovementSystem, PlayerMovementSystem, UpdateCharTileTransformsSystem,
        UpdateTransformsSystem,
    },
};

use super::character::PlayerCharacterPrefab;

pub struct MovementSystemsBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MovementSystemsBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(PlayerMovementSystem, "player_movement_system", &[]);
        builder.add(
            CameraMovementSystem { reader: None },
            "camera_movement_system",
            &["player_movement_system"],
        );
        builder.add(
            UpdateCharTileTransformsSystem { reader: None },
            "update_char_tile_transforms_system",
            &["player_movement_system", "camera_movement_system"],
        );
        builder.add(
            UpdateTransformsSystem { reader: None },
            "update_sprite_transforms_system",
            &["player_movement_system"],
        );

        Ok(())
    }
}

pub struct PrefabLoaderBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PrefabLoaderBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            PrefabLoaderSystem::<PlayerCharacterPrefab>::default(), 
            "character_prefab_loader", 
            &[]
        );

        Ok(())
    }
}