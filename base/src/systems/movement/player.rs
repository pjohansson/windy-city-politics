use amethyst::{
    ecs::prelude::{
        Join, Read, ReadExpect, ReadStorage, Resources, System, SystemData, Write, WriteStorage,
    },
    shrev::{EventChannel, ReaderId},
};

use crate::game::{ActiveArea, Area, Collision, PlayerCharacter, Position};

use super::{update_transforms::UpdateTransformsEvent, Action, Move, PlayerActionEvent};

/// Moves the `PlayerCharacter` inside the current active `Area`.
pub struct PlayerMovementSystem {
    pub reader: Option<ReaderId<PlayerActionEvent>>,
}

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        WriteStorage<'s, Position>,
        Write<'s, EventChannel<UpdateTransformsEvent>>,
        ReadStorage<'s, PlayerCharacter>,
        ReadExpect<'s, ActiveArea>,
        ReadStorage<'s, Area>,
        ReadStorage<'s, Collision>,
        Read<'s, EventChannel<PlayerActionEvent>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut positions, mut events, character, current_area, areas, collisions, event_channel) =
            data;

        for event in event_channel.read(self.reader.as_mut().unwrap()) {
            if let PlayerActionEvent(Action::Move(direction)) = event {
                let [area_size_x, area_size_y] = areas.get(current_area.0).unwrap().dimensions;
                let max_x = area_size_x.saturating_sub(1);
                let max_y = area_size_y.saturating_sub(1);

                let mut occupied_positions = Vec::new();

                for (position, _, _) in (&positions, &collisions, !&character).join() {
                    occupied_positions.push(position.clone());
                }

                for (position, _) in (&mut positions, &character).join() {
                    let destination = get_destination(position, &direction, &[0, 0, max_x, max_y]);

                    if !occupied_positions.contains(&destination) {
                        position.x = destination.x;
                        position.y = destination.y;
                    }
                }

                events.single_write(UpdateTransformsEvent);
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

fn get_destination(
    position: &Position,
    direction: &Move,
    [min_x, min_y, max_x, max_y]: &[u32; 4],
) -> Position {
    let mut new_position = position.clone();

    match direction {
        Move::Up => new_position.y = clamp_position(position.y as i32 + 1, *min_y, *max_y),
        Move::Down => new_position.y = clamp_position(position.y as i32 - 1, *min_y, *max_y),
        Move::Left => new_position.x = clamp_position(position.x as i32 - 1, *min_x, *max_x),
        Move::Right => new_position.x = clamp_position(position.x as i32 + 1, *min_x, *max_x),
    }

    new_position
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
