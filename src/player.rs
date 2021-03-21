use crate::components::{
    door::DoorState, equipable::EquipmentPositions, BlocksTile, Container, Disarmable, Door,
    Grabbing, Hidden, HidingSpot, Item, Monster, Position, Trap, WantsToCloseDoor,
    WantsToDisarmTrap, WantsToDouse, WantsToEquip, WantsToGoDownStairs, WantsToGoUpStairs,
    WantsToGrab, WantsToHide, WantsToLight, WantsToMelee, WantsToMove, WantsToOpenDoor,
    WantsToPickUpItem, WantsToReleaseGrabbed, WantsToSearchHidden, WantsToTrap, WantsToUse,
};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::entity_option::EntityOption;
use crate::interaction_type::InteractionType;
use specs::{Component, Entity, World, WorldExt};
use std::collections::HashSet;

pub fn move_to_position(world: &mut World, idx: usize) {
    let player_entity = world.fetch::<Entity>();
    world
        .write_storage::<WantsToMove>()
        .insert(*player_entity, WantsToMove { idx })
        .expect("couldn't insert player move intent");
}

fn insert_intent<T: Component>(
    world: &mut World,
    intent: T,
) -> Result<Option<T>, specs::error::Error> {
    let player_entity = world.fetch::<Entity>();
    let mut intents = world.write_storage::<T>();
    intents.insert(*player_entity, intent)
}

pub fn use_item(world: &mut World, item: Entity, target: Option<usize>) {
    let is_trap = {
        let traps = world.read_storage::<Trap>();
        traps.get(item).is_some()
    };
    match is_trap {
        true => {
            insert_intent(world, WantsToTrap { item, target })
                .expect("Unable To Insert Trap Intent");
        }
        false => {
            insert_intent(world, WantsToUse { item, target })
                .expect("Unable To Insert Use Item Intent");
        }
    }
}

pub fn open_door(world: &mut World, door: Entity) {
    insert_intent(world, WantsToOpenDoor { door })
        .expect("could not insert wants to open door for player");
}

pub fn close_door(world: &mut World, door: Entity) {
    insert_intent(world, WantsToCloseDoor { door })
        .expect("could not insert wants to close door for player");
}

pub fn search_hidden(world: &mut World) {
    insert_intent(world, WantsToSearchHidden {})
        .expect("could not insert wants to search hidden for player");
}

pub fn disarm_trap(world: &mut World, item: Entity) {
    insert_intent(world, WantsToDisarmTrap { trap: item })
        .expect("Unable to Insert Disarm Trap Intent");
}

pub fn arm_trap(world: &mut World, item: Entity) {
    let target = {
        let positions = world.read_storage::<Position>();
        match positions.get(item) {
            Some(pos) => Some(pos.idx),
            None => None,
        }
    };
    insert_intent(world, WantsToTrap { item, target }).expect("Unable to Insert Trap Intent");
}

pub fn grab_entity(world: &mut World, entity: Entity) {
    insert_intent(
        world,
        WantsToGrab {
            thing: EntityOption::new(Some(entity)),
        },
    )
    .expect("Unable to Insert Grab Intent");
}

pub fn release_entity(world: &mut World) {
    insert_intent(world, WantsToReleaseGrabbed {}).expect("Unable to Insert Release Intent");
}

pub fn attack_entity(world: &mut World, entity: Entity) {
    insert_intent(world, WantsToMelee { target: entity })
        .expect("Unable to Insert Wants To Melee Intent");
}

pub fn hide_in_container(world: &mut World, entity: Entity) {
    insert_intent(
        world,
        WantsToHide {
            hiding_spot: Some(entity),
        },
    )
    .expect("Unable to Insert Hide in Container Intent");
}

pub fn equip_item(world: &mut World, entity: Option<Entity>, position: EquipmentPositions) {
    insert_intent(
        world,
        WantsToEquip {
            equipment: entity,
            position,
        },
    )
    .expect("Unable to Insert Hide in Container Intent");
}

pub fn douse_item(world: &mut World, item: Entity) {
    insert_intent(world, WantsToDouse { item }).expect("Unable to Insert Douse Intent");
}

pub fn light_item(world: &mut World, item: Entity) {
    insert_intent(world, WantsToLight { item }).expect("Unable to Insert Douse Intent");
}

pub fn pickup_items(world: &mut World, items: HashSet<Entity>, container: Option<Entity>) {
    insert_intent(world, WantsToPickUpItem { container, items })
        .expect("Unable to insert want to pick up");
}

pub fn pickup_item(world: &mut World, item: Entity, container: Option<Entity>) {
    let mut items = HashSet::new();
    items.insert(item);
    pickup_items(world, items, container);
}

pub fn go_up_stairs(world: &mut World, idx: usize) {
    insert_intent(world, WantsToGoUpStairs { idx })
        .expect("Unable to insert wants to go up stairs");
}

pub fn go_down_stairs(world: &mut World, idx: usize) {
    insert_intent(world, WantsToGoDownStairs { idx })
        .expect("Unable to insert want to go down stairs");
}

pub fn interact(world: &mut World, interaction_type: InteractionType) {
    match interaction_type {
        InteractionType::Douse(ent) => douse_item(world, ent),
        InteractionType::Light(ent) => light_item(world, ent),
        InteractionType::HideIn(ent) => hide_in_container(world, ent),
        InteractionType::Grab(ent) => grab_entity(world, ent),
        InteractionType::Disarm(ent) => disarm_trap(world, ent),
        InteractionType::Arm(ent) => arm_trap(world, ent),
        InteractionType::Attack(ent) => attack_entity(world, ent),
        InteractionType::Pickup(ent) => pickup_item(world, ent, None),
        InteractionType::OpenDoor(ent) => open_door(world, ent),
        InteractionType::CloseDoor(ent) => close_door(world, ent),
        InteractionType::GoDown(idx) => go_down_stairs(world, idx),
        InteractionType::GoUp(idx) => go_up_stairs(world, idx),
        InteractionType::Move(idx) => move_to_position(world, idx),
        InteractionType::Release => release_entity(world),
        _ => {}
    }
}

fn get_entity_is_visible_to_player(world: &World, entity: &Entity) -> bool {
    let hidden_storage = world.read_storage::<Hidden>();
    let player_ent = world.fetch::<Entity>();
    match hidden_storage.get(*entity) {
        Some(hidden) => hidden.found_by.contains(&*player_ent),
        None => true,
    }
}

fn get_entity_with_component_for_auto_act<T: Component>(
    world: &World,
    ents: &Vec<Entity>,
) -> Option<Entity> {
    let component_storage = world.read_storage::<T>();
    ents.iter()
        .filter(|e| {
            component_storage.get(**e).is_some() && get_entity_is_visible_to_player(world, e)
        })
        .map(|e| *e)
        .next()
}

fn get_open_door_entity_at_idx(world: &World, ents: &Vec<Entity>) -> Option<Entity> {
    let doors = world.read_storage::<Door>();
    ents.iter()
        .filter(|e| {
            let door = doors.get(**e);
            match door {
                Some(d) => d.state == DoorState::Closed,
                None => false,
            }
        })
        .map(|e| *e)
        .next()
}
// this returns interaction type
pub fn get_default_action(world: &World, delta_x: i32, delta_y: i32) -> InteractionType {
    let positions = world.read_storage::<Position>();
    let player_entity = world.fetch::<Entity>();
    let player_position = positions.get(*player_entity).unwrap();
    let floor = player_position.level;
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(floor).unwrap();
    let level_width = level.width as i32;
    let destination_index = level_utils::add_xy_to_idx(
        level_width as i32,
        delta_x as i32,
        delta_y as i32,
        player_position.idx as i32,
    ) as usize;
    let ents = &level.tile_content[destination_index];
    let stairs_up_idx = level.stairs_up;
    let stairs_down_idx = level.stairs_down;
    let exit_idx = level.exit;

    if let Some(e) = get_entity_with_component_for_auto_act::<Container>(world, &ents) {
        let blocks_storage = world.read_storage::<BlocksTile>();
        if blocks_storage.get(e).is_some() {
            return InteractionType::OpenContainer(e);
        }
    }
    if let Some(e) = get_entity_with_component_for_auto_act::<HidingSpot>(world, &ents) {
        let grabbing_storage = world.read_storage::<Grabbing>();
        let grabbed = match grabbing_storage.get(*player_entity) {
            Some(grabbing) => grabbing.thing == e,
            None => false
        };
        if !grabbed {
            return InteractionType::HideIn(e);
        }
    }
    if let Some(e) = get_open_door_entity_at_idx(world, &ents) {
        return InteractionType::OpenDoor(e);
    }
    if let Some(e) = get_entity_with_component_for_auto_act::<Monster>(world, &ents) {
        return InteractionType::Attack(e);
    }
    if let Some(e) = get_entity_with_component_for_auto_act::<Disarmable>(world, &ents) {
        return InteractionType::Disarm(e);
    }
    if let Some(e) = get_entity_with_component_for_auto_act::<Item>(world, &ents) {
        return InteractionType::Pickup(e);
    }
    let idx_is_stairs_down = match stairs_down_idx {
        Some(i) => i == destination_index,
        None => false,
    };
    if idx_is_stairs_down {
        return InteractionType::GoDown(destination_index);
    }
    let idx_is_stairs_up = match stairs_up_idx {
        Some(i) => i == destination_index,
        None => false,
    };
    if idx_is_stairs_up {
        return InteractionType::GoUp(destination_index);
    }
    let idx_is_exit = match exit_idx {
        Some(i) => i == destination_index,
        None => false,
    };
    if idx_is_exit {
        return InteractionType::Exit(destination_index);
    }
    InteractionType::Move(destination_index)
}
