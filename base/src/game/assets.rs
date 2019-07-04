use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    ecs::World,
    ui::{FontAsset, FontHandle, TtfFormat},
};

pub struct Fonts {
    pub main: FontHandle,
}

pub fn load_fonts(world: &mut World, progress: &mut ProgressCounter) {
    let fonts = {
        let loader = world.read_resource::<Loader>();
        let store = world.read_resource::<AssetStorage<FontAsset>>();

        Fonts {
            main: loader.load("fonts/LeagueMono-Regular.ttf", TtfFormat, progress, &store),
        }
    };

    world.add_resource(fonts);
}
