mod area;
mod bundle;
mod game;
mod mainmenu;
mod render;
mod systems;
mod texture;

use amethyst::{
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::{Application, GameDataBuilder},
    renderer::{types::DefaultBackend, RenderingSystem},
    ui::UiBundle,
    utils::application_root_dir,
    window::WindowBundle,
};

use bundle::SpriteBundle;
use mainmenu::MainMenu;
use render::ExampleGraph;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let binding_path = app_root.join("resources").join("bindings_config.ron");
    let display_config_path = app_root.join("resources").join("display_config.ron");

    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(UiBundle::<DefaultBackend, StringBindings>::new())?
        .with_bundle(SpriteBundle)?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            ExampleGraph::default(),
        ));

    let assets_dir = app_root.join("assets");
    let mut game = Application::new(assets_dir, MainMenu::default(), game_data)?;
    game.run();

    Ok(())
}
