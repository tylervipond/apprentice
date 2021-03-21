use std::collections::HashSet;

use super::{level::Level, rect::Rect, tile_type::TileType};
use rltk::{DistanceAlg::Pythagoras, Point, RandomNumberGenerator};
use specs::Entity;

pub fn xy_idx(width: u32, x: i32, y: i32) -> usize {
    (y * width as i32 + x) as usize
}

pub fn idx_xy(width: u32, idx: usize) -> (i32, i32) {
    (idx as i32 % width as i32, idx as i32 / width as i32)
}

pub fn add_xy_to_idx(width: i32, x: i32, y: i32, idx: i32) -> i32 {
    idx + x + width * y
}

pub fn idxs_are_adjacent(width: u8, idx1: usize, idx2: usize) -> bool {
    idx1 == idx2 + 1
        || idx1 == idx2 - 1
        || idx1 == idx2 + width as usize
        || idx1 == idx2 - width as usize
        || idx1 == idx2 + 1 + width as usize
        || idx1 == idx2 - 1 + width as usize
        || idx1 == idx2 + 1 - width as usize
        || idx1 == idx2 - 1 - width as usize
}

pub fn idx_point(width: u32, idx: usize) -> Point {
    let (x, y) = idx_xy(width, idx);
    Point::new(x, y)
}

pub fn get_tile_at_xy(level: &Level, x: i32, y: i32) -> Option<&TileType> {
    level.tiles.get(xy_idx(level.width as u32, x, y) as usize)
}

pub fn tile_at_xy_is_wall(level: &Level, x: i32, y: i32) -> bool {
    get_tile_at_xy(level, x, y) == Some(&TileType::Wall)
}

pub fn tile_at_xy_is_door(level: &Level, x: i32, y: i32) -> bool {
    get_tile_at_xy(level, x, y) == Some(&TileType::Door)
}

pub fn tile_is_door_adjacent(level: &Level, x: i32, y: i32) -> bool {
    tile_at_xy_is_door(level, x - 1, y)
        || tile_at_xy_is_door(level, x, y - 1)
        || tile_at_xy_is_door(level, x + 1, y)
        || tile_at_xy_is_door(level, x, y + 1)
}

pub fn tile_is_between_walls_vertical(level: &Level, x: i32, y: i32) -> bool {
    tile_at_xy_is_wall(level, x, y - 1) && tile_at_xy_is_wall(level, x, y + 1)
}

pub fn tile_is_between_walls_horizontal(level: &Level, x: i32, y: i32) -> bool {
    tile_at_xy_is_wall(level, x - 1, y) && tile_at_xy_is_wall(level, x + 1, y)
}

pub fn tile_is_between_walls(level: &Level, x: i32, y: i32) -> bool {
    tile_is_between_walls_horizontal(level, x, y) || tile_is_between_walls_vertical(level, x, y)
}

pub fn set_tile_to_floor(level: &mut Level, idx: usize) {
    level.tiles[idx] = TileType::Floor;
}

pub fn set_tile_to_door(level: &mut Level, idx: usize) {
    level.tiles[idx] = TileType::Door;
}

pub fn entities_at_idx(level: &Level, idx: usize) -> Vec<Entity> {
    level.tile_content[idx].to_vec()
}

pub fn populate_blocked(level: &mut Level) {
    for (i, tile) in level.tiles.iter_mut().enumerate() {
        level.blocked[i] = match *tile {
            TileType::Wall | TileType::Column | TileType::Ledge | TileType::Door => true,
            _ => false,
        }
    }
}

pub fn populate_opaque(level: &mut Level) {
    for (i, tile) in level.tiles.iter_mut().enumerate() {
        level.opaque[i] = match *tile {
            TileType::Wall | TileType::Column | TileType::Door => true,
            _ => false,
        }
    }
}

pub fn tile_is_blocked(idx: usize, level: &Level) -> bool {
    level.blocked[idx]
}

pub fn idx_not_in_map(level: &Level, idx: usize) -> bool {
    !level.tiles.get(idx).is_some()
}

pub fn is_exit_valid(level: &Level, idx: usize) -> bool {
    if idx > level.tiles.len() {
        return false;
    }
    let tile = level.tiles[idx];
    !(tile == TileType::Wall || tile == TileType::Column || tile == TileType::Ledge)
}

pub fn clear_content_index(level: &mut Level) {
    for content in level.tile_content.iter_mut() {
        content.clear();
    }
}

pub fn get_walkable_tiles_in_rect(rect: &Rect, level: &Level) -> Vec<usize> {
    (rect.y1 + 1..rect.y2)
        .map(|y| {
            (rect.x1 + 1..rect.x2)
                .map(|x| (x, y.clone()))
                .collect::<Vec<(i32, i32)>>()
        })
        .flatten()
        .map(|(x, y)| xy_idx(level.width as u32, x, y))
        .filter(|idx| {
            level.tiles[*idx as usize] == TileType::Floor && !tile_is_blocked(*idx, level)
        })
        .collect()
}

pub fn filter_water_from_tiles(tiles: Vec<usize>, level: &Level) -> Vec<usize> {
    tiles
        .iter()
        .filter(|idx| level.tiles[**idx as usize] != TileType::WaterDeep)
        .map(|idx| idx.to_owned())
        .collect()
}

pub fn get_random_spawn_point(
    rect: &Rect,
    level: &Level,
    rng: &mut RandomNumberGenerator,
) -> usize {
    let walkable_tiles_in_rect =
        filter_water_from_tiles(get_walkable_tiles_in_rect(rect, level), level);
    let selected_index = rng.range(0, walkable_tiles_in_rect.len());
    walkable_tiles_in_rect[selected_index]
}

pub fn get_random_unblocked_floor_point(
    level: &Level,
    rng: &mut RandomNumberGenerator,
) -> Option<usize> {
    let floor_tiles: Vec<usize> = level
        .tiles
        .iter()
        .enumerate()
        .filter(|(idx, t)| *t == &TileType::Floor && !level.blocked[*idx])
        .map(|(i, _t)| i)
        .collect();
    match floor_tiles.len() {
        0 => None,
        count => {
            let idx = rng.range(0, count);
            floor_tiles.get(idx).cloned()
        }
    }
}

pub fn get_spawn_points(
    rect: &Rect,
    level: &Level,
    rng: &mut RandomNumberGenerator,
    count: i32,
) -> Vec<usize> {
    (0..count)
        .map(|_| get_random_spawn_point(rect, level, rng))
        .collect()
}

pub fn get_all_unblocked_tiles_in_radius(
    level: &Level,
    center_idx: usize,
    radius_length: u32,
) -> Vec<usize> {
    let center_xy = idx_xy(level.width as u32, center_idx);
    let center_point = Point::new(center_xy.0, center_xy.1);
    return level
        .tiles
        .iter()
        .enumerate()
        .map(|(tile_idx, _tile_type)| tile_idx)
        .filter(|tile_idx| {
            let this_xy = idx_xy(level.width as u32, *tile_idx);
            let this_point = Point::new(this_xy.0, this_xy.1);
            Pythagoras.distance2d(center_point, this_point) <= radius_length as f32
                && !tile_is_blocked(*tile_idx, level)
        })
        .collect();
}

pub fn get_all_spawnable_tiles_in_radius(
    level: &Level,
    center_idx: usize,
    radius_length: u32,
) -> Vec<usize> {
    filter_water_from_tiles(
        get_all_unblocked_tiles_in_radius(level, center_idx, radius_length),
        level,
    )
}

pub fn get_field_of_view_from_idx(level: &Level, idx: usize, radius: u32) -> HashSet<usize> {
    rltk::field_of_view(idx_point(level.width as u32, idx), radius as i32, &*level)
        .iter()
        .map(|p| xy_idx(level.width as u32, p.x, p.y))
        .collect()
}

pub fn get_distance_between_idxs(level: &Level, idx1: usize, idx2: usize) -> f32 {
    let point1 = idx_point(level.width as u32, idx1);
    let point2 = idx_point(level.width as u32, idx2);
    Pythagoras.distance2d(point1, point2)
}

pub fn get_neighbors_for_idx(level_width: i32, idx: i32) -> [i32; 8] {
    [
        idx + 1,
        idx - 1,
        idx - level_width,
        idx + level_width,
        idx + 1 - level_width,
        idx + 1 + level_width,
        idx - 1 - level_width,
        idx - 1 + level_width,
    ]
}
