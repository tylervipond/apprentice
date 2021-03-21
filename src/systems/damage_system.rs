use crate::{
    components::{
        monster::MonsterSpecies, CombatStats, Container, DamageHistory, Equipment, Hiding,
        Inventory, Monster, Name, Player, Position, Renderable, SufferDamage, Viewshed,
    },
    interaction_type::InteractionType,
    services::{BloodSpawner, CorpseSpawner, DebrisSpawner, GameLog},
};
use rltk::RGB;
use specs::{
    Entities, Entity, Join, ReadExpect, ReadStorage, System, World, WorldExt, WriteExpect,
    WriteStorage,
};

pub struct DamageSystem<'a> {
    pub queued_action: &'a mut Option<InteractionType>,
}

impl<'a> DamageSystem<'a> {
    pub fn delete_the_dead(ecs: &mut World) {
        let mut dead: Vec<Entity> = Vec::new();
        {
            let combat_stats = ecs.read_storage::<CombatStats>();
            let monsters = ecs.read_storage::<Monster>();
            let renderables = ecs.read_storage::<Renderable>();
            let mut positions = ecs.write_storage::<Position>();
            let names = ecs.read_storage::<Name>();
            let containers = ecs.read_storage::<Container>();
            let entities = ecs.entities();
            let mut log = ecs.write_resource::<GameLog>();
            let mut debris_spawner = ecs.write_resource::<DebrisSpawner>();
            let mut corpse_spawner = ecs.write_resource::<CorpseSpawner>();
            let inventory = ecs.read_storage::<Inventory>();
            let equipment = ecs.read_storage::<Equipment>();
            let player_entity = ecs.fetch::<Entity>();
            let damage_histories = ecs.read_storage::<DamageHistory>();
            let viewsheds = ecs.read_storage::<Viewshed>();
            let player_viewshed = viewsheds.get(*player_entity).unwrap();

            for (entity, _stats, name, renderable) in
                (&entities, &combat_stats, &names, &renderables)
                    .join()
                    .filter(|(e, s, _n, _r)| *e != *player_entity && s.hp < 1)
            {
                let position = { positions.get(entity).unwrap().clone() };
                let visible_to_player = player_viewshed.visible_tiles.contains(&position.idx);
                if let Some(m) = monsters.get(entity) {
                    let damage_history = damage_histories.get(entity).unwrap();
                    let mut items = inventory.get(entity).unwrap().items.clone();
                    if let Some(equip) = equipment.get(entity) {
                        for i in equip.as_items().drain() {
                            items.insert(i);
                        }
                    }
                    match m.species {
                        MonsterSpecies::Goblin => {
                            corpse_spawner.request_goblin_corpse(
                                position.idx,
                                position.level,
                                damage_history.describe_in_past_tense(),
                                items,
                            );
                        }
                    }
                    if visible_to_player {
                        log.add(format!("{} has died", name.name));
                    }
                } else {
                    let name = names.get(entity).unwrap();
                    debris_spawner.request(
                        position.idx,
                        renderable.fg,
                        renderable.bg,
                        35,
                        position.level,
                        format!("{} debris", name.name),
                        true,
                    );
                    if visible_to_player {
                        log.add(format!("{} has been destroyed", name.name));
                    }
                    if let Some(container) = containers.get(entity) {
                        container.items.iter().for_each(|e| {
                            positions
                                .insert(*e, position.clone())
                                .expect("could not insert position");
                        })
                    }
                }
                dead.push(entity);
            }
        }

        for victim in &dead {
            {
                let entities = ecs.entities();
                let mut hiding = ecs.write_storage::<Hiding>();
                let hiding_to_expose: Vec<Entity> = (&entities, &mut hiding)
                    .join()
                    .filter(|(_, h)| {
                        if let Some(hiding_spot) = h.hiding_spot {
                            return hiding_spot == *victim;
                        }
                        false
                    })
                    .map(|(e, _h)| e)
                    .collect();
                hiding_to_expose.iter().for_each(|e| {
                    hiding.remove(*e);
                });
            };
        }
        ecs.delete_entities(&dead)
            .expect("could not delete dead entities");
    }
}

impl<'a> System<'a> for DamageSystem<'a> {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, BloodSpawner>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Player>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut stats,
            mut suffer_damage,
            positions,
            mut blood_spawner,
            monsters,
            players,
            player_ent,
        ) = data;
        for (mut stats, suffer_damage, ent) in (&mut stats, &suffer_damage, &entities).join() {
            stats.hp -= suffer_damage.amount;
            // create blood
            if monsters.get(ent).is_some() || players.get(ent).is_some() {
                let position = positions.get(ent).unwrap();
                blood_spawner.request(
                    position.idx,
                    RGB::from_f32(0.85, 0., 0.),
                    RGB::from_f32(0.50, 0., 0.),
                    177,
                    position.level,
                );
            }
            if ent == *player_ent {
                self.queued_action.take();
            }
        }
        suffer_damage.clear();
    }
}
