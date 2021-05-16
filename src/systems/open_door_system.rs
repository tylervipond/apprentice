use crate::components::{door::DoorState, Door, Position, Renderable, Viewshed, WantsToOpenDoor};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use rltk::{BLACK, DARK_GRAY, RGB};
use specs::{Join, ReadStorage, System, WriteExpect, WriteStorage};
use std::collections::HashSet;

pub struct OpenDoorSystem {}

impl<'a> System<'a> for OpenDoorSystem {
    type SystemData = (
        WriteExpect<'a, Dungeon>,
        WriteStorage<'a, WantsToOpenDoor>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Door>,
        WriteStorage<'a, Renderable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut dungeon,
            mut wants_to_open_door,
            positions,
            mut viewsheds,
            mut doors,
            mut renderables,
        ) = data;
        let mut levels_with_door_open = HashSet::new();
        for intent in (&wants_to_open_door).join() {
            if let Some(door) = doors.get_mut(intent.door) {
                door.state = DoorState::Opened;
                let door_position = positions.get(intent.door).unwrap();
                let mut level = dungeon.get_level_mut(door_position.level).unwrap();
                level_utils::set_tile_to_floor(&mut level, door_position.idx);
                level.blocked[door_position.idx] = false;
                level.opaque[door_position.idx] = false;
                levels_with_door_open.insert(door_position.level);
                let mut door_renderable = renderables.get_mut(intent.door).unwrap();
                door_renderable.bg = RGB::named(BLACK);
                door_renderable.fg = RGB::named(DARK_GRAY);
            }
        }
        if levels_with_door_open.len() > 0 {
            (&positions, &mut viewsheds)
                .join()
                .filter(|(p, _)| levels_with_door_open.contains(&p.level))
                .for_each(|(_, v)| v.dirty = true);
        }
        wants_to_open_door.clear();
    }
}
