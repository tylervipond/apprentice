use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::{
    components::{Monster, Position, Viewshed},
    interaction_type::InteractionType,
};
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct VisibilitySystem<'a> {
    pub queued_action: &'a mut Option<InteractionType>,
}

impl<'a> System<'a> for VisibilitySystem<'a> {
    type SystemData = (
        WriteExpect<'a, Dungeon>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Monster>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut dungeon, entities, mut viewsheds, positions, player_ent, monsters) = data;
        let player_position = &positions.get(*player_ent).unwrap();
        let enemies_in_player_sight_at_start: Box<[Entity]> = {
            let player_viewshed = &viewsheds.get(*player_ent).unwrap();
            (&positions, &entities, &monsters)
                .join()
                .filter(|(p, _e, _m)| {
                    p.level == player_position.level
                        && player_viewshed.visible_tiles.contains(&p.idx)
                })
                .map(|(_p, e, _m)| e)
                .collect()
        };
        {
            for (ent, viewshed, position) in (&entities, &mut viewsheds, &positions).join() {
                let level = dungeon.get_level_mut(position.level).unwrap();
                if viewshed.dirty {
                    viewshed.dirty = false;
                    viewshed.los_tiles.clear();
                    viewshed.los_tiles = level_utils::get_field_of_view_from_idx(
                        &*level,
                        position.idx,
                        viewshed.range,
                    );
                }
                let lit_indexes =
                    level
                        .lit_tiles
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, lit)| match lit {
                            true => Some(idx),
                            false => None,
                        });
                viewshed.visible_tiles = viewshed
                    .los_tiles
                    .clone()
                    .intersection(&HashSet::from_iter(lit_indexes))
                    .copied()
                    .collect::<HashSet<usize>>()
                    .union(&level_utils::get_field_of_view_from_idx(
                        &*level,
                        position.idx,
                        2,
                    ))
                    .copied()
                    .collect();

                if ent == *player_ent {
                    for idx in viewshed.visible_tiles.iter() {
                        level.revealed_tiles[*idx] = true;
                    }
                }
            }
        }
        if let Some(_) = {
            let player_viewshed = &viewsheds.get(*player_ent).unwrap();
            (&positions, &entities, &monsters)
                .join()
                .find(|(p, e, _m)| {
                    p.level == player_position.level
                        && player_viewshed.visible_tiles.contains(&p.idx)
                        && !enemies_in_player_sight_at_start.contains(e)
                })
        } {
            self.queued_action.take();
        }
    }
}
