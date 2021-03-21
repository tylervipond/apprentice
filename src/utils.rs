use crate::components::position::Position;
use rltk::RandomNumberGenerator;
use specs::{Entity, World, WorldExt};

pub fn get_random_between_numbers(rng: &mut RandomNumberGenerator, n1: i32, n2: i32) -> i32 {
    n1 + rng.roll_dice(1, i32::abs(n2 - n1))
}

pub fn get_x_random_elements<T>(
    rng: &mut RandomNumberGenerator,
    x: u32,
    elements: &mut Vec<T>,
) -> Vec<T> {
    (0..x)
        .map(|_| {
            let idx = rng.range(0, elements.len() as i32);
            elements.remove(idx as usize)
        })
        .collect()
}

pub fn get_random_element<'a, T>(rng: &mut RandomNumberGenerator, elements: &'a Vec<T>) -> &'a T {
    let idx = rng.range(0, elements.len() as i32);
    elements.get(idx as usize).unwrap()
}

pub fn get_current_level_from_world(world: &World) -> u8 {
    let player_ent = world.fetch::<Entity>();
    let dungeon_level = world.read_storage::<Position>();
    dungeon_level.get(*player_ent).unwrap().level
}

pub fn select_next_idx(idx: usize, length: usize) -> usize {
    if idx + 1 >= length {
        return 0;
    }
    idx + 1
}

pub fn select_previous_idx(idx: usize, length: usize) -> usize {
    if idx == 0 {
        return length - 1;
    }
    idx - 1
}
