use amethyst::{
    core::{ArcThreadPool, SystemBundle},
    prelude::*,
    renderer::{
        debug_drawing::DebugLinesComponent,
        palette::{Pixel, Srgba},
    },
    shred::{Dispatcher, DispatcherBuilder},
    shrev::EventChannel,
    ui::FontHandle,
};

use crate::systems::movement::update_transforms::UpdateTransformsEvent;

use super::{
    area::{get_world_coordinates, Area, CurrentArea, TILE_HEIGHT, TILE_WIDTH},
    bundle::MovementSystemsBundle,
};

const DEBUG_SPRITE_LAYER: f32 = -1.0;
const BACKGROUND_SPRITE_LAYER: f32 = 0.0;

#[derive(Default)]
pub struct Regular<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for Regular<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.dispatcher = Some(setup_game_system_dispatcher(world));

        init_area(40, 20, world);

        // All rendered entities should have correct `Positions` at this stage
        // but once the camera is set up we need to trigger an update for
        // corresponding `UiTransform`s before the first frame is rendered.
        world
            .write_resource::<EventChannel<UpdateTransformsEvent>>()
            .single_write(UpdateTransformsEvent);

        // Debug grid
        draw_area_grid(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world.res);
        }

        Trans::None
    }
}

fn setup_game_system_dispatcher<'a, 'b>(world: &mut World) -> Dispatcher<'a, 'b> {
    let mut dispatcher_builder = DispatcherBuilder::new();

    MovementSystemsBundle
        .build(&mut dispatcher_builder)
        .expect("failed to register MoveSystemsBundle");

    let mut dispatcher = dispatcher_builder
        .with_pool(world.read_resource::<ArcThreadPool>().clone())
        .build();

    dispatcher.setup(&mut world.res);

    dispatcher
}

fn init_area(size_x: u32, size_y: u32, world: &mut World) {
    let area = world
        .create_entity()
        .with(Area {
            dimensions: [size_x, size_y],
        })
        .build();

    world.add_resource(CurrentArea(area));
}

fn draw_area_grid(world: &mut World) {
    let [nx, ny] = {
        let CurrentArea(entity) = *world.read_resource::<CurrentArea>();
        world.read_storage::<Area>().get(entity).unwrap().dimensions
    };

    let (size_x, size_y) = get_world_coordinates(nx, ny);

    let mut debug_lines = DebugLinesComponent::new();

    let color = Srgba::from_raw(&[110.0 / 255.0, 110.0 / 255.0, 110.0 / 255.0, 0.5]);

    for col in 0..=nx {
        let x = (col * TILE_WIDTH) as f32;
        let start = [x, 0.0, DEBUG_SPRITE_LAYER];
        let end = [x, size_y, DEBUG_SPRITE_LAYER];
        debug_lines.add_line(start.into(), end.into(), color.clone());
    }

    for row in 0..=ny {
        let y = (row * TILE_HEIGHT) as f32;
        let start = [0.0, y, DEBUG_SPRITE_LAYER];
        let end = [size_x, y, DEBUG_SPRITE_LAYER];
        debug_lines.add_line(start.into(), end.into(), color.clone());
    }

    world.create_entity().with(debug_lines).build();
}

