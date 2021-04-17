use crate::components::{
    AreaOfEffect, CausesDamage, CausesFire, CausesLight, CombatStats, Confused, Confusion,
    Consumable, DamageHistory, Flammable, Inventory, Name, OnFire, Position, ProvidesHealing,
    SufferDamage, WantsToUse,
};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::services::{GameLog, ParticleEffectSpawner};
use rltk::{RandomNumberGenerator, BLACK, MAGENTA, ORANGE, RED, RGB};
use specs::{
    storage::GenericWriteStorage, Entities, Entity, Join, ReadExpect, ReadStorage, System,
    WriteExpect, WriteStorage,
};

pub struct UseItemSystem {}

impl<'a> System<'a> for UseItemSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUse>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, CausesDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadExpect<'a, Dungeon>,
        ReadStorage<'a, AreaOfEffect>,
        ReadStorage<'a, Confusion>,
        WriteStorage<'a, Confused>,
        WriteExpect<'a, ParticleEffectSpawner>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, CausesFire>,
        ReadStorage<'a, Flammable>,
        WriteStorage<'a, OnFire>,
        WriteStorage<'a, CausesLight>,
        WriteStorage<'a, DamageHistory>,
        WriteStorage<'a, Inventory>,
        WriteExpect<'a, RandomNumberGenerator>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut game_log,
            entities,
            mut wants_to_use,
            names,
            consumables,
            mut combat_stats,
            provides_healing,
            causes_damage,
            mut suffer_damage,
            dungeon,
            aoe,
            causes_confusion,
            mut is_confused,
            mut particle_spawner,
            positions,
            causes_fire,
            flammables,
            mut on_fire,
            mut causes_light,
            mut damage_histories,
            mut inventories,
            mut rng,
        ) = data;
        let player_position = positions.get(*player_entity).unwrap();
        let level = dungeon.get_level(player_position.level).unwrap();
        for (to_use, entity, inventory) in (&wants_to_use, &entities, &mut inventories).join() {
            let targets = match to_use.target {
                None => vec![*player_entity],
                Some(target) => match aoe.get(to_use.item) {
                    None => level_utils::entities_at_idx(&level, target),
                    Some(area) => {
                        level_utils::get_field_of_view_from_idx(&*level, target, area.radius)
                            .iter()
                            .filter(|idx| !level_utils::idx_not_in_map(&level, **idx))
                            .map(|idx| level_utils::entities_at_idx(&level, *idx))
                            .flatten()
                            .collect()
                    }
                },
            };

            match to_use.target {
                None => {}
                Some(target) => match aoe.get(to_use.item) {
                    None => {}
                    Some(area) => {
                        let position = positions.get(entity).unwrap();
                        level_utils::get_field_of_view_from_idx(&*level, target, area.radius)
                            .iter()
                            .filter(|idx| !level_utils::idx_not_in_map(&level, **idx))
                            .for_each(|idx| {
                                particle_spawner.request(
                                    *idx as usize,
                                    RGB::named(ORANGE),
                                    RGB::named(RED),
                                    rltk::to_cp437('░'),
                                    200.0,
                                    position.level,
                                )
                            })
                    }
                },
            };

            let heals = provides_healing.get(to_use.item);
            let damages = causes_damage.get(to_use.item);
            let confuses = causes_confusion.get(to_use.item);
            let burns = causes_fire.get(to_use.item);
            for target in targets {
                let pos = positions.get(target).unwrap();
                if burns.is_some() {
                    if let Some(f) = flammables.get(target) {
                        on_fire
                            .insert(target, OnFire {})
                            .expect("couldn't light target on fire");
                        causes_light
                            .insert(
                                target,
                                CausesLight {
                                    radius: 3,
                                    lit: true,
                                    turns_remaining: Some(f.turns_remaining as u32),
                                },
                            )
                            .expect("couldn't insert cause light for target");
                    }
                }
                if let Some(stats) = combat_stats.get_mut(target) {
                    if let Some(damages) = damages {
                        let damage = rng.range(damages.min, damages.max) + damages.bonus;
                        if let Some(suffer_damage) = suffer_damage.get_mut_or_default(target) {
                            suffer_damage.amount += damage;
                        }
                        if let Some(damage_history) = damage_histories.get_mut(target) {
                            let damage_type = rng.random_slice_entry(&damages.damage_type).unwrap();
                            damage_history.events.insert(*damage_type);
                        }
                        particle_spawner.request(
                            pos.idx,
                            RGB::named(RED),
                            RGB::named(BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                            pos.level,
                        );
                        let ent_is_player = entity == *player_entity;
                        if ent_is_player {
                            if let Some(mob_name) = names.get(target) {
                                let item_name = names.get(to_use.item).unwrap();
                                game_log.add(format!(
                                    "you use {} on {} causing {} damage",
                                    item_name.name, mob_name.name, damage
                                ));
                            }
                        }
                        if let Some(heals) = heals {
                            stats.hp = i32::min(stats.max_hp, stats.hp + heals.amount);
                            particle_spawner.request(
                                pos.idx,
                                RGB::named(RED),
                                RGB::named(BLACK),
                                rltk::to_cp437('♥'),
                                200.0,
                                pos.level,
                            );
                            if ent_is_player {
                                game_log.add(format!(
                                    "You use the {}, healing {} hp.",
                                    names.get(to_use.item).unwrap().name,
                                    heals.amount
                                ));
                            }
                        }

                        if let Some(confuses) = confuses {
                            is_confused
                                .insert(
                                    target,
                                    Confused {
                                        turns: confuses.turns,
                                    },
                                )
                                .expect("Failed to confuse target");
                            particle_spawner.request(
                                pos.idx,
                                RGB::named(MAGENTA),
                                RGB::named(BLACK),
                                rltk::to_cp437('?'),
                                200.0,
                                pos.level,
                            );
                            if ent_is_player {
                                let mob_name = names.get(target).unwrap();
                                let item_name = names.get(to_use.item).unwrap();
                                game_log.add(format!(
                                    "you use {} on {}, confusing them.",
                                    item_name.name, mob_name.name,
                                ));
                            }
                            if target == *player_entity {
                                let mob_name = names.get(entity).unwrap();
                                let item_name = names.get(to_use.item).unwrap();
                                game_log.add(format!(
                                    "{} uses {} on you, you are confused.",
                                    item_name.name, mob_name.name,
                                ));
                            }
                        }
                    }
                }
            }
            if let Some(_) = consumables.get(to_use.item) {
                inventory.items.remove(&to_use.item);
                entities.delete(to_use.item).expect("Delete Failed");
            };
        }
        wants_to_use.clear();
    }
}
