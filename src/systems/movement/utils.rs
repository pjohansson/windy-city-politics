use crate::game::Position;

use super::Move;

/// Update the input position, clamping at given edges.
pub fn update_position(
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
