mod camera;
mod player;
pub mod update_transforms;

pub use camera::CameraMovementSystem;
pub use player::PlayerMovementSystem;
pub use update_transforms::{UpdateCharTileTransformsSystem, UpdateTransformsSystem};

#[derive(Debug)]
/// Event emitted if the player character has done something.
pub struct PlayerActionEvent(pub Action);

#[derive(Debug)]
pub enum Action {
    Action,
    Move(Move),
}

#[derive(Debug)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}
