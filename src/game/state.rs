use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, ProgressCounter, RonFormat},
    core::{transform::Transform, ArcThreadPool, SystemBundle},
    ecs::{
        prelude::{
            Component, DenseVecStorage, Join, NullStorage, Read, ReadStorage, VecStorage,
            WriteStorage,
        },
        world::EntitiesRes,
    },
    prelude::*,
    renderer::{
        debug_drawing::DebugLinesComponent,
        palette::{Pixel, Srgba},
        ActiveCamera, Camera, Sprite, SpriteRender, SpriteSheet, Texture, Transparent,
    },
    shred::{Dispatcher, DispatcherBuilder},
    ui::{Anchor, FontHandle, UiText, UiTransform},
    window::ScreenDimensions,
};

use crate::{
    render::get_screen_center_coordinates,
    systems::movement::update_transforms::{
        get_active_camera_position, get_screen_absolute_coordinates_for_entity_grid_position,
    },
    texture::create_texture,
};

use super::{
    area::{get_world_coordinates, Area, CurrentArea, Position, TILE_HEIGHT, TILE_WIDTH},
    bundle::{MovementSystemsBundle, PrefabLoaderBundle},
    character::*,
};

const DEBUG_SPRITE_LAYER: f32 = -1.0;
const BACKGROUND_SPRITE_LAYER: f32 = 0.0;
const PLAYER_SPRITE_LAYER: f32 = 1.0;
const CAMERA_POSITION_Z: f32 = 10.0;

pub struct Fonts {
    pub main: FontHandle,
}

#[derive(Default)]
pub struct Regular<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for Regular<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.dispatcher = Some(setup_game_system_dispatcher(world));

        init_area(40, 20, world);
        init_camera(20, 10, world);
        init_player_character(20, 10, world);

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

fn init_camera(x: u32, y: u32, world: &mut World) {
    let (width, height) = {
        let dimensions = world.read_resource::<ScreenDimensions>();
        (dimensions.width(), dimensions.height())
    };

    let (xs, ys) = get_world_coordinates(x, y);
    let mut transform = Transform::default();
    transform.set_translation_xyz(xs, ys, CAMERA_POSITION_Z);

    let camera = world
        .create_entity()
        .with(Camera::standard_2d(width, height))
        .with(Position { x, y })
        .with(transform)
        .build();

    *world.write_resource::<ActiveCamera>() = ActiveCamera {
        entity: Some(camera),
    };
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

fn init_player_character(x: u32, y: u32, world: &mut World) {
    let transform = {
        let screen_center =
            get_screen_center_coordinates(&world.read_resource::<ScreenDimensions>());

        let camera_position = get_active_camera_position(
            &world.read_resource::<ActiveCamera>(),
            &world.read_storage::<Position>(),
        );

        let (xs, ys) = get_screen_absolute_coordinates_for_entity_grid_position(
            screen_center,
            &camera_position,
            &Position { x, y },
        );

        UiTransform::new(
            "player_character".to_string(),
            Anchor::BottomLeft,
            Anchor::Middle,
            xs,
            ys,
            PLAYER_SPRITE_LAYER,
            TILE_WIDTH as f32,
            TILE_HEIGHT as f32,
        )
    };

    type SystemData<'a> = (
        WriteStorage<'a, UiTransform>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, CharacterChar>,
        Read<'a, EntitiesRes>,
    );

    world.exec(
        |(mut transforms, mut positions, chars, entities): SystemData| {
            for (_, entity) in (&chars, &entities).join() {
                transforms
                    .insert(entity, transform.clone())
                    .expect("could not insert character `Transform` component");

                positions
                    .insert(entity, Position { x, y })
                    .expect("could not insert character `Position` component");
            }
        },
    );
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