use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::services::GameLog;
use crate::{
    components::{Hidden, Name, Position, Viewshed},
    interaction_type::InteractionType,
};
use rltk::RandomNumberGenerator;
use specs::{Entity, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

pub struct RevealTrapsSystem<'a> {
    pub queued_action: &'a mut Option<InteractionType>,
}

impl<'a> System<'a> for RevealTrapsSystem<'a> {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Dungeon>,
        ReadExpect<'a, Entity>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadStorage<'a, Name>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewsheds, positions, dungeon, player_ent, mut hidden, mut rng, names, mut log) = data;
        let dungeon_level = positions.get(*player_ent).unwrap();
        let level = dungeon.get_level(dungeon_level.level).unwrap();
        let player_viewshed = viewsheds.get(*player_ent).unwrap();
        player_viewshed
            .visible_tiles
            .iter()
            .map(|idx| level_utils::entities_at_idx(&level, *idx as usize))
            .flatten()
            .for_each(|e| {
                if let Some(this_hidden) = hidden.get_mut(e) {
                    if rng.roll_dice(1, 24) == 1 {
                        self.queued_action.take();
                        let inserted = this_hidden.found_by.insert(*player_ent);
                        if inserted {
                            if let Some(name) = names.get(e) {
                                log.add(format!("You spotted a {}.", name.name));
                            }
                        }
                    }
                }
            });
    }
}
