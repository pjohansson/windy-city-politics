use amethyst::{
    assets::{
        AssetStorage, Completion, Handle, Loader, Prefab, PrefabLoader, ProgressCounter, RonFormat,
    },
    core::{ArcThreadPool, SystemBundle},
    ecs::{world::EntitiesRes, Join, Read, ReadExpect, ReadStorage, WriteStorage},
    prelude::*,
    shred::{Dispatcher, DispatcherBuilder},
    ui::{FontAsset, TtfFormat, UiText},
};

use super::{
    area::TILE_HEIGHT,
    bundle::PrefabLoaderBundle,
    character::*,
    state::{Fonts, Regular},
};

pub struct PrefabLoaderHandles {
    pub player_character: Handle<Prefab<PlayerCharacterPrefab>>,
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
        setup_character_ui_text_components(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world.res);
        }

        match self.progress.complete() {
            Completion::Complete => Trans::Switch(Box::new(Regular::default())),
            Completion::Failed => {
                panic!("could not read all required assets");
            }
            Completion::Loading => Trans::None,
        }
    }
}

fn load_character_entities(world: &mut World) {
    let handle = {
        world
            .read_resource::<PrefabLoaderHandles>()
            .player_character
            .clone()
    };

    world.create_entity().with(handle).build();
}

fn load_fonts(world: &mut World, progress: &mut ProgressCounter) {
    let fonts = {
        let loader = world.read_resource::<Loader>();
        let store = world.read_resource::<AssetStorage<FontAsset>>();

        Fonts {
            main: loader.load("fonts/LeagueMono-Regular.ttf", TtfFormat, progress, &store),
        }
    };

    world.add_resource(fonts);
}

fn setup_character_ui_text_components<'a>(world: &mut World) {
    type SystemData<'a> = (
        WriteStorage<'a, UiText>,
        ReadStorage<'a, CharacterChar>,
        Read<'a, EntitiesRes>,
        ReadExpect<'a, Fonts>,
    );

    world.exec(|(mut ui_texts, chars, entities, fonts): SystemData| {
        let font = &fonts.main;

        for (CharacterChar(c), entity) in (&chars, &entities).join() {
            let text = UiText::new(
                font.clone(),
                c.to_string(),
                [1.0, 1.0, 1.0, 1.0],
                TILE_HEIGHT as f32,
            );

            ui_texts
                .insert(entity, text)
                .expect("could not insert character `UiText` component");
        }
    });
}

fn setup_dispatcher<'a, 'b>(world: &mut World) -> Dispatcher<'a, 'b> {
    let mut dispatcher_builder = DispatcherBuilder::new();

    PrefabLoaderBundle
        .build(&mut dispatcher_builder)
        .expect("failed to register PrefabLoaderBundle");

    let mut dispatcher = dispatcher_builder
        .with_pool(world.read_resource::<ArcThreadPool>().clone())
        .build();

    dispatcher.setup(&mut world.res);

    dispatcher
}

fn setup_prefab_loaders(world: &mut World, progress: &mut ProgressCounter) {
    let handles = {
        let player_character = world.exec(|loader: PrefabLoader<'_, PlayerCharacterPrefab>| {
            loader.load("prefab/playercharacter.ron", RonFormat, progress)
        });

        PrefabLoaderHandles { player_character }
    };

    world.add_resource(handles);
}
