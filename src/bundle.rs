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

pub struct SpriteBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for SpriteBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            Processor::<SpriteSheet>::new(),
            "sprite_sheet_processor",
            &[],
        );
        builder.add(
            SpriteVisibilitySortingSystem::default(),
            "sprite_visibility_sorting_system",
            &["sprite_sheet_processor"],
        );

        Ok(())
    }
}