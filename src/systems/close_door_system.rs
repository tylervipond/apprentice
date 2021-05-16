use crate::components::{door::DoorState, Door, Position, Renderable, Viewshed, WantsToCloseDoor};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use rltk::{BLACK, BROWN4, RGB};
use specs::{Join, ReadStorage, System, WriteExpect, WriteStorage};
use std::collections::HashSet;

pub struct CloseDoorSystem {}

impl<'a> System<'a> for CloseDoorSystem {
    type SystemData = (
        WriteExpect<'a, Dungeon>,
        WriteStorage<'a, WantsToCloseDoor>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Door>,
        WriteStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut dungeon,
            mut wants_to_close_door,
            positions,
            mut viewsheds,
            mut doors,
            mut renderables,
        ) = data;
        let mut levels_with_door_close = HashSet::new();
        for intent in (&wants_to_close_door).join() {
            if let Some(door) = doors.get_mut(intent.door) {
                door.state = DoorState::Closed;
                let door_position = positions.get(intent.door).unwrap();
                let mut level = dungeon.get_level_mut(door_position.level).unwrap();
                level_utils::set_tile_to_door(&mut level, door_position.idx);
                level.blocked[door_position.idx] = true;
                level.opaque[door_position.idx] = true;
                levels_with_door_close.insert(door_position.level);
                let mut door_renderable = renderables.get_mut(intent.door).unwrap();
                door_renderable.fg = RGB::named(BROWN4);
                door_renderable.bg = RGB::named(BLACK);
            }
        }
        if levels_with_door_close.len() > 0 {
            (&positions, &mut viewsheds)
                .join()
                .filter(|(p, _)| levels_with_door_close.contains(&p.level))
                .for_each(|(_, v)| v.dirty = true);
        }
        wants_to_close_door.clear();
    }
}
