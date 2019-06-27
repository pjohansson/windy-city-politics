use amethyst::{
    core::Transform,
    ecs::prelude::{
        Join, Read, ReadExpect, ReadStorage, Resources, System, SystemData, WriteStorage,
    },
    renderer::Camera,
    shrev::{EventChannel, ReaderId},
};

use crate::area::{Area, CurrentArea, Position};

use super::{
    utils::{clamp_position, update_position, update_transform},
    Action, PlayerActionEvent,
};

// Camera position buffers to halt movement this many tiles before the current area edge.
const CAMERA_AREA_EDGE_BUFFER_WIDTH_X: u32 = 17;
const CAMERA_AREA_EDGE_BUFFER_WIDTH_Y: u32 = 7;

/// System to move the camera along with the player character.
pub struct CameraMovementSystem {
    pub reader: Option<ReaderId<PlayerActionEvent>>,
}

impl<'s> System<'s> for CameraMovementSystem {
    type SystemData = (
        WriteStorage<'s, Position>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        ReadExpect<'s, CurrentArea>,
        ReadStorage<'s, Area>,
        Read<'s, EventChannel<PlayerActionEvent>>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut transforms,
            cameras,
            current_area,
            areas,
            event_channel
        ): Self::SystemData,
    ) {
        for action in event_channel.read(self.reader.as_mut().unwrap()) {
            if let PlayerActionEvent(Action::Move(direction)) = action {
                let area_size = areas.get(current_area.0).unwrap().dimensions;
                let [min_x, min_y, max_x, max_y] = get_valid_camera_positions(
                    &area_size,
                    CAMERA_AREA_EDGE_BUFFER_WIDTH_X,
                    CAMERA_AREA_EDGE_BUFFER_WIDTH_Y,
                );

                for (position, transform, _) in (&mut positions, &mut transforms, &cameras).join() {
                    update_position(position, direction, &[min_x, min_y, max_x, max_y]);
                    update_transform(transform, position);
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(
            res.fetch_mut::<EventChannel<PlayerActionEvent>>()
                .register_reader(),
        );
    }
}

/// Get the closed area in which the camera can move on the current grid.
///
/// The allowed area will leave a border of input size to all edges in which the camera
/// will not enter.
fn get_valid_camera_positions(
    [size_x, size_y]: &[u32; 2],
    border_x: u32,
    border_y: u32,
) -> [u32; 4] {
    [
        clamp_position(border_x as i32, 0, size_x.saturating_sub(1) / 2),
        clamp_position(border_y as i32, 0, size_y.saturating_sub(1) / 2),
        clamp_position(
            *size_x as i32 - border_x as i32 - 1,
            size_x.saturating_sub(1) / 2,
            *size_x,
        ),
        clamp_position(
            *size_y as i32 - border_y as i32 - 1,
            size_y.saturating_sub(1) / 2,
            *size_y,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_camera_positions_without_border_is_full_area() {
        assert_eq!(&[0, 0, 0, 0], &get_valid_camera_positions(&[0, 0], 0, 0));
        assert_eq!(&[0, 0, 19, 9], &get_valid_camera_positions(&[20, 10], 0, 0));
    }

    #[test]
    fn valid_camera_positions_with_small_border_works() {
        assert_eq!(&[1, 1, 18, 8], &get_valid_camera_positions(&[20, 10], 1, 1));
        assert_eq!(&[2, 2, 17, 7], &get_valid_camera_positions(&[20, 10], 2, 2));
    }

    #[test]
    fn valid_camera_positions_with_large_borders_are_centered() {
        assert_eq!(
            &[9, 4, 10, 4],
            &get_valid_camera_positions(&[20, 10], 9, 9),
            "border larger than size along y but not x"
        );
        assert_eq!(
            &[9, 4, 9, 4],
            &get_valid_camera_positions(&[20, 10], 100, 100),
            "border larger than area"
        );
    }

    #[test]
    fn valid_camera_positions_adjusts_with_different_border_values_along_x_and_y() {
        // Zero sized along both axes
        assert_eq!(&[0, 0, 0, 0], &get_valid_camera_positions(&[0, 0], 1, 0));
        assert_eq!(&[0, 0, 0, 0], &get_valid_camera_positions(&[0, 0], 0, 1));

        // Small borders
        assert_eq!(&[1, 0, 8, 0], &get_valid_camera_positions(&[10, 0], 1, 0));
        assert_eq!(&[0, 1, 0, 8], &get_valid_camera_positions(&[0, 10], 0, 1));
        assert_eq!(&[1, 2, 18, 7], &get_valid_camera_positions(&[20, 10], 1, 2));

        // Large border for either
        assert_eq!(&[9, 1, 9, 8], &get_valid_camera_positions(&[20, 10], 10, 1));
        assert_eq!(
            &[1, 4, 18, 4],
            &get_valid_camera_positions(&[20, 10], 1, 10)
        );
    }
}
