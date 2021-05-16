#[cfg(debug_assertions)]
use crate::debug;
use crate::{
    components::{
        door::DoorState, equipable::EquipmentPositions, Armable, CombatStats, Container,
        Disarmable, Door, Dousable, Equipable, Equipment, Grabbable, Grabbing, Hidden, HidingSpot,
        Info, Inventory, Item, Lightable, Name, Objective, Position, Ranged, Trap, Viewshed,
        WantsToDropItem,
    },
    copy,
    dungeon::{dungeon::Dungeon, level::Level, level_builders, level_utils, tile_type::TileType},
    interaction_type::InteractionType,
    inventory,
    menu::{Menu, MenuOption, MenuOptionState},
    music::Music,
    patches, persistence, player, ranged,
    run_state::{RunState, TargetIntent},
    screens::{
        ScreenCredits, ScreenDeath, ScreenFailure, ScreenIntro, ScreenLoading, ScreenMainMenu,
        ScreenMapGeneric, ScreenMapInteractMenu, ScreenMapInteractTarget, ScreenMapMenu,
        ScreenMapNestedMenu, ScreenMapTargeting, ScreenNewGame, ScreenOptions, ScreenSaving,
        ScreenSetKey, ScreenSuccess,
    },
    services::GameLog,
    settings::Settings,
    spawner,
    systems::{
        BloodSpawnSystem, CloseDoorSystem, CorpseSpawnSystem, DamageSystem, DebrisSpawnSystem,
        DisarmTrapSystem, DouseItemSystem, EquipSystem, FireBurnSystem, FireDieSystem,
        FireSpreadSystem, GoDownStairsSystem, GoUpStairsSystem, GrabSystem, HideSystem,
        ItemCollectionSystem, ItemDropSystem, ItemSpawnSystem, LightItemSystem, LightSystem,
        MapIndexingSystem, MeleeCombatSystem, MemoryCullSystem, MonsterAI, MoveSystem,
        OpenDoorSystem, ParticleSpawnSystem, ReleaseSystem, RemoveParticleEffectsSystem,
        RemoveTriggeredTrapsSystem, RevealTrapsSystem, SearchForHiddenSystem, SetTrapSystem,
        TrapSpawnSystem, TriggerSystem, UpdateMemoriesSystem, UpdateParticleEffectsSystem,
        UseItemSystem, VisibilitySystem,
    },
    types::EquipMenuType,
    user_actions::{
        InteractionTargetingAction, MapAction, MenuAction, StaticAction, TargetingAction,
    },
    utils, world_utils,
};
use rltk::{a_star_search, GameState, NavigationPath, RandomNumberGenerator, Rltk};
use specs::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    iter,
    iter::FromIterator,
    iter::IntoIterator,
};

fn player_can_leave_dungeon(world: &mut World) -> bool {
    let player_level = utils::get_current_level_from_world(world);
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(player_level).unwrap();
    match level.exit {
        Some(exit_idx) => {
            let player_entity = world.fetch::<Entity>();
            let player_position = world
                .read_storage::<Position>()
                .get(*player_entity)
                .unwrap()
                .idx;
            player_position == exit_idx
        }
        None => false,
    }
}

fn has_objective_in_backpack(world: &World) -> bool {
    let player_ent = world.fetch::<Entity>();
    let inventories = world.read_storage::<Inventory>();
    let player_inventory = inventories.get(*player_ent).unwrap();
    let entities = world.entities();
    let objectives = world.read_storage::<Objective>();
    for (entity, _objective) in (&entities, &objectives).join() {
        if player_inventory.items.contains(&entity) {
            return true;
        }
    }
    false
}

fn get_visible_entities(world: &World) -> Box<[Entity]> {
    let player_ent = world.fetch::<Entity>();
    let viewsheds = world.read_storage::<Viewshed>();
    let player_viewshed = viewsheds.get(*player_ent).unwrap();
    let positions = world.read_storage::<Position>();
    let player_position = positions.get(*player_ent).unwrap();
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(player_position.level).unwrap();
    let hiddens = world.read_storage::<Hidden>();
    player_viewshed
        .visible_tiles
        .iter()
        .map(|idx| {
            level
                .tile_content
                .get(*idx as usize)
                .unwrap()
                .iter()
                .filter(|e| match hiddens.get(**e) {
                    Some(hidden) => hidden.found_by.contains(&*player_ent),
                    None => true,
                })
        })
        .flatten()
        .map(|e| *e)
        .collect()
}
fn get_visible_entities_at_idx(world: &World, idx: usize) -> Box<[Entity]> {
    let player_ent = world.fetch::<Entity>();
    let viewsheds = world.read_storage::<Viewshed>();
    let positions = world.read_storage::<Position>();
    let player_position = positions.get(*player_ent).unwrap();
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(player_position.level).unwrap();
    let hiddens = world.read_storage::<Hidden>();

    level
        .tile_content
        .get(idx)
        .unwrap()
        .iter()
        .filter(|e| match hiddens.get(**e) {
            Some(hidden) => hidden.found_by.contains(&*player_ent),
            None => true,
        })
        .map(|e| *e)
        .collect()
}

fn get_openable_doors(world: &World, entities: Box<[Entity]>) -> Box<[Entity]> {
    let doors = world.read_storage::<Door>();
    entities
        .iter()
        .filter(|e| match doors.get(**e) {
            Some(d) => d.state == DoorState::Closed,
            None => false,
        })
        .map(|e| *e)
        .collect()
}

fn filter_for_component<T: Component>(world: &World, entities: Box<[Entity]>) -> Box<[Entity]> {
    let interactables = world.read_storage::<T>();
    entities
        .iter()
        .filter(|e| interactables.get(**e).is_some())
        .map(|e| *e)
        .collect()
}

fn get_interaction_type_targets<T: Component>(world: &World) -> Box<[Entity]> {
    filter_for_component::<T>(world, get_visible_entities(world))
}

fn filter_targets_are_interactable(world: &World, targets: Box<[Entity]>) -> Box<[Entity]> {
    let grabbables = world.read_storage::<Grabbable>();
    let lightables = world.read_storage::<Lightable>();
    let dousables = world.read_storage::<Dousable>();
    let disarmables = world.read_storage::<Trap>();
    let hideables = world.read_storage::<HidingSpot>();
    let combat_stats = world.read_storage::<CombatStats>();
    let containers = world.read_storage::<Container>();
    let items = world.read_storage::<Item>();
    let doors = world.read_storage::<Door>();
    targets
        .iter()
        .filter(|e| {
            grabbables.get(**e).is_some()
                || lightables.get(**e).is_some()
                || dousables.get(**e).is_some()
                || disarmables.get(**e).is_some()
                || hideables.get(**e).is_some()
                || combat_stats.get(**e).is_some()
                || items.get(**e).is_some()
                || containers.get(**e).is_some()
                || doors.get(**e).is_some()
        })
        .map(|e| *e)
        .collect()
}

fn get_interaction_targets(world: &World) -> Box<[Entity]> {
    filter_targets_are_interactable(world, get_visible_entities(world))
}

fn get_interaction_targets_at_idx(world: &World, idx: usize) -> Box<[Entity]> {
    filter_targets_are_interactable(world, get_visible_entities_at_idx(world, idx))
}

fn get_interaction_options_for_target(world: &World, target: Entity) -> Vec<InteractionType> {
    let mut interactions: Vec<InteractionType> = vec![];
    if world.read_storage::<Disarmable>().get(target).is_some() {
        interactions.push(InteractionType::Disarm(target));
    }
    if world.read_storage::<Armable>().get(target).is_some() {
        interactions.push(InteractionType::Arm(target));
    }
    if world.read_storage::<Dousable>().get(target).is_some() {
        interactions.push(InteractionType::Douse(target));
    }
    if world.read_storage::<Lightable>().get(target).is_some() {
        interactions.push(InteractionType::Light(target));
    }
    let player_ent = *(world.fetch::<Entity>());
    let releasable = match world.read_storage::<Grabbing>().get(player_ent) {
        Some(grabbing) => grabbing.thing == target,
        None => false,
    };
    if releasable {
        interactions.push(InteractionType::Release)
    } else if world.read_storage::<Grabbable>().get(target).is_some() {
        interactions.push(InteractionType::Grab(target));
    }
    if world.read_storage::<HidingSpot>().get(target).is_some() {
        interactions.push(InteractionType::HideIn(target));
    }
    if world.read_storage::<CombatStats>().get(target).is_some() {
        interactions.push(InteractionType::Attack(target));
    }
    if world.read_storage::<Item>().get(target).is_some() {
        interactions.push(InteractionType::Pickup(target));
    }
    if world.read_storage::<Container>().get(target).is_some() {
        interactions.push(InteractionType::OpenContainer(target));
    }
    if let Some(door) = world.read_storage::<Door>().get(target) {
        interactions.push(match door.state {
            DoorState::Closed => InteractionType::OpenDoor(target),
            DoorState::Opened => InteractionType::CloseDoor(target),
        })
    }
    interactions
}

fn get_menu_from_interaction_options(
    highlighted: usize,
    options: &Vec<InteractionType>,
) -> Menu<&str> {
    let options = options
        .iter()
        .enumerate()
        .map(|(idx, interaction_type)| {
            let name = interaction_type.short_text();
            let state = match idx == highlighted {
                true => MenuOptionState::Highlighted,
                false => MenuOptionState::Normal,
            };
            MenuOption::new(name, state)
        })
        .collect();
    Menu::new(options, 10)
}

fn get_menu_from_ents(world: &World, ents: &Box<[Entity]>, highlight_idx: usize) -> Menu<String> {
    let names = world.read_storage::<Name>();
    let options: Box<[MenuOption<String>]> = ents
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let name = names.get(*e).unwrap();
            let state = match highlight_idx == idx {
                true => MenuOptionState::Highlighted,
                false => MenuOptionState::Normal,
            };
            MenuOption::new(name.name.clone().to_owned(), state)
        })
        .collect();
    Menu::new(options, 10)
}

fn intent_to_interaction_type(intent: TargetIntent, target: Entity) -> InteractionType {
    match intent {
        TargetIntent::HideIn => InteractionType::HideIn(target),
        TargetIntent::Arm => InteractionType::Arm(target),
        TargetIntent::Grab => InteractionType::Grab(target),
        TargetIntent::Attack => InteractionType::Attack(target),
        TargetIntent::OpenDoor => InteractionType::OpenDoor(target),
        TargetIntent::OpenContainer => InteractionType::OpenContainer(target),
        TargetIntent::Disarm => InteractionType::Disarm(target),
        TargetIntent::Douse => InteractionType::Douse(target),
        TargetIntent::Pickup => InteractionType::Pickup(target),
        TargetIntent::Light => InteractionType::Light(target),
    }
}

fn generate_dungeon(world: &mut World) -> Dungeon {
    // maybe this does have to be a hashmap...
    let mut levels = Vec::new();
    let mut main_level = level_builders::build(0, 256, 256, true, true);
    spawner::spawn_entities_for_level(world, &mut main_level);
    levels.push(main_level);
    Dungeon { levels }
}

fn initialize_new_game(world: &mut World) {
    world_utils::initialize_new_game(world);
    let dungeon = generate_dungeon(world);
    let level = dungeon.get_level(0).unwrap();
    let (player_idx, _) = level
        .tiles
        .iter()
        .enumerate()
        .find(|(_, tile_type)| **tile_type == TileType::Exit)
        .unwrap();
    world.remove::<Entity>();
    let player_entity = spawner::spawn_player(world, player_idx, level);
    world.insert(player_entity);
    let rng = world.get_mut::<RandomNumberGenerator>().unwrap();
    // let objective_floor = utils::get_random_between_numbers(rng, 1, 9) as usize;
    let level = dungeon.get_level(0).unwrap();
    let room_idx = utils::get_random_between_numbers(rng, 0, (level.rooms.len() - 1) as i32);
    let room = level.rooms.get(room_idx as usize).unwrap();
    spawner::spawn_objective_for_room(world, &room.rect, &level);
    world.remove::<Dungeon>();
    world.insert(dungeon);
}

fn handle_auto_act(world: &mut World, delta_x: i32, delta_y: i32) -> RunState {
    let interaction = player::get_default_action(world, delta_x, delta_y);
    match interaction {
        InteractionType::OpenContainer(e) => RunState::OpenContainerMenu {
            highlighted: 0,
            container: e,
        },
        InteractionType::Exit(_) => RunState::ExitGameMenu { highlighted: 0 },
        _ => {
            player::interact(world, interaction);
            RunState::PlayerTurn
        }
    }
}

fn get_idx_from_delta(world: &World, delta_x: i32, delta_y: i32) -> usize {
    let positions = world.read_storage::<Position>();
    let player_entity = world.fetch::<Entity>();
    let player_position = positions.get(*player_entity).unwrap();
    let floor = player_position.level;
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(floor).unwrap();
    let level_width = level.width as i32;
    level_utils::add_xy_to_idx(
        level_width as i32,
        delta_x as i32,
        delta_y as i32,
        player_position.idx as i32,
    ) as usize
}

pub fn handle_move(world: &mut World, delta_x: i32, delta_y: i32) -> RunState {
    let destination_idx = get_idx_from_delta(world, delta_x, delta_y);
    player::move_to_position(world, destination_idx);
    RunState::PlayerTurn
}

pub fn handle_act(world: &mut World, delta_x: i32, delta_y: i32) -> RunState {
    let destination_idx = get_idx_from_delta(world, delta_x, delta_y);
    let targets = get_interaction_targets_at_idx(world, destination_idx);
    if targets.len() == 0 {
        return RunState::AwaitingInput {
            offset_x: 0,
            offset_y: 0,
        };
    }
    RunState::InteractAtIdx {
        idx: destination_idx,
        interaction_menu_idx: 0,
        target_menu_idx: 0,
        interaction_menu_active: false,
    }
}

fn get_player_path_to_target(world: &World, target_idx: usize) -> NavigationPath {
    let player_entity = world.fetch::<Entity>();
    let positions = world.read_storage::<Position>();

    let player_position = positions.get(*player_entity).unwrap();
    let dungeon = world.fetch::<Dungeon>();
    let level = dungeon.get_level(player_position.level).unwrap();
    // this should be updated to only search tiles that the player has seen.
    a_star_search(player_position.idx as i32, target_idx as i32, level)
}

pub struct State {
    pub world: World,
    pub run_state: RunState,
    pub queued_action: Option<InteractionType>,
    pub settings: Settings,
    pub music: Music,
}

impl State {
    fn run_systems(&mut self, ctx: &mut Rltk) {
        let mut update_particles = UpdateParticleEffectsSystem {
            elapsed_time: ctx.frame_time_ms,
        };
        update_particles.run_now(&self.world);
        let mut remove_particles = RemoveParticleEffectsSystem {};
        remove_particles.run_now(&self.world);
        if self.run_state == RunState::PreRun
            || self.run_state == RunState::PlayerTurn
            || self.run_state == RunState::MonsterTurn
        {
            let mut equip_system = EquipSystem {};
            equip_system.run_now(&self.world);
            let mut light = LightSystem {};
            light.run_now(&self.world);
            let mut vis = VisibilitySystem {
                queued_action: &mut self.queued_action,
            };
            vis.run_now(&self.world);
            let mut update_memories_system = UpdateMemoriesSystem {};
            update_memories_system.run_now(&self.world);
        }
        if self.run_state == RunState::MonsterTurn {
            let mut mob = MonsterAI {};
            mob.run_now(&self.world);
        }
        let mut move_system = MoveSystem {};
        move_system.run_now(&self.world);
        let mut go_up_stairs_system = GoUpStairsSystem {};
        go_up_stairs_system.run_now(&self.world);
        let mut go_down_stairs_system = GoDownStairsSystem {};
        go_down_stairs_system.run_now(&self.world);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.world);
        let mut melee_combat = MeleeCombatSystem {};
        melee_combat.run_now(&self.world);
        let mut triggers = TriggerSystem {};
        triggers.run_now(&self.world);
        if self.run_state == RunState::MonsterTurn {
            let mut fire_burn_system = FireBurnSystem {};
            fire_burn_system.run_now(&self.world);
            let mut fire_spread_system = FireSpreadSystem {};
            fire_spread_system.run_now(&self.world);
            let mut fire_die_system = FireDieSystem {};
            fire_die_system.run_now(&self.world);
        }
        let mut damage = DamageSystem {
            queued_action: &mut self.queued_action,
        };
        damage.run_now(&self.world);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.world);
        let mut to_use = UseItemSystem {};
        to_use.run_now(&self.world);
        let mut drop = ItemDropSystem {};
        drop.run_now(&self.world);
        let mut remove_triggered_single_activation_traps_system = RemoveTriggeredTrapsSystem {};
        remove_triggered_single_activation_traps_system.run_now(&self.world);
        if self.run_state == RunState::PlayerTurn {
            let mut reveal_traps = RevealTrapsSystem {
                queued_action: &mut self.queued_action,
            };
            reveal_traps.run_now(&self.world);
        }
        let mut release_system = ReleaseSystem {};
        release_system.run_now(&self.world);
        if self.run_state == RunState::PlayerTurn || self.run_state == RunState::MonsterTurn {
            let mut search_for_hidden_system = SearchForHiddenSystem {};
            search_for_hidden_system.run_now(&self.world);
            let mut set_trap_system = SetTrapSystem {};
            set_trap_system.run_now(&self.world);
            let mut disarm_trap_system = DisarmTrapSystem {};
            disarm_trap_system.run_now(&self.world);
            let mut grab_system = GrabSystem {};
            grab_system.run_now(&self.world);
            let mut open_door_system = OpenDoorSystem {};
            open_door_system.run_now(&self.world);
            let mut close_door_system = CloseDoorSystem {};
            close_door_system.run_now(&self.world);
            let mut hide_system = HideSystem {};
            hide_system.run_now(&self.world);
            let mut light_item_system = LightItemSystem {};
            light_item_system.run_now(&self.world);
            let mut douse_item_system = DouseItemSystem {};
            douse_item_system.run_now(&self.world);
        }
        let mut blood_spawn_system = BloodSpawnSystem {};
        blood_spawn_system.run_now(&self.world);
        let mut particle_spawn_system = ParticleSpawnSystem {};
        particle_spawn_system.run_now(&self.world);
        let mut trap_spawn_system = TrapSpawnSystem {};
        trap_spawn_system.run_now(&self.world);
        let mut item_spawn_system = ItemSpawnSystem {};
        item_spawn_system.run_now(&self.world);
        let mut debris_spawn_system = DebrisSpawnSystem {};
        debris_spawn_system.run_now(&self.world);
        let mut corpse_spawn_system = CorpseSpawnSystem {};
        corpse_spawn_system.run_now(&self.world);
        if self.run_state == RunState::PlayerTurn || self.run_state == RunState::MonsterTurn {
            DamageSystem::delete_the_dead(&mut self.world);
            let mut memory_cull_system = MemoryCullSystem {};
            memory_cull_system.run_now(&self.world);
        }
        self.world.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        patches::patch_mod_keys(ctx);
        match self.run_state {
            RunState::PreRun
            | RunState::AwaitingInput { .. }
            | RunState::PlayerTurn
            | RunState::MonsterTurn => self.run_systems(ctx),
            _ => (),
        }
        self.run_state = match &mut self.run_state {
            RunState::PreRun => {
                ScreenMapGeneric::new(0, 0).draw(ctx, &mut self.world);
                RunState::AwaitingInput {
                    offset_x: 0,
                    offset_y: 0,
                }
            }
            RunState::AwaitingInput { offset_x, offset_y } => {
                ScreenMapGeneric::new(*offset_x, *offset_y).draw(ctx, &mut self.world);
                let action = self.settings.control_scheme.map.get_value_with_context(ctx);

                if action.is_some() {
                    self.queued_action = None
                }
                match action {
                    Some(action) => match action {
                        #[cfg(debug_assertions)]
                        MapAction::ShowDebugMenu => RunState::DebugMenu { highlighted: 0 },
                        MapAction::Exit => {
                            self.music.pause_music();
                            RunState::SavingScreen { count_down: 15 }
                        }
                        MapAction::ShowInventoryMenu => RunState::InventoryMenu { highlighted: 0 },
                        MapAction::ShowDropMenu => RunState::DropItemMenu { highlighted: 0 },
                        MapAction::ShowEquipmentMenu => RunState::EquipmentMenu {
                            highlighted: 0,
                            action_highlighted: 0,
                            action_menu: false,
                        },
                        MapAction::LeaveDungeon => {
                            match player_can_leave_dungeon(&mut self.world) {
                                true => {
                                    self.music.pause_music();
                                    RunState::ExitGameMenu { highlighted: 0 }
                                }
                                false => {
                                    let mut log = self.world.fetch_mut::<GameLog>();
                                    log.add(
                                        "You must first locate the exit to leave the dungeon"
                                            .to_string(),
                                    );
                                    RunState::AwaitingInput {
                                        offset_x: *offset_x,
                                        offset_y: *offset_y,
                                    }
                                }
                            }
                        }
                        MapAction::ShowActionMenu => RunState::ActionMenu { highlighted: 0 },

                        MapAction::SearchContainer => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<Container>(&self.world),
                            intent: TargetIntent::OpenContainer,
                            cta: Some(copy::CTA_INTERACT_OPEN_CONTAINER),
                        },
                        MapAction::GrabFurniture => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<Grabbable>(&self.world),
                            intent: TargetIntent::Grab,
                            cta: Some(copy::CTA_INTERACT_GRAB),
                        },
                        MapAction::DisarmTrap => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<Disarmable>(&self.world),
                            intent: TargetIntent::Disarm,
                            cta: Some(copy::CTA_INTERACT_DISARM),
                        },
                        MapAction::ArmTrap => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<Armable>(&self.world),
                            intent: TargetIntent::Arm,
                            cta: Some(copy::CTA_INTERACT_ARM),
                        },
                        MapAction::Hide => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<HidingSpot>(&self.world),
                            intent: TargetIntent::HideIn,
                            cta: Some(copy::CTA_INTERACT_HIDE),
                        },
                        MapAction::PickupItem => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<Item>(&self.world),
                            intent: TargetIntent::Pickup,
                            cta: Some(copy::CTA_INTERACT_PICKUP),
                        },
                        MapAction::OpenDoor => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_openable_doors(
                                &self.world,
                                get_visible_entities(&self.world),
                            ),
                            intent: TargetIntent::OpenDoor,
                            cta: Some(copy::CTA_INTERACT_OPEN_DOOR),
                        },
                        MapAction::Attack => RunState::InteractionTypeEntityTargeting {
                            target_idx: 0,
                            targets: get_interaction_type_targets::<CombatStats>(&self.world),
                            intent: TargetIntent::Attack,
                            cta: Some(copy::CTA_INTERACT_ATTACK),
                        },
                        MapAction::ScrollDown => RunState::AwaitingInput {
                            offset_x: *offset_x,
                            offset_y: *offset_y + 1,
                        },
                        MapAction::ScrollUp => RunState::AwaitingInput {
                            offset_x: *offset_x,
                            offset_y: *offset_y - 1,
                        },
                        MapAction::ScrollLeft => RunState::AwaitingInput {
                            offset_x: *offset_x - 1,
                            offset_y: *offset_y,
                        },
                        MapAction::ScrollRight => RunState::AwaitingInput {
                            offset_x: *offset_x + 1,
                            offset_y: *offset_y,
                        },
                        MapAction::Interact => {
                            RunState::InteractiveEntityTargeting { target_idx: 0 }
                        }
                        MapAction::MoveLeft => handle_move(&mut self.world, -1, 0),
                        MapAction::MoveRight => handle_move(&mut self.world, 1, 0),
                        MapAction::MoveUp => handle_move(&mut self.world, 0, -1),
                        MapAction::MoveDown => handle_move(&mut self.world, 0, 1),
                        MapAction::MoveUpLeft => handle_move(&mut self.world, -1, -1),
                        MapAction::MoveUpRight => handle_move(&mut self.world, 1, -1),
                        MapAction::MoveDownLeft => handle_move(&mut self.world, -1, 1),
                        MapAction::MoveDownRight => handle_move(&mut self.world, 1, 1),
                        MapAction::ActLeft => handle_act(&mut self.world, -1, 0),
                        MapAction::ActRight => handle_act(&mut self.world, 1, 0),
                        MapAction::ActUp => handle_act(&mut self.world, 0, -1),
                        MapAction::ActDown => handle_act(&mut self.world, 0, 1),
                        MapAction::ActUpLeft => handle_act(&mut self.world, -1, -1),
                        MapAction::ActUpRight => handle_act(&mut self.world, 1, -1),
                        MapAction::ActDownLeft => handle_act(&mut self.world, -1, 1),
                        MapAction::ActDownRight => handle_act(&mut self.world, 1, 1),
                        MapAction::AutoActLeft => handle_auto_act(&mut self.world, -1, 0),
                        MapAction::AutoActRight => handle_auto_act(&mut self.world, 1, 0),
                        MapAction::AutoActUp => handle_auto_act(&mut self.world, 0, -1),
                        MapAction::AutoActDown => handle_auto_act(&mut self.world, 0, 1),
                        MapAction::AutoActUpLeft => handle_auto_act(&mut self.world, -1, -1),
                        MapAction::AutoActUpRight => handle_auto_act(&mut self.world, 1, -1),
                        MapAction::AutoActDownLeft => handle_auto_act(&mut self.world, -1, 1),
                        MapAction::AutoActDownRight => handle_auto_act(&mut self.world, 1, 1),
                        MapAction::StayStill => RunState::PlayerTurn,
                        MapAction::SearchHidden => {
                            player::search_hidden(&mut self.world);
                            RunState::PlayerTurn
                        }
                        MapAction::ReleaseFurniture => {
                            player::release_entity(&mut self.world);
                            RunState::PlayerTurn
                        }
                    },
                    None => {
                        let mut next_state = RunState::AwaitingInput {
                            offset_x: *offset_x,
                            offset_y: *offset_y,
                        };
                        if let Some(interaction) = self.queued_action {
                            let interaction_idx = match interaction {
                                InteractionType::Douse(e)
                                | InteractionType::Light(e)
                                | InteractionType::HideIn(e)
                                | InteractionType::Attack(e)
                                | InteractionType::Grab(e)
                                | InteractionType::Disarm(e)
                                | InteractionType::Arm(e)
                                | InteractionType::Pickup(e)
                                | InteractionType::OpenDoor(e)
                                | InteractionType::CloseDoor(e)
                                | InteractionType::OpenContainer(e) => {
                                    let positions = self.world.read_storage::<Position>();
                                    Some(positions.get(e).unwrap().idx)
                                }
                                InteractionType::GoUp(idx)
                                | InteractionType::GoDown(idx)
                                | InteractionType::Move(idx)
                                | InteractionType::Exit(idx) => Some(idx),
                                _ => None,
                            };

                            if let Some(interaction_idx) = interaction_idx {
                                let path =
                                    { get_player_path_to_target(&self.world, interaction_idx) };
                                if !path.success {
                                    self.queued_action = None
                                } else if path.steps.len() > 2 {
                                    let next_index = path.steps[1];
                                    player::move_to_position(&mut self.world, next_index);
                                    next_state = RunState::PlayerTurn;
                                } else {
                                    self.queued_action = None;
                                    match interaction {
                                        InteractionType::OpenContainer(target) => {
                                            next_state = RunState::OpenContainerMenu {
                                                highlighted: 0,
                                                container: target,
                                            }
                                        }
                                        _ => {
                                            player::interact(&mut self.world, interaction);
                                            next_state = RunState::PlayerTurn;
                                        }
                                    }
                                }
                            } else {
                                self.queued_action = None
                            }
                        }
                        next_state
                    }
                }
            }
            RunState::PlayerTurn => {
                ScreenMapGeneric::new(0, 0).draw(ctx, &mut self.world);
                RunState::MonsterTurn
            }
            RunState::MonsterTurn => {
                ScreenMapGeneric::new(0, 0).draw(ctx, &mut self.world);
                let combat_stats = self.world.read_storage::<CombatStats>();
                let player_ent = self.world.fetch::<Entity>();
                let player_stats = combat_stats.get(*player_ent).unwrap();
                match player_stats.hp < 1 {
                    true => {
                        persistence::delete_save();
                        RunState::DeathScreen
                    }
                    _ => RunState::AwaitingInput {
                        offset_x: 0,
                        offset_y: 0,
                    },
                }
            }
            RunState::InventoryMenu { highlighted } => {
                let inventory = inventory::get_player_inventory_list(&mut self.world);
                let (inventory_entities, inventory_names): (Vec<_>, Vec<_>) =
                    inventory.into_iter().unzip();
                let menu_options: Box<[MenuOption<&String>]> = inventory_names
                    .iter()
                    .enumerate()
                    .map(|(index, text)| {
                        let state = match *highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(text, state)
                    })
                    .collect();
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    &format!(
                        "Use Item  < {}/{} >",
                        menu.page_number_at_index(*highlighted) + 1,
                        menu.page_count() + 1
                    ),
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::MoveHighlightNext => RunState::InventoryMenu {
                            highlighted: menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::InventoryMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                        },
                        MenuAction::NextPage => RunState::InventoryMenu {
                            highlighted: menu.get_next_page_index(*highlighted),
                        },
                        MenuAction::PreviousPage => RunState::InventoryMenu {
                            highlighted: menu.get_previous_page_index(*highlighted),
                        },
                        MenuAction::Select => match inventory_entities.get(*highlighted) {
                            Some(ent) => {
                                let is_ranged = {
                                    let ranged = self.world.read_storage::<Ranged>();
                                    match ranged.get(*ent) {
                                        Some(ranged_props) => Some(ranged_props.range),
                                        _ => None,
                                    }
                                };
                                match is_ranged {
                                    Some(range) => RunState::ItemUseTargeting { range, item: *ent },
                                    None => {
                                        player::use_item(&mut self.world, *ent, None);
                                        RunState::PlayerTurn
                                    }
                                }
                            }
                            None => RunState::InventoryMenu {
                                highlighted: *highlighted,
                            },
                        },
                        _ => RunState::InventoryMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::InventoryMenu {
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::DropItemMenu { highlighted } => {
                let inventory = inventory::get_player_inventory_list(&mut self.world);
                let (inventory_entities, inventory_names): (Vec<_>, Vec<_>) =
                    inventory.into_iter().unzip();
                let menu_options: Box<[MenuOption<&String>]> = inventory_names
                    .iter()
                    .enumerate()
                    .map(|(index, text)| {
                        let state = match *highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(text, state)
                    })
                    .collect();
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    &format!(
                        "Drop Item  < {}/{} >",
                        menu.page_number_at_index(*highlighted) + 1,
                        menu.page_count() + 1
                    ),
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::MoveHighlightNext => RunState::DropItemMenu {
                            highlighted: menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::DropItemMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                        },
                        MenuAction::NextPage => RunState::DropItemMenu {
                            highlighted: menu.get_next_page_index(*highlighted),
                        },
                        MenuAction::PreviousPage => RunState::DropItemMenu {
                            highlighted: menu.get_previous_page_index(*highlighted),
                        },
                        MenuAction::Select => match inventory_entities.get(*highlighted) {
                            Some(ent) => {
                                let mut intent = self.world.write_storage::<WantsToDropItem>();
                                intent
                                    .insert(
                                        *self.world.fetch::<Entity>(),
                                        WantsToDropItem { item: *ent },
                                    )
                                    .expect("Unable To Insert Drop Item Intent");
                                RunState::PlayerTurn
                            }
                            None => RunState::DropItemMenu {
                                highlighted: *highlighted,
                            },
                        },
                        _ => RunState::DropItemMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::DropItemMenu {
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::ExitGameMenu { highlighted } => {
                let menu_options: Box<[MenuOption<&str>]> =
                    ["Yes, exit the dungeon", "No, remain in the dungeon"]
                        .iter()
                        .enumerate()
                        .map(|(index, text)| {
                            let state = match *highlighted == index {
                                true => MenuOptionState::Highlighted,
                                false => MenuOptionState::Normal,
                            };
                            MenuOption::new(*text, state)
                        })
                        .collect();
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(menu.get_page(0), "Exit Dungeon?", "Escape to Cancel")
                    .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::MoveHighlightNext => RunState::ExitGameMenu {
                            highlighted: menu.get_next_page_index(*highlighted),
                        },

                        MenuAction::MoveHighlightPrev => RunState::ExitGameMenu {
                            highlighted: menu.get_previous_page_index(*highlighted),
                        },
                        MenuAction::Select => match highlighted {
                            0 => {
                                persistence::delete_save();
                                match has_objective_in_backpack(&self.world) {
                                    true => RunState::SuccessScreen,
                                    false => RunState::FailureScreen,
                                }
                            }
                            _ => RunState::AwaitingInput {
                                offset_x: 0,
                                offset_y: 0,
                            },
                        },
                        _ => RunState::ExitGameMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::ExitGameMenu {
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::ItemUseTargeting { range, item } => {
                let visible_tiles = ranged::get_visible_tiles_in_range(&self.world, *range);
                let target = ranged::get_target(&self.world, ctx, &visible_tiles);
                ScreenMapTargeting::new(*range, target, Some("Select Target".to_string()))
                    .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .targeting
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        TargetingAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        TargetingAction::Selected => match target {
                            Some(idx) => {
                                player::use_item(&mut self.world, *item, Some(idx));
                                RunState::PlayerTurn
                            }
                            None => RunState::AwaitingInput {
                                offset_x: 0,
                                offset_y: 0,
                            },
                        },
                    },
                    None => RunState::ItemUseTargeting {
                        range: *range,
                        item: *item,
                    },
                }
            }
            RunState::InteractAtIdx {
                idx,
                target_menu_idx,
                interaction_menu_idx,
                interaction_menu_active,
            } => {
                let targets = get_interaction_targets_at_idx(&self.world, *idx);
                let target_menu = get_menu_from_ents(&self.world, &targets, *target_menu_idx);
                let target = targets[*target_menu_idx];
                let options: Vec<InteractionType> =
                    get_interaction_options_for_target(&self.world, target);
                let interaction_menu: Menu<&str> =
                    get_menu_from_interaction_options(*interaction_menu_idx, &options);
                let interaction_description = options[*interaction_menu_idx].descriptive_text();
                ScreenMapNestedMenu::new(
                    target_menu.get_page_at_index(*target_menu_idx),
                    interaction_menu.get_page_at_index(*interaction_menu_idx),
                    *interaction_menu_active,
                    interaction_description,
                    "Interact",
                    "Use arrow keys to navigate menu, Enter to select interaction",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::MoveHighlightNext => RunState::InteractAtIdx {
                            idx: *idx,
                            target_menu_idx: match interaction_menu_active {
                                true => *target_menu_idx,
                                false => target_menu.get_next_index(*target_menu_idx),
                            },
                            interaction_menu_idx: match interaction_menu_active {
                                true => interaction_menu.get_next_index(*interaction_menu_idx),
                                false => *interaction_menu_idx,
                            },
                            interaction_menu_active: *interaction_menu_active,
                        },
                        MenuAction::MoveHighlightPrev => RunState::InteractAtIdx {
                            idx: *idx,
                            target_menu_idx: match interaction_menu_active {
                                true => *target_menu_idx,
                                false => target_menu.get_previous_index(*target_menu_idx),
                            },
                            interaction_menu_idx: match interaction_menu_active {
                                true => interaction_menu.get_next_index(*interaction_menu_idx),
                                false => *interaction_menu_idx,
                            },
                            interaction_menu_active: *interaction_menu_active,
                        },
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::Select => match options[*interaction_menu_idx] {
                            InteractionType::OpenContainer(container) => {
                                RunState::OpenContainerMenu {
                                    highlighted: 0,
                                    container,
                                }
                            }
                            _ => {
                                player::interact(&mut self.world, options[*interaction_menu_idx]);
                                RunState::PlayerTurn
                            }
                        },
                        MenuAction::NextMenu | MenuAction::PreviousMenu => {
                            RunState::InteractAtIdx {
                                target_menu_idx: *target_menu_idx,
                                interaction_menu_idx: *interaction_menu_idx,
                                interaction_menu_active: !*interaction_menu_active,
                                idx: *idx,
                            }
                        }
                        _ => RunState::InteractAtIdx {
                            target_menu_idx: *target_menu_idx,
                            interaction_menu_idx: *interaction_menu_idx,
                            interaction_menu_active: *interaction_menu_active,
                            idx: *idx,
                        },
                    },
                    None => RunState::InteractAtIdx {
                        target_menu_idx: *target_menu_idx,
                        interaction_menu_idx: *interaction_menu_idx,
                        interaction_menu_active: *interaction_menu_active,
                        idx: *idx,
                    },
                }
            }
            RunState::InteractMenu {
                highlighted,
                target,
            } => {
                let options: Vec<InteractionType> =
                    get_interaction_options_for_target(&self.world, *target);
                let menu: Menu<&str> = get_menu_from_interaction_options(*highlighted, &options);
                let title = self
                    .world
                    .read_storage::<Name>()
                    .get(*target)
                    .unwrap()
                    .name
                    .clone();
                ScreenMapInteractMenu::new(
                    menu.get_page_at_index(*highlighted),
                    Some(&title),
                    Some(""),
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::InteractiveEntityTargeting { target_idx: 0 },
                        MenuAction::Select => {
                            if let Some(interaction_type) = options.get(*highlighted) {
                                self.queued_action = Some(interaction_type.clone());
                            }
                            RunState::AwaitingInput {
                                offset_x: 0,
                                offset_y: 0,
                            }
                        }
                        MenuAction::MoveHighlightNext => RunState::InteractMenu {
                            highlighted: menu.get_next_index(*highlighted),
                            target: *target,
                        },
                        MenuAction::MoveHighlightPrev => RunState::InteractMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                            target: *target,
                        },
                        MenuAction::NextPage => RunState::InteractMenu {
                            highlighted: menu.get_next_page_index(*highlighted),
                            target: *target,
                        },
                        MenuAction::PreviousPage => RunState::InteractMenu {
                            highlighted: menu.get_previous_page_index(*highlighted),
                            target: *target,
                        },
                        _ => RunState::InteractMenu {
                            highlighted: *highlighted,
                            target: *target,
                        },
                    },
                    None => RunState::InteractMenu {
                        highlighted: *highlighted,
                        target: *target,
                    },
                }
            }
            RunState::InteractiveEntityTargeting { target_idx } => {
                let targets = get_interaction_targets(&self.world);
                let target_ent = targets.get(*target_idx);
                ScreenMapInteractTarget::new(target_ent, Some(copy::CTA_INTERACT))
                    .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .interaction
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        InteractionTargetingAction::Selected => match target_ent {
                            Some(target_ent) => RunState::InteractMenu {
                                highlighted: 0,
                                target: *target_ent,
                            },
                            None => RunState::AwaitingInput {
                                offset_x: 0,
                                offset_y: 0,
                            },
                        },
                        InteractionTargetingAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        InteractionTargetingAction::Next => RunState::InteractiveEntityTargeting {
                            target_idx: utils::select_next_idx(*target_idx, targets.len()),
                        },
                        InteractionTargetingAction::Previous => {
                            RunState::InteractiveEntityTargeting {
                                target_idx: utils::select_previous_idx(*target_idx, targets.len()),
                            }
                        }
                    },
                    None => RunState::InteractiveEntityTargeting {
                        target_idx: *target_idx,
                    },
                }
            }
            RunState::InteractionTypeEntityTargeting {
                target_idx,
                targets,
                intent,
                cta,
            } => {
                let target_ent = targets.get(*target_idx);
                ScreenMapInteractTarget::new(target_ent, *cta).draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .interaction
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        InteractionTargetingAction::Selected => {
                            if let Some(target_ent) = target_ent {
                                self.queued_action =
                                    Some(intent_to_interaction_type(*intent, *target_ent));
                            }
                            RunState::AwaitingInput {
                                offset_x: 0,
                                offset_y: 0,
                            }
                        }
                        InteractionTargetingAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        InteractionTargetingAction::Next => {
                            RunState::InteractionTypeEntityTargeting {
                                target_idx: utils::select_next_idx(*target_idx, targets.len()),
                                targets: targets.clone(),
                                intent: *intent,
                                cta: *cta,
                            }
                        }
                        InteractionTargetingAction::Previous => {
                            RunState::InteractionTypeEntityTargeting {
                                target_idx: utils::select_previous_idx(*target_idx, targets.len()),
                                targets: targets.clone(),
                                intent: *intent,
                                cta: *cta,
                            }
                        }
                    },
                    None => RunState::InteractionTypeEntityTargeting {
                        target_idx: *target_idx,
                        targets: targets.clone(),
                        intent: *intent,
                        cta: *cta,
                    },
                }
            }
            RunState::OpenContainerMenu {
                highlighted,
                container,
            } => {
                let inventory =
                    inventory::get_container_inventory_list(&mut self.world, &container);
                let (inventory_entities, inventory_names): (Vec<_>, Vec<_>) =
                    inventory.into_iter().unzip();
                let menu_options: Box<[MenuOption<&String>]> = inventory_names
                    .iter()
                    .enumerate()
                    .map(|(index, text)| {
                        let state = match *highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(text, state)
                    })
                    .collect();

                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    &format!(
                        "Take Item  < {}/{} >",
                        menu.page_number_at_index(*highlighted),
                        menu.page_count() + 1
                    ),
                    "A to take all. Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::MoveHighlightNext => RunState::OpenContainerMenu {
                            highlighted: menu.get_next_index(*highlighted),
                            container: *container,
                        },
                        MenuAction::MoveHighlightPrev => RunState::OpenContainerMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                            container: *container,
                        },
                        MenuAction::NextPage => RunState::OpenContainerMenu {
                            highlighted: menu.get_next_page_index(*highlighted),
                            container: *container,
                        },
                        MenuAction::PreviousPage => RunState::OpenContainerMenu {
                            highlighted: menu.get_previous_page_index(*highlighted),
                            container: *container,
                        },
                        MenuAction::Select => match inventory_entities.get(*highlighted) {
                            Some(ent) => {
                                player::pickup_item(&mut self.world, *ent, Some(*container));
                                RunState::PlayerTurn
                            }
                            None => RunState::OpenContainerMenu {
                                highlighted: *highlighted,
                                container: *container,
                            },
                        },
                        MenuAction::SelectAll => {
                            let items = HashSet::from_iter(inventory_entities);
                            player::pickup_items(&mut self.world, items, Some(*container));
                            RunState::PlayerTurn
                        }
                        _ => RunState::OpenContainerMenu {
                            highlighted: *highlighted,
                            container: *container,
                        },
                    },
                    None => RunState::OpenContainerMenu {
                        highlighted: *highlighted,
                        container: *container,
                    },
                }
            }
            RunState::ActionMenu { highlighted } => {
                let menu_options: Box<[MenuOption<&str>]> = [
                    "Use Item",
                    "Drop Item",
                    "Open Container",
                    "Search Area",
                    "Disarm Trap",
                    "Arm Trap",
                    "Grab Furniture",
                    "Release Furniture",
                    "Attack",
                    "Hide in Container",
                    "Douse",
                    "Light",
                    "Equipment Menu",
                ]
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    let state = match *highlighted == index {
                        true => MenuOptionState::Highlighted,
                        false => MenuOptionState::Normal,
                    };
                    MenuOption::new(*text, state)
                })
                .collect();
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    "Choose an action",
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::MoveHighlightNext => RunState::ActionMenu {
                            highlighted: menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::ActionMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                        },
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::Select => match highlighted {
                            0 => RunState::InventoryMenu { highlighted: 0 },
                            1 => RunState::DropItemMenu { highlighted: 0 },
                            2 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Container>(&self.world),
                                intent: TargetIntent::OpenContainer,
                                cta: Some(copy::CTA_INTERACT_OPEN_CONTAINER),
                            },
                            3 => {
                                player::search_hidden(&mut self.world);
                                RunState::PlayerTurn
                            }
                            4 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Disarmable>(&self.world),
                                intent: TargetIntent::Disarm,
                                cta: Some(copy::CTA_INTERACT_DISARM),
                            },
                            5 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Armable>(&self.world),
                                intent: TargetIntent::Disarm,
                                cta: Some(copy::CTA_INTERACT_DISARM),
                            },
                            6 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Grabbable>(&self.world),
                                intent: TargetIntent::Grab,
                                cta: Some(copy::CTA_INTERACT_GRAB),
                            },
                            7 => {
                                player::release_entity(&mut self.world);
                                RunState::AwaitingInput {
                                    offset_x: 0,
                                    offset_y: 0,
                                }
                            }
                            8 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<CombatStats>(&self.world),
                                intent: TargetIntent::Attack,
                                cta: Some(copy::CTA_INTERACT_ATTACK),
                            },
                            9 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<HidingSpot>(&self.world),
                                intent: TargetIntent::HideIn,
                                cta: Some(copy::CTA_INTERACT_HIDE),
                            },
                            10 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Dousable>(&self.world),
                                intent: TargetIntent::Douse,
                                cta: Some(copy::CTA_INTERACT_DOUSE),
                            },
                            11 => RunState::InteractionTypeEntityTargeting {
                                target_idx: 0,
                                targets: get_interaction_type_targets::<Lightable>(&self.world),
                                intent: TargetIntent::Light,
                                cta: Some(copy::CTA_INTERACT_LIGHT),
                            },
                            12 => RunState::EquipmentMenu {
                                highlighted: 0,
                                action_highlighted: 0,
                                action_menu: false,
                            },
                            _ => RunState::ActionMenu {
                                highlighted: *highlighted,
                            },
                        },
                        _ => RunState::ActionMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::ActionMenu {
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::EquipMenu {
                highlighted,
                position,
            } => {
                let equipment: Vec<(String, Option<Entity>)> = {
                    let player_ent = self.world.fetch::<Entity>();
                    let equipment = self.world.read_storage::<Equipable>();
                    let names = self.world.read_storage::<Name>();
                    let inventories = self.world.read_storage::<Inventory>();
                    let player_inventory = inventories.get(*player_ent).unwrap();
                    iter::once((String::from("Nothing"), None))
                        .chain(
                            player_inventory
                                .items
                                .iter()
                                .filter(|ent| {
                                    if let Some(equip) = equipment.get(**ent) {
                                        return equip
                                            .positions
                                            .iter()
                                            .find(|p| *p == position)
                                            .is_some();
                                    }
                                    false
                                })
                                .map(|ent| (names.get(*ent).unwrap().name.clone(), Some(*ent))),
                        )
                        .collect()
                };

                let menu_options = equipment
                    .iter()
                    .enumerate()
                    .map(|(index, (name, _))| {
                        let state = match *highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(name, state)
                    })
                    .collect();
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    "Choose an Item to Equip",
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::MoveHighlightNext => RunState::EquipMenu {
                            highlighted: menu.get_next_index(*highlighted),
                            position: *position,
                        },
                        MenuAction::MoveHighlightPrev => RunState::EquipMenu {
                            highlighted: menu.get_next_index(*highlighted),
                            position: *position,
                        },
                        MenuAction::Exit => RunState::EquipmentMenu {
                            highlighted: 0,
                            action_highlighted: 0,
                            action_menu: false,
                        },
                        MenuAction::Select => {
                            let (_, entity) = equipment[*highlighted];
                            player::equip_item(&mut self.world, entity, *position);
                            RunState::PlayerTurn
                        }
                        _ => RunState::EquipMenu {
                            highlighted: *highlighted,
                            position: *position,
                        },
                    },
                    None => RunState::EquipMenu {
                        highlighted: *highlighted,
                        position: *position,
                    },
                }
            }
            RunState::EquipmentMenu {
                highlighted,
                action_highlighted,
                action_menu,
            } => {
                let slots: Vec<(String, EquipmentPositions, Option<Entity>)> = {
                    let player_ent = self.world.fetch::<Entity>();
                    let equipment = self.world.read_storage::<Equipment>();
                    let player_equipment = equipment.get(*player_ent).unwrap();
                    let names = self.world.read_storage::<Name>();
                    let off_hand_item = match player_equipment.off_hand {
                        Some(e) => (names.get(e).unwrap().name.clone(), Some(e)),
                        None => (String::from("Empty"), None),
                    };
                    let dominant_hand_item = match player_equipment.dominant_hand {
                        Some(e) => (names.get(e).unwrap().name.clone(), Some(e)),
                        None => (String::from("Empty"), None),
                    };
                    vec![
                        (
                            format!("Dominant Hand: {}", dominant_hand_item.0),
                            EquipmentPositions::DominantHand,
                            dominant_hand_item.1,
                        ),
                        (
                            format!("Off Hand:  {}", off_hand_item.0),
                            EquipmentPositions::OffHand,
                            off_hand_item.1,
                        ),
                    ]
                };

                let mut submenu_actions = vec![];
                if let Some((_name, position, ent)) = slots.get(*highlighted) {
                    submenu_actions.push(("Exchange", EquipMenuType::Exchange(*position)));
                    if let Some(equipment_ent) = ent {
                        if self
                            .world
                            .read_storage::<Lightable>()
                            .get(*equipment_ent)
                            .is_some()
                        {
                            submenu_actions.push(("Light", EquipMenuType::Light(*equipment_ent)))
                        }
                        if self
                            .world
                            .read_storage::<Dousable>()
                            .get(*equipment_ent)
                            .is_some()
                        {
                            submenu_actions.push(("Douse", EquipMenuType::Douse(*equipment_ent)))
                        }
                    }
                }
                let description = {
                    match slots.get(*highlighted) {
                        Some((_position, _, e)) => match e {
                            Some(ent) => {
                                let info = self.world.read_storage::<Info>();
                                let ent_info = info.get(*ent);
                                match ent_info {
                                    Some(i) => String::from(&i.description),
                                    None => String::from("No Description"),
                                }
                            }
                            None => String::from("No Description"),
                        },
                        None => String::from("No Description"),
                    }
                };
                let submenu_options = submenu_actions
                    .iter()
                    .enumerate()
                    .map(|(index, (text, _))| {
                        let state = match *action_highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(text, state)
                    })
                    .collect();

                let menu_options = slots
                    .iter()
                    .enumerate()
                    .map(|(index, (text, _position, _))| {
                        let state = match *highlighted == index {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        };
                        MenuOption::new(text, state)
                    })
                    .collect();
                let menu = Menu::new(menu_options, 10);
                let submenu = Menu::new(submenu_options, 10);
                ScreenMapNestedMenu::new(
                    menu.get_page_at_index(*highlighted),
                    submenu.get_page_at_index(*action_highlighted),
                    *action_menu,
                    &description,
                    "Equipment",
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::MoveHighlightNext => RunState::EquipmentMenu {
                            highlighted: match action_menu {
                                true => *highlighted,
                                false => menu.get_next_index(*highlighted),
                            },
                            action_highlighted: match action_menu {
                                true => submenu.get_next_index(*action_highlighted),
                                false => *action_highlighted,
                            },
                            action_menu: *action_menu,
                        },
                        MenuAction::MoveHighlightPrev => RunState::EquipmentMenu {
                            highlighted: match action_menu {
                                true => *highlighted,
                                false => menu.get_previous_index(*highlighted),
                            },
                            action_highlighted: match action_menu {
                                true => submenu.get_next_index(*action_highlighted),
                                false => *action_highlighted,
                            },
                            action_menu: *action_menu,
                        },
                        MenuAction::Exit => RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        },
                        MenuAction::Select => match submenu_actions[*action_highlighted].1 {
                            EquipMenuType::Exchange(position) => RunState::EquipMenu {
                                highlighted: 0,
                                position,
                            },
                            EquipMenuType::Douse(ent) => {
                                player::douse_item(&mut self.world, ent);
                                RunState::PlayerTurn
                            }
                            EquipMenuType::Light(ent) => {
                                player::light_item(&mut self.world, ent);
                                RunState::PlayerTurn
                            }
                        },
                        MenuAction::NextMenu | MenuAction::PreviousMenu => {
                            RunState::EquipmentMenu {
                                highlighted: *highlighted,
                                action_highlighted: *action_highlighted,
                                action_menu: !*action_menu,
                            }
                        }
                        _ => RunState::EquipmentMenu {
                            highlighted: *highlighted,
                            action_highlighted: *action_highlighted,
                            action_menu: *action_menu,
                        },
                    },
                    None => RunState::EquipmentMenu {
                        highlighted: *highlighted,
                        action_highlighted: *action_highlighted,
                        action_menu: *action_menu,
                    },
                }
            }
            RunState::IntroScreen => {
                ScreenIntro::new().draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .static_screen
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        StaticAction::Exit => RunState::MainMenu { highlighted: 0 },
                        StaticAction::Continue => {
                            self.music.play_music();
                            RunState::PreRun
                        }
                    },
                    None => RunState::IntroScreen,
                }
            }
            RunState::DeathScreen => {
                ScreenDeath::new().draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .static_screen
                    .get_value_with_context(ctx)
                {
                    Some(_action) => RunState::MainMenu { highlighted: 0 },
                    None => RunState::DeathScreen,
                }
            }
            RunState::SuccessScreen => {
                ScreenSuccess::new().draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .static_screen
                    .get_value_with_context(ctx)
                {
                    Some(_action) => RunState::CreditsScreen,
                    None => RunState::SuccessScreen,
                }
            }
            RunState::FailureScreen => {
                ScreenFailure::new().draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .static_screen
                    .get_value_with_context(ctx)
                {
                    Some(_action) => RunState::MainMenu { highlighted: 0 },
                    None => RunState::FailureScreen,
                }
            }
            RunState::CreditsScreen => {
                ScreenCredits::new().draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .static_screen
                    .get_value_with_context(ctx)
                {
                    Some(_action) => RunState::MainMenu { highlighted: 0 },
                    None => RunState::CreditsScreen,
                }
            }
            RunState::SavingScreen { count_down } => {
                ScreenSaving::new().draw(ctx);
                match *count_down > 0 {
                    true => RunState::SavingScreen {
                        count_down: *count_down - 1,
                    },
                    _ => {
                        persistence::save_game(&mut self.world);
                        RunState::MainMenu { highlighted: 0 }
                    }
                }
            }
            RunState::LoadingScreen { count_down } => {
                ScreenLoading::new().draw(ctx);
                match *count_down > 0 {
                    true => RunState::LoadingScreen {
                        count_down: *count_down - 1,
                    },
                    _ => {
                        persistence::load_game(&mut self.world);
                        persistence::delete_save();
                        self.music.play_music();
                        RunState::AwaitingInput {
                            offset_x: 0,
                            offset_y: 0,
                        }
                    }
                }
            }
            RunState::NewGameScreen { count_down } => {
                ScreenNewGame::new().draw(ctx);
                match *count_down > 0 {
                    true => RunState::NewGameScreen {
                        count_down: *count_down - 1,
                    },
                    false => {
                        initialize_new_game(&mut self.world);
                        RunState::IntroScreen
                    }
                }
            }
            RunState::SetKey {
                action,
                highlighted,
            } => {
                // add screen that prompts for key input
                ScreenSetKey::new(action).draw(ctx);
                match ctx.key {
                    Some(key) => match key {
                        rltk::VirtualKeyCode::Escape => RunState::OptionsScreen {
                            highlighted: *highlighted,
                        },
                        rltk::VirtualKeyCode::LShift
                        | rltk::VirtualKeyCode::RShift
                        | rltk::VirtualKeyCode::LControl
                        | rltk::VirtualKeyCode::RControl => RunState::SetKey {
                            action: *action,
                            highlighted: *highlighted,
                        },
                        _ => {
                            //set the key
                            self.settings.control_scheme.map.remove_by_value(action);
                            self.settings
                                .control_scheme
                                .map
                                .insert_with_context(ctx, *action);
                            RunState::OptionsScreen {
                                highlighted: *highlighted,
                            }
                        }
                    },
                    None => RunState::SetKey {
                        action: *action,
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::OptionsScreen { highlighted } => {
                let map_controls = &self.settings.control_scheme.map;
                let menu_option_text: Box<[String]> = MapAction::actions()
                    .iter()
                    .map(|map_action| {
                        let control_text = match map_controls.get_control_for_value(map_action) {
                            Some(c) => c.to_string(),
                            None => String::from("None"),
                        };
                        let action_text = map_action.to_string();
                        let space_count = 50 - (control_text.len() + action_text.len());
                        let space_text = match space_count > 0 {
                            true => (0..space_count).map(|_| " ").collect(),
                            false => String::from(""),
                        };
                        format!("{}{}{}", action_text, space_text, control_text)
                    })
                    .collect();
                let menu_options = menu_option_text
                    .iter()
                    .enumerate()
                    .map(|(i, text)| {
                        MenuOption::new(
                            text,
                            match *highlighted == i {
                                true => MenuOptionState::Highlighted,
                                _ => MenuOptionState::Normal,
                            },
                        )
                    })
                    .collect();
                let controls_menu = Menu::new(menu_options, MapAction::actions().len());
                ScreenOptions::new(
                    "Options",
                    "Press Enter to set control, Delete to clear",
                    controls_menu.get_page_at_index(*highlighted),
                )
                .draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => {
                            self.settings.save();
                            RunState::MainMenu { highlighted: 0 }
                        }
                        MenuAction::Select => RunState::SetKey {
                            action: MapAction::actions()[*highlighted],
                            highlighted: *highlighted,
                        },
                        MenuAction::Delete => {
                            let highlighted_value = MapAction::actions()[*highlighted];
                            self.settings
                                .control_scheme
                                .map
                                .remove_by_value(&highlighted_value);
                            RunState::OptionsScreen {
                                highlighted: *highlighted,
                            }
                        }
                        MenuAction::MoveHighlightNext => RunState::OptionsScreen {
                            highlighted: controls_menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::OptionsScreen {
                            highlighted: controls_menu.get_previous_index(*highlighted),
                        },
                        _ => RunState::OptionsScreen {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::OptionsScreen {
                        highlighted: *highlighted,
                    },
                }
            }
            RunState::MainMenu { highlighted } => {
                let has_save_game = persistence::has_save_game();
                let new_game_state = match *highlighted == 0 {
                    true => MenuOptionState::Highlighted,
                    false => MenuOptionState::Normal,
                };
                let continue_state = match has_save_game {
                    true => match *highlighted == 1 {
                        true => MenuOptionState::Highlighted,
                        false => MenuOptionState::Normal,
                    },
                    false => MenuOptionState::Disabled,
                };
                let options_state = match *highlighted == 2 {
                    true => MenuOptionState::Highlighted,
                    false => MenuOptionState::Normal,
                };
                let credits_state = match *highlighted == 3 {
                    true => MenuOptionState::Highlighted,
                    false => MenuOptionState::Normal,
                };
                let menu = if cfg!(target_arch = "wasm32") {
                    Menu::new(
                        Box::new([
                            MenuOption::new("New Game", new_game_state),
                            MenuOption::new("Continue", continue_state),
                            MenuOption::new("Options", options_state),
                            MenuOption::new("Credits", credits_state),
                        ]),
                        10,
                    )
                } else {
                    let quit_state = match *highlighted == 4 {
                        true => MenuOptionState::Highlighted,
                        false => MenuOptionState::Normal,
                    };
                    Menu::new(
                        Box::new([
                            MenuOption::new("New Game", new_game_state),
                            MenuOption::new("Continue", continue_state),
                            MenuOption::new("Options", options_state),
                            MenuOption::new("Credits", credits_state),
                            MenuOption::new("Quit", quit_state),
                        ]),
                        10,
                    )
                };

                ScreenMainMenu::new(menu.get_page_at_index(*highlighted)).draw(ctx);
                match self
                    .settings
                    .control_scheme
                    .horizontal_menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::MainMenu {
                            highlighted: *highlighted,
                        },
                        MenuAction::MoveHighlightNext => RunState::MainMenu {
                            highlighted: menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::MainMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                        },
                        MenuAction::Select => match *highlighted {
                            0 => RunState::NewGameScreen { count_down: 15 },
                            1 => RunState::LoadingScreen { count_down: 15 },
                            2 => RunState::OptionsScreen { highlighted: 0 },
                            3 => RunState::CreditsScreen,
                            4 => std::process::exit(0),
                            _ => RunState::MainMenu {
                                highlighted: *highlighted,
                            },
                        },
                        _ => RunState::MainMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::MainMenu {
                        highlighted: *highlighted,
                    },
                }
            }
            #[cfg(debug_assertions)]
            RunState::DebugMenu { highlighted } => {
                let menu_options = Box::new([
                    MenuOption::new(
                        "Wrath of God",
                        match *highlighted == 0 {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        },
                    ),
                    MenuOption::new(
                        "Gitaxian Probe",
                        match *highlighted == 1 {
                            true => MenuOptionState::Highlighted,
                            false => MenuOptionState::Normal,
                        },
                    ),
                ]);
                let menu = Menu::new(menu_options, 10);
                ScreenMapMenu::new(
                    menu.get_page_at_index(*highlighted),
                    "Debug Menu",
                    "Escape to Cancel",
                )
                .draw(ctx, &mut self.world);
                match self
                    .settings
                    .control_scheme
                    .menu
                    .get_value_with_context(ctx)
                {
                    Some(action) => match action {
                        MenuAction::Exit => RunState::DebugMenu {
                            highlighted: *highlighted,
                        },
                        MenuAction::MoveHighlightNext => RunState::DebugMenu {
                            highlighted: menu.get_next_index(*highlighted),
                        },
                        MenuAction::MoveHighlightPrev => RunState::DebugMenu {
                            highlighted: menu.get_previous_index(*highlighted),
                        },
                        MenuAction::Select => match highlighted {
                            0 => {
                                debug::kill_all_monsters(&mut self.world);
                                self.world
                                    .fetch_mut::<GameLog>()
                                    .entries
                                    .insert(0, "all monsters removed".to_owned());
                                RunState::AwaitingInput {
                                    offset_x: 0,
                                    offset_y: 0,
                                }
                            }
                            1 => {
                                debug::reveal_map(&mut self.world);
                                self.world
                                    .fetch_mut::<GameLog>()
                                    .entries
                                    .insert(0, "map revealed".to_owned());
                                RunState::AwaitingInput {
                                    offset_x: 0,
                                    offset_y: 0,
                                }
                            }
                            _ => RunState::DebugMenu {
                                highlighted: *highlighted,
                            },
                        },
                        _ => RunState::DebugMenu {
                            highlighted: *highlighted,
                        },
                    },
                    None => RunState::DebugMenu {
                        highlighted: *highlighted,
                    },
                }
            }
        };
    }
}
