use amethyst::{
    ecs::prelude::{Read, System, Write},
    input::{InputHandler, StringBindings},
    shrev::EventChannel,
};

use crate::{
    config::Config,
    systems::movement::{Action, Move, PlayerActionEvent},
};

use std::time::{Duration, Instant};

/// Looks for input events and sends signals to systems.
pub struct InputSystem {
    dirty: bool,
    hold_start: Option<Instant>,
    last_event: Option<Instant>,
}

impl Default for InputSystem {
    fn default() -> Self {
        InputSystem {
            dirty: false,
            hold_start: None,
            last_event: None,
        }
    }
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        Write<'s, EventChannel<PlayerActionEvent>>,
        Read<'s, Config>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event_channel, config, input) = data;

        let action = parse_move_event(&input).or_else(|| parse_action_event(&input));

        if let Some(action) = action {
            let current_time = Instant::now();

            let allow_new_event = match self.hold_start {
                Some(hold_start) => {
                    let allow_hold_events =
                        current_time.duration_since(hold_start) >= config.min_duration_hold;

                    let allow_repeat_event = self
                        .last_event
                        .map(|time| current_time.duration_since(time) >= config.min_duration_repeat)
                        .unwrap_or(false);

                    allow_hold_events && allow_repeat_event
                }
                None => {
                    self.hold_start.replace(current_time);
                    true
                }
            };

            if !self.dirty || allow_new_event {
                event_channel.single_write(PlayerActionEvent(action));

                self.dirty = true;
                self.last_event = Some(current_time);
            }
        } else {
            self.dirty = false;
            self.hold_start = None;
        }
    }
}

fn parse_action_event<'s>(input: &Read<'s, InputHandler<StringBindings>>) -> Option<Action> {
    input
        .action_is_down("action")
        .and_then(|value| if value { Some(Action::Action) } else { None })
}

fn parse_move_event<'s>(input: &Read<'s, InputHandler<StringBindings>>) -> Option<Action> {
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

    direction.map(|dir| Action::Move(dir))
}
