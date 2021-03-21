use crate::dungeon::dungeon::Dungeon;
use crate::{
    components::{Position, Viewshed, WantsToGoUpStairs},
    dungeon::level_utils,
};
use specs::{Entities, Join, ReadExpect, System, WriteStorage};
pub struct GoUpStairsSystem {}

impl<'a> System<'a> for GoUpStairsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToGoUpStairs>,
        WriteStorage<'a, Position>,
        ReadExpect<'a, Dungeon>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_go_up_stairs, mut positions, dungeon, mut viewsheds) = data;
        for (intent, position, entity) in
            (&mut wants_to_go_up_stairs, &mut positions, &entities).join()
        {
            let level = dungeon.get_level(position.level).unwrap();
            if level_utils::idxs_are_adjacent(level.width, position.idx, intent.idx) {
                if let Some(next_level) = dungeon.get_level(position.level + 1) {
                    position.level = position.level + 1;
                    position.idx = next_level.stairs_down.unwrap();
                    let mut viewshed = viewsheds.get_mut(entity).unwrap();
                    viewshed.dirty = true;
                }
            }
        }
        wants_to_go_up_stairs.clear();
    }
}
