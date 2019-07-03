use amethyst::{
    assets::{Completion, Handle, Prefab, PrefabLoader, ProgressCounter, RonFormat},
    core::{ArcThreadPool, SystemBundle, Transform},
    ecs::Join,
    prelude::{Builder, GameData, SimpleState, SimpleTrans, StateData, Trans, World},
    renderer::{ActiveCamera, Camera},
    shred::{Dispatcher, DispatcherBuilder},
    window::ScreenDimensions,
};

use std::borrow::BorrowMut;

use super::{
    area::Position,
    assets::load_fonts,
    bundle::PrefabLoaderBundle,
    character::{CharacterPrefab, PlayerCharacter},
    consts::CAMERA_POSITION_Z,
    state::Regular,
};

pub struct PrefabLoaderHandles {
    pub character: Handle<Prefab<CharacterPrefab>>,
    pub player_character: Handle<Prefab<CharacterPrefab>>,
}

/// Load all required assets and prefabs, then set up all components
/// and switch to the game state.
pub struct Loading<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    progress: ProgressCounter,
}

impl<'a, 'b> Default for Loading<'a, 'b> {
    fn default() -> Self {
        Loading {
            dispatcher: None,
            progress: ProgressCounter::new(),
        }
    }
}

impl<'a, 'b> SimpleState for Loading<'a, 'b> {
    fn on_start(&mut self, data: StateData<GameData>) {
        let progress = &mut self.progress;

        let world = data.world;

        self.dispatcher.replace(setup_dispatcher(world));

        load_fonts(world, progress);

        setup_prefab_loaders(world, progress);
        load_character_entities(world);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let world = data.world;
        init_camera(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world.res);
        }

        match self.progress.complete() {
            Completion::Complete => Trans::Switch(Box::new(Regular::default())),
            Completion::Loading => Trans::None,
            Completion::Failed => {
                panic!("could not read all required assets");
            }
        }
    }
}

fn setup_dispatcher<'a, 'b>(world: &mut World) -> Dispatcher<'a, 'b> {
    let mut dispatcher_builder = DispatcherBuilder::new();

    PrefabLoaderBundle
        .build(&mut dispatcher_builder)
        .expect("failed to register `PrefabLoaderBundle`");

    let mut dispatcher = dispatcher_builder
        .with_pool(world.read_resource::<ArcThreadPool>().clone())
        .build();

    dispatcher.setup(&mut world.res);

    dispatcher
}

/****************************************
 * Entity and component setup functions *
 ****************************************/

fn init_camera(world: &mut World) {
    let (width, height) = {
        let dimensions = world.read_resource::<ScreenDimensions>();
        (dimensions.width(), dimensions.height())
    };

    let position = {
        let positions = world.read_storage::<Position>();
        let characters = world.read_storage::<PlayerCharacter>();

        (&positions, &characters)
            .join()
            .map(|(position, _)| position)
            .next()
            .cloned()
            .unwrap_or(Position { x: 0, y: 0 })
    };

    let mut transform = Transform::default();
    transform.set_translation_z(CAMERA_POSITION_Z);

    let camera = world
        .create_entity()
        .with(Camera::standard_2d(width, height))
        .with(position)
        .with(transform)
        .build();

    *world.write_resource::<ActiveCamera>() = ActiveCamera {
        entity: Some(camera),
    };
}

fn load_character_entities(world: &mut World) {
    let prefab_handles = {
        let character = world
            .read_resource::<PrefabLoaderHandles>()
            .character
            .clone();

        let player_character = world
            .read_resource::<PrefabLoaderHandles>()
            .player_character
            .clone();

        vec![character, player_character]
    };

    for handle in prefab_handles {
        world.create_entity().with(handle).build();
    }
}

fn setup_prefab_loaders(world: &mut World, progress: &mut ProgressCounter) {
    let handles = {
        let character = world.exec(|loader: PrefabLoader<'_, CharacterPrefab>| {
            loader.load("prefab/character.ron", RonFormat, progress.borrow_mut())
        });

        let player_character = world.exec(|loader: PrefabLoader<'_, CharacterPrefab>| {
            loader.load(
                "prefab/playercharacter.ron",
                RonFormat,
                progress.borrow_mut(),
            )
        });

        PrefabLoaderHandles {
            character,
            player_character,
        }
    };

    world.add_resource(handles);
}
