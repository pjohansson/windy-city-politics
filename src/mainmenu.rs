use amethyst::{
    assets::{AssetStorage, Loader},
    core::{transform::Parent, Hidden},
    ecs::{world::EntitiesRes, Entity, Join},
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
    ui::{FontAsset, FontHandle, TtfFormat, UiCreator, UiTransform},
};

use crate::game::*;

#[derive(Default)]
pub struct MainMenu {
    ui_entity: Option<Entity>,
}

impl SimpleState for MainMenu {
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::P) {
                return Trans::Push(Box::new(Loading::default()));
            } else if [VirtualKeyCode::Q, VirtualKeyCode::Escape]
                .iter()
                .any(|&key| is_key_down(&event, key))
            {
                return Trans::Quit;
            }
        }

        Trans::None
    }

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        world.exec(|mut creator: UiCreator<'_>| {
            self.ui_entity = Some(creator.create("ui/mainmenu.ron", ()));
        });
    }

    fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(ui) = self.ui_entity {
            let world = data.world;

            hide_entity_and_children(ui, world);
        }
    }
}

fn hide_entity_and_children(current_entity: Entity, world: &mut World) {
    for ent in find_children(current_entity, world) {
        hide_entity_and_children(ent, world);
    }

    let mut hidden_store = world.write_storage::<Hidden>();
    hidden_store
        .insert(current_entity, Hidden)
        .expect("could not access Hidden entity storage");
}

fn find_children(current_entity: Entity, world: &World) -> Vec<Entity> {
    let entities = world.read_resource::<EntitiesRes>();
    let parents = world.read_storage::<Parent>();
    let ui_transforms = world.read_storage::<UiTransform>();

    (&*entities, &parents, &ui_transforms)
        .join()
        .filter(|(_, Parent { entity: parent }, _)| parent == &current_entity)
        .map(|(child, _, _)| child)
        .collect::<Vec<_>>()
}
