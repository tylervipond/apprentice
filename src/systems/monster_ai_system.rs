use crate::components::{
    CombatStats, Paralyzed, Door, Furniture, Hiding, Memory, Monster, Position, Viewshed,
    WantsToMelee, WantsToMove, WantsToOpenDoor,
};
use crate::dungeon::{dungeon::Dungeon, level::Level, level_utils, tile_type::TileType};
use crate::{
    ai::{choose_action, reasoner, Action, WeightedAction},
    components::memory::MemoryLocation,
};
use rltk::{a_star_search, RandomNumberGenerator};
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

fn get_move_action(
    level: &Level,
    move_idx: usize,
    step_count: i32,
    furniture_storage: &ReadStorage<Furniture>,
    door_storage: &ReadStorage<Door>,
) -> WeightedAction {
    if level.tiles[move_idx as usize] == TileType::Door {
        for entity in level_utils::entities_at_idx(level, move_idx) {
            if door_storage.get(entity).is_some() {
                return WeightedAction::new(
                    Action::OpenDoor(entity),
                    reasoner::move_weight(step_count, 2.0),
                );
            }
        }
    } else if level.blocked[move_idx as usize] {
        for entity in level_utils::entities_at_idx(level, move_idx) {
            if furniture_storage.get(entity).is_some() {
                return WeightedAction::new(
                    Action::Attack(entity),
                    reasoner::move_weight(step_count, 3.0),
                );
            }
        }
    }

    WeightedAction::new(
        Action::MoveTo(move_idx),
        reasoner::move_weight(step_count, 2.0),
    )
}

fn get_next_step(level: &Level, start_idx: usize, end_idx: usize) -> Option<(usize, usize)> {
    let path = a_star_search(start_idx as i32, end_idx as i32, level);
    let step_count = path.steps.len();
    match path.success && step_count > 1 {
        true => Some((path.steps[1], step_count)),
        _ => None,
    }
}

fn get_move_action_from_path(
    level: &Level,
    start_idx: usize,
    end_idx: usize,
    furniture_storage: &ReadStorage<Furniture>,
    door_storage: &ReadStorage<Door>,
) -> Option<WeightedAction> {
    match get_next_step(level, start_idx, end_idx) {
        Some((next_step, step_count)) => Some(get_move_action(
            level,
            next_step,
            step_count as i32,
            furniture_storage,
            door_storage,
        )),
        _ => None,
    }
}

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Dungeon>,
        ReadExpect<'a, Entity>,
        Entities<'a>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Paralyzed>,
        WriteStorage<'a, WantsToMove>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, Memory>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteStorage<'a, WantsToOpenDoor>,
        ReadStorage<'a, Furniture>,
        ReadStorage<'a, Hiding>,
        ReadStorage<'a, Door>,
    );
    // This is currently very limited. Monsters will only act if they can see a player, which means that they must
    // also be on the same level to act.
    fn run(&mut self, data: Self::SystemData) {
        let (
            mut dungeon,
            player_entity,
            entities,
            viewsheds,
            positions,
            monsters,
            mut wants_to_melee,
            mut paralyzed,
            mut wants_to_move,
            combat_stats,
            mut memory,
            mut rng,
            mut wants_to_open_door,
            furniture,
            hiding,
            doors,
        ) = data;
        let (player_idx, player_level) = {
            let pos = positions.get(*player_entity).unwrap();
            (pos.idx, pos.level)
        };
        let level = dungeon.get_level_mut(player_level).unwrap();
        let player_hp = combat_stats.get(*player_entity).unwrap().hp;

        for (_monsters, entity, viewshed, position, memory) in
            (&monsters, &entities, &viewsheds, &positions, &mut memory).join()
        {
            if let Some(is_paralyzed) = paralyzed.get_mut(entity) {
                is_paralyzed.turns -= 1;
                if is_paralyzed.turns < 1 {
                    paralyzed.remove(entity);
                }
                continue;
            }
            if position.level != player_level {
                continue;
            }
            let mut weighted_actions = vec![];
            let current_idx = position.idx;
            let player_is_not_hiding = hiding.get(*player_entity).is_none();
            let distance = level_utils::get_distance_between_idxs(&level, position.idx, player_idx);
            if player_is_not_hiding && distance < 1.5 {
                weighted_actions.push(WeightedAction::new(
                    Action::Attack(*player_entity),
                    reasoner::attack_weight(player_hp),
                ));
            } else if player_is_not_hiding && viewshed.visible_tiles.contains(&player_idx) {
                if let Some((next_step, step_count)) =
                    get_next_step(&level, current_idx, player_idx)
                {
                    weighted_actions.push(WeightedAction::new(
                        Action::Chase(next_step),
                        reasoner::chase_weight(player_hp, step_count as i32),
                    ));
                }
            } else if !memory.known_enemy_hiding_spots.is_empty()
                || !memory.last_known_enemy_positions.is_empty()
            {
                for (_e, MemoryLocation(enemy_level, enemy_idx)) in
                    memory.last_known_enemy_positions.iter()
                {
                    if *enemy_level != position.level as i32 {
                        continue;
                    }
                    if let Some(action) = get_move_action_from_path(
                        &level,
                        current_idx,
                        *enemy_idx,
                        &furniture,
                        &doors,
                    ) {
                        weighted_actions.push(action);
                    }
                }
                for (_enemy, hiding_spot) in memory.known_enemy_hiding_spots.iter() {
                    if let Some(hiding_position) = positions.get(*hiding_spot) {
                        let hiding_idx = hiding_position.idx;
                        let distance = level_utils::get_distance_between_idxs(
                            &level,
                            position.idx,
                            hiding_idx,
                        );
                        if distance < 1.5 {
                            let hiding_place_hp = combat_stats.get(*hiding_spot).unwrap().hp;
                            weighted_actions.push(WeightedAction::new(
                                Action::Attack(*hiding_spot),
                                reasoner::attack_weight(hiding_place_hp),
                            ));
                        } else if viewshed.visible_tiles.contains(&hiding_idx) {
                            if let Some((next_step, step_count)) =
                                get_next_step(&level, current_idx, player_idx)
                            {
                                let hiding_place_hp = combat_stats.get(*hiding_spot).unwrap().hp;
                                weighted_actions.push(WeightedAction::new(
                                    Action::Chase(next_step),
                                    reasoner::chase_weight(hiding_place_hp, step_count as i32),
                                ));
                            }
                        }
                    }
                }
            } else {
                let destination_idx = match memory.wander_destination {
                    Some(dest) => Some(dest.1),
                    None => level_utils::get_random_unblocked_floor_point(&level, &mut rng),
                };
                if let Some(idx) = destination_idx {
                    memory.wander_destination = Some(MemoryLocation(position.level as i32, idx));
                    if let Some(action) =
                        get_move_action_from_path(&level, current_idx, idx, &furniture, &doors)
                    {
                        weighted_actions.push(action);
                    }
                }
            }
            match choose_action(weighted_actions) {
                Some(Action::Chase(idx)) => {
                    wants_to_move
                        .insert(entity, WantsToMove { idx })
                        .expect("couldn't insert move intent");
                }
                Some(Action::Attack(target)) => {
                    wants_to_melee
                        .insert(entity, WantsToMelee { target })
                        .expect("Unable to insert attack intent");
                }
                Some(Action::MoveTo(idx)) => {
                    wants_to_move
                        .insert(entity, WantsToMove { idx })
                        .expect("couldn't insert move intent");
                }
                Some(Action::OpenDoor(door)) => {
                    wants_to_open_door
                        .insert(entity, WantsToOpenDoor { door })
                        .expect("couldn't insert open door intent");
                }
                _ => {}
            };
        }
    }
}
