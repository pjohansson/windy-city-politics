use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, NullStorage},
    prelude::*,
    renderer::{
        debug_drawing::DebugLinesComponent,
        palette::{Pixel, Srgba},
        Camera, Sprite, SpriteRender, SpriteSheet, Texture, Transparent,
    },
    window::ScreenDimensions,
};

use crate::{
    area::{get_screen_coordinates, Area, CurrentArea, Position, TILE_HEIGHT, TILE_WIDTH},
    texture::create_texture,
};

const DEBUG_SPRITE_LAYER: f32 = -1.0;
const BACKGROUND_SPRITE_LAYER: f32 = 0.0;
const PLAYER_SPRITE_LAYER: f32 = 1.0;
const CAMERA_POSITION_Z: f32 = 10.0;

#[derive(Default)]
pub struct PlayerCharacter;

impl Component for PlayerCharacter {
    type Storage = NullStorage<Self>;
}

// #[derive(Default)]
// struct Collision;

// impl Component for Collision {
//     type Storage = NullStorage<Self>;
// }

#[derive(Default)]
pub struct Regular;

impl SimpleState for Regular {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_area(40, 20, world);
        init_player_character(20, 10, world);
        init_camera(20, 10, world);

        // Debug grid
        draw_area_grid(world);
    }
}

fn init_camera(x: u32, y: u32, world: &mut World) {
    let (width, height) = {
        let dimensions = world.read_resource::<ScreenDimensions>();
        (dimensions.width(), dimensions.height())
    };

    let (xs, ys) = get_screen_coordinates(x, y);
    let mut transform = Transform::default();
    transform.set_translation_xyz(xs, ys, CAMERA_POSITION_Z);

    world
        .create_entity()
        .with(Camera::standard_2d(width, height))
        .with(Position { x, y })
        .with(transform)
        .build();
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
    let (xs, ys) = get_screen_coordinates(x, y);
    let mut transform = Transform::default();
    transform.set_translation_xyz(xs, ys, PLAYER_SPRITE_LAYER);

    let texture = {
        let loader = world.read_resource::<Loader>();
        let store = world.read_resource::<AssetStorage<Texture>>();

        // Checkerboard pattern by reversing every other row
        let pair = [[210, 210, 210, 255], [0, 0, 0, 0]];
        let row = pair.iter().cycle().take(TILE_WIDTH as usize);
        let row_reversed = pair.iter().rev().cycle().take(TILE_WIDTH as usize);

        let data = row
            .chain(row_reversed)
            .cycle()
            .take((TILE_WIDTH * TILE_HEIGHT) as usize)
            .cloned()
            .collect::<Vec<_>>();

        create_texture(&data, (TILE_WIDTH, TILE_HEIGHT), &store, &loader, ()).unwrap()
    };

    let sprite_sheet = {
        let loader = world.read_resource::<Loader>();
        let store = world.read_resource::<AssetStorage<SpriteSheet>>();

        let sprite = Sprite::from_pixel_values(
            TILE_WIDTH,
            TILE_HEIGHT,
            TILE_WIDTH,
            TILE_HEIGHT,
            0,
            0,
            [-((TILE_WIDTH / 2) as f32), -((TILE_HEIGHT / 2) as f32)],
            false,
            false,
        );

        let data = SpriteSheet {
            texture,
            sprites: vec![sprite],
        };

        loader.load_from_data(data, (), &store)
    };

    world
        .create_entity()
        .with(transform)
        .with(PlayerCharacter)
        .with(Position { x, y })
        .with(SpriteRender {
            sprite_sheet,
            sprite_number: 0,
        })
        .with(Transparent)
        .build();
}

fn draw_area_grid(world: &mut World) {
    let [nx, ny] = {
        let CurrentArea(entity) = *world.read_resource::<CurrentArea>();
        world.read_storage::<Area>().get(entity).unwrap().dimensions
    };

    let (size_x, size_y) = get_screen_coordinates(nx, ny);

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
