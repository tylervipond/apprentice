use crate::components::{Container, Inventory, Name};
use specs::{Entity, World, WorldExt};

pub type InventoryList = Vec<(Entity, String)>;

pub fn get_player_inventory_list(ecs: &World) -> InventoryList {
    let player_entity = ecs.fetch::<Entity>();
    let inventories = ecs.read_storage::<Inventory>();
    let player_inventory = inventories.get(*player_entity).unwrap();
    let names = ecs.read_storage::<Name>();
    player_inventory
        .items
        .iter()
        .map(|e| (*e, names.get(*e).unwrap().name.clone()))
        .collect()
}

pub fn get_container_inventory_list(ecs: &World, container_entity: &Entity) -> InventoryList {
    let names = ecs.read_storage::<Name>();
    let containers = ecs.read_storage::<Container>();
    let container = containers.get(*container_entity).unwrap();
    container
        .items
        .iter()
        .map(|e| (*e, names.get(*e).unwrap().name.clone()))
        .collect()
}
