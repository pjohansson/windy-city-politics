use amethyst::{
    core::Transform,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage},
    input::{InputHandler, StringBindings},
    shrev::EventChannel,
};

use crate::{
    area::{Area, CurrentArea, Position},
    game::PlayerCharacter,
};

use super::{
    utils::{update_position, update_transform},
    Action, Move, PlayerActionEvent,
};

/// System to move the player on an area grid.
pub struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        WriteStorage<'s, Position>,
        WriteStorage<'s, Transform>,
        Write<'s, EventChannel<PlayerActionEvent>>,
        ReadStorage<'s, PlayerCharacter>,
        ReadExpect<'s, CurrentArea>,
        ReadStorage<'s, Area>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut transforms,
            mut event_channel,
            character,
            current_area,
            areas,
            input
        ): Self::SystemData,
    ) {
        let dx = input
            .axis_value("move_horizontal")
            .map(|v| v as i32)
            .unwrap_or(0);

        let dy = input
            .axis_value("move_vertical")
            .map(|v| v as i32)
            .unwrap_or(0);

        let direction = match (dx, dy) {
            (_, 1) => Some(Move::Up),
            (_, -1) => Some(Move::Down),
            (-1, _) => Some(Move::Left),
            (1, _) => Some(Move::Right),
            _ => None,
        };

        if let Some(direction) = direction {
            let [area_size_x, area_size_y] = areas.get(current_area.0).unwrap().dimensions;
            let max_x = area_size_x.saturating_sub(1);
            let max_y = area_size_y.saturating_sub(1);

            for (position, transform, _) in (&mut positions, &mut transforms, &character).join() {
                update_position(position, &direction, &[0, 0, max_x, max_y]);
                update_transform(transform, position);
            }

            event_channel.single_write(PlayerActionEvent(Action::Move(direction)));
        }
    }
}
