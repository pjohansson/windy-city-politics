use amethyst::{
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage},
    input::{InputHandler, StringBindings},
    shrev::EventChannel,
};

use crate::game::{Area, CurrentArea, PlayerCharacter, Position};

use super::{utils::update_position, Action, Move, PlayerActionEvent};

/// Moves the `PlayerCharacter` inside the current active `Area`.
pub struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        WriteStorage<'s, Position>,
        Write<'s, EventChannel<PlayerActionEvent>>,
        ReadStorage<'s, PlayerCharacter>,
        ReadExpect<'s, CurrentArea>,
        ReadStorage<'s, Area>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (mut positions, mut events, character, current_area, areas, input): Self::SystemData,
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

            for (position, _) in (&mut positions, &character).join() {
                update_position(position, &direction, &[0, 0, max_x, max_y]);
            }

            events.single_write(PlayerActionEvent(Action::Move(direction)));
        }
    }
}
