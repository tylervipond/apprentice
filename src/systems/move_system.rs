use crate::components::{
    BlocksTile, EntityMoved, Grabbing, Hiding, Position, Viewshed, WantsToMove,
};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

pub struct MoveSystem {}

impl<'a> System<'a> for MoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Grabbing>,
        WriteStorage<'a, WantsToMove>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteExpect<'a, Dungeon>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, BlocksTile>,
        WriteStorage<'a, Hiding>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut grabbings,
            mut wants_to_moves,
            mut positions,
            mut viewsheds,
            mut dungeon,
            mut moved,
            blocks_tiles,
            mut hidings,
        ) = data;

        for (entity, wants_to_move, grabbing, viewshed) in (
            &entities,
            &mut wants_to_moves,
            (&mut grabbings).maybe(),
            (&mut viewsheds).maybe(),
        )
            .join()
        {
            let (ent_idx, ent_level) = {
                let pos = positions.get_mut(entity).unwrap();
                (pos.idx, pos.level)
            };
            let level = dungeon.get_level_mut(ent_level).unwrap();
            let delta = wants_to_move.idx as i32 - ent_idx as i32;
            let ent_destination_idx = wants_to_move.idx;
            if let Some(grabbing) = grabbing {
                let mut thing_pos = positions.get_mut(grabbing.thing).unwrap();
                let thing_destination_idx = thing_pos.idx as i32 + delta;
                let thing_destination_is_ent_position = thing_destination_idx == ent_idx as i32;
                let thing_destination_is_blocked =
                    level_utils::tile_is_blocked(thing_destination_idx as usize, &level)
                        && !thing_destination_is_ent_position;
                let thing_current_idx = thing_pos.idx;
                let ent_destination_is_thing_position = ent_destination_idx == thing_current_idx;
                let ent_destination_is_blocked =
                    level_utils::tile_is_blocked(ent_destination_idx, &level)
                        && !ent_destination_is_thing_position;
                if !thing_destination_is_blocked && !ent_destination_is_blocked {
                    thing_pos.idx = thing_destination_idx as usize;
                    if let Some(_) = blocks_tiles.get(grabbing.thing) {
                        level.blocked[thing_current_idx] = false;
                        level.blocked[thing_destination_idx as usize] = true;
                    }
                }
            }
            if !level_utils::tile_is_blocked(ent_destination_idx, &level) {
                if let Some(mut viewshed) = viewshed {
                    viewshed.dirty = true;
                }
                let mut ent_pos = positions.get_mut(entity).unwrap();
                ent_pos.idx = wants_to_move.idx;
                if let Some(_) = blocks_tiles.get(entity) {
                    level.blocked[ent_idx] = false;
                    level.blocked[ent_destination_idx] = true;
                }
                moved
                    .insert(entity, EntityMoved {})
                    .expect("unable to insert EntityMoved");
                hidings.remove(entity);
            }
        }
        wants_to_moves.clear();
    }
}
