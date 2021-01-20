use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::{
    components::{Monster, Position, Viewshed},
    player::InteractionType,
};
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

pub struct VisibilitySystem<'a> {
    pub queued_action: &'a mut Option<(Entity, InteractionType)>,
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
                        && player_viewshed.visible_tiles.contains(&(p.idx as i32))
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
                        position.idx as i32,
                        viewshed.range,
                    );
                }
                viewshed.visible_tiles = viewshed
                    .los_tiles
                    .iter()
                    .cloned()
                    .filter(|idx| level.lit_tiles[*idx as usize])
                    .chain(
                        level_utils::get_field_of_view_from_idx(&*level, position.idx as i32, 2)
                            .iter()
                            .cloned(),
                    )
                    .collect();
                if ent == *player_ent {
                    for t in level.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for idx in viewshed.visible_tiles.iter() {
                        level.revealed_tiles[*idx as usize] = true;
                        level.visible_tiles[*idx as usize] = true;
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
                        && player_viewshed.visible_tiles.contains(&(p.idx as i32))
                        && !enemies_in_player_sight_at_start.contains(e)
                })
        } {
            self.queued_action.take();
        }
    }
}
