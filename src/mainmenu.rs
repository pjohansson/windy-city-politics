use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::prelude::{Component, DenseVecStorage, NullStorage},
    prelude::*,
    renderer::{
        debug_drawing::DebugLinesComponent,
        palette::{Pixel, Srgba},
        Camera, Sprite, SpriteRender, SpriteSheet, Texture, Transparent,
    },
    ui::{FontAsset, FontHandle, TtfFormat, UiCreator, UiTransformBuilder, UiWidget},
    window::ScreenDimensions,
};

use crate::{
    area::{get_screen_coordinates, Area, CurrentArea, Position, TILE_HEIGHT, TILE_WIDTH},
    texture::create_texture,
};

pub struct MainMenu;

impl SimpleState for MainMenu {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // world.register::<Fonts>();

        // init_fonts(world);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/mainmenu.ron", ());
        });
    }
}


#[derive(Debug, Default)]
/// Center of UI element, used as a parent to group elements with a common transform.
pub struct Center;

impl Component for Center {
    type Storage = NullStorage<Self>;
}


#[derive(Debug)]
pub struct Fonts {
    menu: FontHandle,
}

fn init_fonts(world: &mut World) {
    let fonts = Fonts {
        menu: world.read_resource::<Loader>().load(
            "fonts/LeagueMono-Medium.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        ),
    };

    world.add_resource(fonts);
}

fn draw_main_menu(world: &mut World) {
    // let (width, height) = {
    //     let dimensions = world.read_resource::<ScreenDimensions>();
    //     (dimensions.width(), dimensions.height())
    // };

    // let xmargin = 
    let fonts = world.read_resource::<Fonts>();
    let menu_font_handle = fonts.menu.clone();

    // let mut selection_widget = UiWidget::Container {
    //     transform: UiTransformBuilder::new()
    // }
    // let mut selection_widget = Ui
}
