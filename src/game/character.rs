use amethyst::{
    assets::PrefabData,
    core::Named,
    derive::PrefabData,
    ecs::prelude::{Component, DenseVecStorage, Entity, NullStorage, ReadExpect, WriteStorage},
    ui::{Anchor, FontHandle, UiText, UiTransform},
    Error,
};

use serde::{Deserialize, Serialize};

use super::{
    area::Position,
    assets::Fonts,
    consts::{GLYPH_FONT_SIZE, NPC_SPRITE_LAYER, PLAYER_SPRITE_LAYER, TILE_HEIGHT, TILE_WIDTH},
};

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize, PrefabData)]
#[prefab(Component)]
/// Tag for the player character entity.
pub struct PlayerCharacter;

impl Component for PlayerCharacter {
    type Storage = NullStorage<Self>;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// A glyph that represents a character on screen in classic rogue like fashion.
pub struct Glyph(pub char);

impl Component for Glyph {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Prefab structure for deriving a character. See the implementation of `PrefabData`
/// below for more information.
pub struct CharacterPrefab {
    glyph: char,
    position: Option<Position>,
    variant: CharacterVariant,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
enum CharacterVariant {
    PlayerCharacter,
    NonPlayerCharacter { name: String },
}

/// Derive and add all required Components from the prefab when loading from a `PrefabLoader`.
///
/// For all characters:
///  * `Glyph`
///  * `Position`       (defaults to (0, 0) if not specified)
///  * `UiText`         for rendering the character as the given glyph
///  * `UiTransform`    (coordinates are not set, that's up to the rendering system
///
/// For `PlayerCharacter` variant:
///  * `PlayerCharacter`
///
/// For `NonPlayerCharacter` variant:
///  * `Named` with the given name
///
/// # Notes
///  * Requires the `Fonts` resource to exist.
///  * The glyph's `UiTransform` places the entity in screen-absolute coordinates
///    from the lower left corner.
///    ** This is not valid if a parent entity also has a `UiTransform`! **
impl<'a> PrefabData<'a> for CharacterPrefab {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Glyph>,
        WriteStorage<'a, PlayerCharacter>,
        WriteStorage<'a, Named>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, UiTransform>,
        ReadExpect<'a, Fonts>,
    );

    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        data: &mut Self::SystemData,
        _entities: &[Entity],
        _children: &[Entity],
    ) -> Result<Self::Result, Error> {
        eprintln!("CharacterPrefab: creating entity {:?}", &entity);

        let (positions, glyphs, player_characters, names, ui_texts, ui_transforms, fonts) = data;

        let position = self.position.clone().unwrap_or(Position { x: 0, y: 0 });
        positions.insert(entity, position)?;

        glyphs.insert(entity, Glyph(self.glyph))?;

        match self.variant {
            CharacterVariant::PlayerCharacter => {
                player_characters.insert(entity, PlayerCharacter)?;
            }
            CharacterVariant::NonPlayerCharacter { ref name } => {
                names.insert(entity, Named::new(name.clone()))?;
            }
        }

        let zlayer = match self.variant {
            CharacterVariant::PlayerCharacter => PLAYER_SPRITE_LAYER,
            CharacterVariant::NonPlayerCharacter { .. } => NPC_SPRITE_LAYER,
        };

        ui_texts.insert(entity, get_base_ui_text(self.glyph, fonts.main.clone()))?;
        ui_transforms.insert(entity, get_base_ui_transform(zlayer))?;

        Ok(())
    }
}

fn get_base_ui_text(glyph: char, font: FontHandle) -> UiText {
    UiText::new(
        font,
        glyph.to_string(),
        [1.0, 1.0, 1.0, 1.0],
        GLYPH_FONT_SIZE,
    )
}

fn get_base_ui_transform(zlayer: f32) -> UiTransform {
    UiTransform::new(
        "character".to_string(),
        Anchor::BottomLeft, // Relative to the lower left corner  of the screen
        Anchor::Middle,
        0.0,
        0.0,
        zlayer,
        TILE_WIDTH as f32,
        TILE_HEIGHT as f32,
    )
}
