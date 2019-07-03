use amethyst::{
    core::Transform,
    ecs::prelude::{
        Join, Read, ReadExpect, ReadStorage, Resources, System, SystemData, WriteStorage,
    },
    renderer::ActiveCamera,
    shrev::{EventChannel, ReaderId},
    ui::UiTransform,
    window::ScreenDimensions,
};

use crate::{
    game::{
        consts::{TILE_HEIGHT, TILE_WIDTH},
        get_world_coordinates, Position,
    },
    render::get_screen_center_coordinates,
};

/// Event used to signal that entity transforms should be updated.
pub struct UpdateTransformsEvent;

/// Updates `UiTransforms` for world entities which are represented by a `char`.
///
/// In contrast to regular `Transforms` these have to be placed in screen-absolute
/// coordinates since UI elements are fixed on screen. We calculate this position
/// first relative to the camera, then use the fact that the camera is centered
/// in the screen to calculate the screen-absolute position.
///
/// NOTE: Upcoming problem if a UI is added to occupy a slice of the screen since
/// the camera will no longer be centered at the screen.
///
/// TODO: Replace this logic by using a custom render pass for character tiles
/// which uses `Transform`?
pub struct UpdateCharTileTransformsSystem {
    pub reader: Option<ReaderId<UpdateTransformsEvent>>,
}

impl<'s> System<'s> for UpdateCharTileTransformsSystem {
    type SystemData = (
        WriteStorage<'s, UiTransform>,
        Read<'s, ActiveCamera>,
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Position>,
        Read<'s, EventChannel<UpdateTransformsEvent>>,
    );

    fn run(&mut self, (mut transforms, camera, dimensions, positions, events): Self::SystemData) {
        for _ in events.read(self.reader.as_mut().unwrap()) {
            let screen_center = get_screen_center_coordinates(&dimensions);
            let camera_position = get_active_camera_position(&camera, &positions);

            for (transform, entity_position) in (&mut transforms, &positions).join() {
                let (x, y) = get_screen_absolute_coordinates_for_entity_grid_position(
                    screen_center,
                    &camera_position,
                    entity_position,
                );

                transform.local_x = x;
                transform.local_y = y;
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(
            res.fetch_mut::<EventChannel<UpdateTransformsEvent>>()
                .register_reader(),
        );
    }
}

/// Updates `Transforms` for world entities.
pub struct UpdateTransformsSystem {
    pub reader: Option<ReaderId<UpdateTransformsEvent>>,
}

impl<'s> System<'s> for UpdateTransformsSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Position>,
        Read<'s, EventChannel<UpdateTransformsEvent>>,
    );

    fn run(&mut self, (mut transforms, positions, events): Self::SystemData) {
        for _ in events.read(self.reader.as_mut().unwrap()) {
            for (transform, position) in (&mut transforms, &positions).join() {
                let (x, y) = get_world_coordinates(position.x, position.y);
                transform.set_translation_x(x);
                transform.set_translation_y(y);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(
            res.fetch_mut::<EventChannel<UpdateTransformsEvent>>()
                .register_reader(),
        );
    }
}

pub fn get_active_camera_position(
    camera: &ActiveCamera,
    positions: &ReadStorage<Position>,
) -> Position {
    camera
        .entity
        .and_then(|entity| positions.get(entity))
        .cloned()
        .unwrap_or(Position { x: 0, y: 0 })
}

pub fn get_screen_absolute_coordinates_for_entity_grid_position(
    screen_center: (f32, f32),
    camera: &Position,
    entity: &Position,
) -> (f32, f32) {
    let (x0, y0) = screen_center;

    let (xcamera, ycamera) = get_world_coordinates(camera.x, camera.y);
    let (xentity, yentity) = get_world_coordinates(entity.x, entity.y);

    let dx = xentity - xcamera;
    let dy = yentity - ycamera;

    (
        x0 + dx + (TILE_WIDTH / 2) as f32,
        y0 + dy + (TILE_HEIGHT / 2) as f32,
    )
}
