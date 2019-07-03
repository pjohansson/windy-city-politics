use amethyst::{
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, Write, WriteStorage},
    input::{InputHandler, StringBindings},
    shrev::EventChannel,
};

use crate::game::{Area, CurrentArea, PlayerCharacter, Position};

use super::{update_transforms::UpdateTransformsEvent, Move};

/// Moves the `PlayerCharacter` inside the current active `Area`.
pub struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        WriteStorage<'s, Position>,
        Write<'s, EventChannel<UpdateTransformsEvent>>,
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
                move_position(position, &direction, &[0, 0, max_x, max_y]);
            }

            events.single_write(UpdateTransformsEvent);
        }
    }
}

/// Update the input position by moving it along the input direction. 
fn move_position(
    position: &mut Position,
    direction: &Move,
    [min_x, min_y, max_x, max_y]: &[u32; 4],
) {
    match direction {
        Move::Up => position.y = clamp_position(position.y as i32 + 1, *min_y, *max_y),
        Move::Down => position.y = clamp_position(position.y as i32 - 1, *min_y, *max_y),
        Move::Left => position.x = clamp_position(position.x as i32 - 1, *min_x, *max_x),
        Move::Right => position.x = clamp_position(position.x as i32 + 1, *min_x, *max_x),
    }
}

/// Clamp input value to the range [min, max]. Assumes that max >= min.
pub fn clamp_position(position: i32, min: u32, max: u32) -> u32 {
    if position < min as i32 {
        min
    } else if position >= max as i32 {
        max
    } else {
        position as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_clamps_to_closed_range() {
        // Lower than min
        assert_eq!(5, clamp_position(0, 5, 5));
        assert_eq!(0, clamp_position(-1, 0, 5));

        // Higher than max
        assert_eq!(5, clamp_position(11, 0, 5));

        // Same min and max
        assert_eq!(5, clamp_position(4, 5, 5));
        assert_eq!(5, clamp_position(5, 5, 5));
        assert_eq!(5, clamp_position(6, 5, 5));

        // In range
        assert_eq!(1, clamp_position(1, 1, 4));
        assert_eq!(2, clamp_position(2, 1, 4));
        assert_eq!(3, clamp_position(3, 1, 4));
        assert_eq!(4, clamp_position(4, 1, 4));
    }
}
