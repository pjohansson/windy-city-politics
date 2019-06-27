mod camera;
mod player;
mod utils;

pub use camera::CameraMovementSystem;
pub use player::PlayerMovementSystem;

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
