use crate::components::{
  area_of_effect::AreaOfEffect, combat_stats::CombatStats, confused::Confused,
  confusion::Confusion, consumable::Consumable, dungeon_level::DungeonLevel,
  inflicts_damage::InflictsDamage, name::Name, position::Position,
  provides_healing::ProvidesHealing, suffer_damage::SufferDamage, wants_to_use::WantsToUse,
};
use crate::dungeon::{
  dungeon::Dungeon,
  operations::{entities_at_xy, point_not_in_map},
};
use crate::game_log::GameLog;
use crate::services::particle_effect_spawner::ParticleEffectSpawner;
use rltk::{to_cp437, BLACK, MAGENTA, ORANGE, RED, RGB};
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
    ReadStorage<'a, InflictsDamage>,
    WriteStorage<'a, SufferDamage>,
    WriteExpect<'a, Dungeon>,
    ReadStorage<'a, AreaOfEffect>,
    ReadStorage<'a, Confusion>,
    WriteStorage<'a, Confused>,
    ReadStorage<'a, DungeonLevel>,
    WriteExpect<'a, ParticleEffectSpawner>,
    ReadStorage<'a, Position>,
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
      inflicts_damage,
      mut suffer_damage,
      mut dungeon,
      aoe,
      causes_confusion,
      mut is_confused,
      dungeon_levels,
      mut particle_spawner,
      positions,
    ) = data;
    let player_level = dungeon_levels.get(*player_entity).unwrap();
    let level = dungeon.get_level(player_level.level).unwrap();
    for (to_use, entity) in (&wants_to_use, &entities).join() {
      let targets = match to_use.target {
        None => vec![*player_entity],
        Some(target) => match aoe.get(to_use.item) {
          None => entities_at_xy(&level, target.x, target.y),
          Some(area) => rltk::field_of_view(target, area.radius, &*level)
            .iter()
            .filter(|p| !point_not_in_map(&level, p))
            .map(|p| entities_at_xy(&level, p.x, p.y))
            .flatten()
            .collect(),
        },
      };

      match to_use.target {
        None => {}
        Some(target) => match aoe.get(to_use.item) {
          None => {}
          Some(area) => {
            let dungeon_level = dungeon_levels.get(entity).unwrap();
            rltk::field_of_view(target, area.radius, &*level)
              .iter()
              .filter(|p| !point_not_in_map(&level, p))
              .for_each(|p| {
                particle_spawner.request(
                  p.x,
                  p.y,
                  RGB::named(ORANGE),
                  RGB::named(RED),
                  to_cp437('░'),
                  200.0,
                  dungeon_level.level,
                )
              })
          }
        },
      };

      let heals = provides_healing.get(to_use.item);
      let damages = inflicts_damage.get(to_use.item);
      let confuses = causes_confusion.get(to_use.item);
      for target in targets {
        let pos = positions.get(target).unwrap();
        let dungeon_level = dungeon_levels.get(target).unwrap();
        if let Some(heals) = heals {
          if let Some(stats) = combat_stats.get_mut(target) {
            stats.hp = i32::min(stats.max_hp, stats.hp + heals.amount);
          }
          particle_spawner.request(
            pos.x,
            pos.y,
            RGB::named(RED),
            RGB::named(BLACK),
            to_cp437('♥'),
            200.0,
            dungeon_level.level,
          );
          if entity == *player_entity {
            game_log.entries.insert(
              0,
              format!(
                "You use the {}, healing {} hp.",
                names.get(to_use.item).unwrap().name,
                heals.amount
              ),
            );
          }
        }
        if let Some(damages) = damages {
          if let Some(suffer_damage) = suffer_damage.get_mut_or_default(target) {
            suffer_damage.amount += damages.amount;
          }
          particle_spawner.request(
            pos.x,
            pos.y,
            RGB::named(RED),
            RGB::named(BLACK),
            to_cp437('‼'),
            200.0,
            dungeon_level.level,
          );
          if entity == *player_entity {
            if let Some(mob_name) = names.get(target) {
              let item_name = names.get(to_use.item).unwrap();
              game_log.entries.insert(
                0,
                format!(
                  "you use {} on {} causing {} damage",
                  item_name.name, mob_name.name, damages.amount
                ),
              )
            }
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
            pos.x,
            pos.y,
            RGB::named(MAGENTA),
            RGB::named(BLACK),
            to_cp437('?'),
            200.0,
            dungeon_level.level,
          );
          if entity == *player_entity {
            let mob_name = names.get(target).unwrap();
            let item_name = names.get(to_use.item).unwrap();
            game_log.entries.insert(
              0,
              format!(
                "you use {} on {}, confusing them.",
                item_name.name, mob_name.name,
              ),
            )
          }
          if target == *player_entity {
            let mob_name = names.get(entity).unwrap();
            let item_name = names.get(to_use.item).unwrap();
            game_log.entries.insert(
              0,
              format!(
                "{} uses {} on you, you are confused.",
                item_name.name, mob_name.name,
              ),
            )
          }
        }
      }

      if let Some(_) = consumables.get(to_use.item) {
        entities.delete(to_use.item).expect("Delete Failed");
      };
    }
    wants_to_use.clear();
  }
}
