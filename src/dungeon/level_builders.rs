use super::room_feature::RoomFeature::{
    ColumnsDoubleAll, ColumnsDoubleBottom, ColumnsDoubleHorizontal, ColumnsDoubleLeft,
    ColumnsDoubleMiddle, ColumnsDoubleRight, ColumnsDoubleTop, ColumnsDoubleVertical,
    ColumnsSingleAll, ColumnsSingleBottom, ColumnsSingleHorizontal, ColumnsSingleLeft,
    ColumnsSingleMiddle, ColumnsSingleRight, ColumnsSingleTop, ColumnsSingleVertical,
    ColumnsTripleAll, ColumnsTripleBottom, ColumnsTripleHorizontal, ColumnsTripleLeft,
    ColumnsTripleRight, ColumnsTripleTop, ColumnsTripleVertical,
};
use super::{
    column_placers,
    level::Level,
    level_utils,
    rect::Rect,
    room::Room,
    room_decorators,
    room_decorators::{
        RoomPart,
        RoomPart::{Column, Door, DownStairs, Exit, Floor, Ledge, UpStairs, Wall, WaterDeep},
    },
    tile_type::TileType,
};
use crate::utils::get_x_random_elements;
use rltk::{DistanceAlg::Pythagoras, Point, RandomNumberGenerator};
use stamp_rs::{
    StampPart,
    StampPart::{Transparent, Use},
};
use std::cmp;
use std::collections::HashMap;

fn generate_rects_for_level(
    level_width: i32,
    level_height: i32,
    rng: &mut RandomNumberGenerator,
) -> Vec<Rect> {
    const MIN_ROOM_SIZE: i32 = 5;
    let mut rects = Vec::new();
    rects.push(Rect::new(1, 1, level_width - 3, level_height - 3));
    for _ in 0..100 {
        let random_index = rng.range(0, rects.len() as i32);
        let rect = rects[random_index as usize];
        let width = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        if width > MIN_ROOM_SIZE * 2 && width > height {
            let x = cmp::max(
                rect.x1 + MIN_ROOM_SIZE,
                rng.range(rect.x1, rect.x2 - MIN_ROOM_SIZE),
            );
            rects.remove(random_index as usize);
            rects.push(Rect::new(rect.x1, rect.y1, x - rect.x1, rect.y2 - rect.y1));
            rects.push(Rect::new(x, rect.y1, rect.x2 - x, rect.y2 - rect.y1));
        } else if height > MIN_ROOM_SIZE * 2 && height >= width {
            let y = cmp::max(
                rect.y1 + MIN_ROOM_SIZE,
                rng.range(rect.y1, rect.y2 - MIN_ROOM_SIZE),
            );
            rects.remove(random_index as usize);
            rects.push(Rect::new(rect.x1, rect.y1, rect.x2 - rect.x1, y - rect.y1));
            rects.push(Rect::new(rect.x1, y, rect.x2 - rect.x1, rect.y2 - y));
        }
    }
    rects
}

fn update_level_from_room_features(level: &mut Level, rng: &mut RandomNumberGenerator) {
    let mut updates: Vec<((i32, i32), TileType)> = vec![];
    for room in &level.rooms {
        let rect = &room.rect;
        for feature in &room.features {
            let mut additional_updates: Vec<((i32, i32), TileType)> = match feature {
                Some(ColumnsSingleLeft) => column_placers::get_columns_left(rect, rng, 1),
                Some(ColumnsDoubleLeft) => column_placers::get_columns_left(rect, rng, 2),
                Some(ColumnsTripleLeft) => column_placers::get_columns_left(rect, rng, 3),
                Some(ColumnsSingleRight) => column_placers::get_columns_right(rect, rng, 1),
                Some(ColumnsDoubleRight) => column_placers::get_columns_right(rect, rng, 2),
                Some(ColumnsTripleRight) => column_placers::get_columns_right(rect, rng, 3),
                Some(ColumnsSingleBottom) => column_placers::get_columns_bottom(rect, rng, 1),
                Some(ColumnsDoubleBottom) => column_placers::get_columns_bottom(rect, rng, 2),
                Some(ColumnsTripleBottom) => column_placers::get_columns_bottom(rect, rng, 3),
                Some(ColumnsSingleTop) => column_placers::get_columns_top(rect, rng, 1),
                Some(ColumnsDoubleTop) => column_placers::get_columns_top(rect, rng, 2),
                Some(ColumnsTripleTop) => column_placers::get_columns_top(rect, rng, 3),
                Some(ColumnsSingleVertical) => column_placers::get_columns_vertical(rect, rng, 1),
                Some(ColumnsDoubleVertical) => column_placers::get_columns_vertical(rect, rng, 2),
                Some(ColumnsTripleVertical) => column_placers::get_columns_vertical(rect, rng, 3),
                Some(ColumnsSingleHorizontal) => {
                    column_placers::get_columns_horizontal(rect, rng, 1)
                }
                Some(ColumnsDoubleHorizontal) => {
                    column_placers::get_columns_horizontal(rect, rng, 2)
                }
                Some(ColumnsTripleHorizontal) => {
                    column_placers::get_columns_horizontal(rect, rng, 3)
                }
                Some(ColumnsSingleAll) => column_placers::get_columns_all(rect, rng, 1),
                Some(ColumnsDoubleAll) => column_placers::get_columns_all(rect, rng, 2),
                Some(ColumnsTripleAll) => column_placers::get_columns_all(rect, rng, 3),
                Some(ColumnsSingleMiddle) => column_placers::get_columns_single_middle(rect),
                Some(ColumnsDoubleMiddle) => column_placers::get_columns_double_middle(rect, rng),
                _ => vec![],
            };
            &mut updates.append(&mut additional_updates);
        }
    }
    let level_width = level.width as u32;
    for (xy, tile_type) in &updates {
        if !level_utils::tile_at_xy_is_wall(level, xy.0, xy.1)
            && !level_utils::tile_is_between_walls(level, xy.0, xy.1)
            && !level_utils::tile_is_door_adjacent(level, xy.0, xy.1)
        {
            let idx = level_utils::xy_idx(level_width, xy.0, xy.1);
            level.tiles[idx as usize] = *tile_type;
        }
    }
}

fn update_level_from_room_stamps(level: &mut Level) {
    let mut updates: Vec<(usize, usize, Option<TileType>)> = vec![];
    for room in level.rooms.iter() {
        for x in room.rect.x1..room.rect.x2 {
            for y in room.rect.y1..room.rect.y2 {
                let room_x = x - room.rect.x1;
                let room_y = y - room.rect.y1;
                let tile_type = match room.stamp.get_at((room_x as usize, room_y as usize)) {
                    Some(Use(Ledge)) => Some(TileType::Ledge),
                    Some(Use(WaterDeep)) => Some(TileType::WaterDeep),
                    _ => None,
                };
                updates.push((x as usize, y as usize, tile_type));
            }
        }
    }
    let width = level.width as u32;
    updates.iter().for_each(|(x, y, tile_type)| {
        let idx = level_utils::xy_idx(width, *x as i32, *y as i32);
        if let Some(t) = tile_type {
            level.tiles[idx as usize] = *t;
        }
    });
}

fn decorate_level(level: &mut Level, rng: &mut RandomNumberGenerator) {
    level.rooms.iter_mut().for_each(|room| {
        room_decorators::decorate_room(&mut room.stamp, &room.room_type, rng);
    });
}

fn update_room_stamps_from_level(level: &mut Level) {
    let mut updates: Vec<(usize, usize, usize, StampPart<RoomPart>)> = vec![];
    level.rooms.iter_mut().for_each(|room| {
        // the rect needs to be expanded at this point so that the stamp can have all 4 walls
        // TODO: This feels messey and should be part of a level generation refactor.
        room.rect.x2 = room.rect.x2 + 1;
        room.rect.y2 = room.rect.y2 + 1;
    });
    for (room_index, room) in level.rooms.iter().enumerate() {
        for x in room.rect.x1..room.rect.x2 {
            for y in room.rect.y1..room.rect.y2 {
                let room_x = x - room.rect.x1;
                let room_y = y - room.rect.y1;
                let room_stamp_part = match level_utils::get_tile_at_xy(level, x, y) {
                    Some(&TileType::Floor) => Use(Floor),
                    Some(&TileType::Wall) => Use(Wall),
                    Some(&TileType::Door) => Use(Door),
                    Some(&TileType::DownStairs) => Use(DownStairs),
                    Some(&TileType::UpStairs) => Use(UpStairs),
                    Some(&TileType::Exit) => Use(Exit),
                    Some(&TileType::Column) => Use(Column),
                    Some(&TileType::Ledge) => Use(Ledge),
                    Some(&TileType::WaterDeep) => Use(WaterDeep),
                    None => Transparent,
                };
                updates.push((
                    room_index,
                    room_x as usize,
                    room_y as usize,
                    room_stamp_part,
                ));
            }
        }
    }
    updates
        .iter()
        .for_each(|(room_index, room_x, room_y, stamp_part)| {
            level.rooms[*room_index]
                .stamp
                .set_at((*room_x, *room_y), stamp_part.clone());
        });
}

fn add_rectangular_room(level: &mut Level, room: &Rect) {
    let width = level.width as u32;
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let idx = level_utils::xy_idx(width, x, y) as usize;
            level_utils::set_tile_to_floor(level, idx);
        }
    }
}

fn add_circular_room(level: &mut Level, room: &Rect) {
    let radius = i32::min(room.x2 - room.x1, room.y2 - room.y1) as f32 / 2.0;
    let center_point = Point::from(room.center());
    let width = level.width as u32;
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let distance = Pythagoras.distance2d(center_point, rltk::Point::new(x, y));
            if distance < radius {
                let idx = level_utils::xy_idx(width, x, y);
                level_utils::set_tile_to_floor(level, idx as usize);
            }
        }
    }
}

fn add_horizontal_tunnel(level: &mut Level, x1: i32, x2: i32, y: i32) {
    let width = level.width as u32;
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        let idx = level_utils::xy_idx(width, x, y);
        if level.tiles.get(idx).is_some() {
            level_utils::set_tile_to_floor(level, idx as usize);
        }
    }
}

fn add_vertical_tunnel(level: &mut Level, x: i32, y1: i32, y2: i32) {
    let width = level.width as u32;
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        let idx = level_utils::xy_idx(width, x, y);
        if level.tiles.get(idx).is_some() {
            level_utils::set_tile_to_floor(level, idx as usize);
        }
    }
}

fn add_corridor(level: &mut Level, rng: &mut RandomNumberGenerator, from: Point, to: Point) {
    if rng.range(0, 2) == 1 {
        add_horizontal_tunnel(level, from.x, to.x, from.y);
        add_vertical_tunnel(level, to.x, to.y, from.y);
    } else {
        add_vertical_tunnel(level, from.x, from.y, to.y);
        add_horizontal_tunnel(level, from.x, to.x, to.y);
    }
}

fn add_nearest_neighbor_corridors(level: &mut Level, rng: &mut RandomNumberGenerator) {
    let mut connected: HashMap<usize, Vec<(Point, Point)>> = HashMap::new();
    for (i, room) in level.rooms.iter().enumerate() {
        let room_center_point = Point::from(room.rect.center());
        let mut room_distance: Vec<(usize, f32, Point)> = level
            .rooms
            .iter()
            .enumerate()
            .filter(|(j, _)| &i != j && !connected.contains_key(&j))
            .map(|(j, other_room)| {
                let other_room_center_point = Point::from(other_room.rect.center());
                let distance = Pythagoras.distance2d(room_center_point, other_room_center_point);
                (j, distance, other_room_center_point)
            })
            .collect();
        room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let points: Vec<(Point, Point)> = room_distance
            .iter()
            .take(2)
            .map(|(_, __, other_room_center_point)| (room_center_point, *other_room_center_point))
            .collect();
        connected.insert(i, points);
    }
    for points_pairs in connected.values() {
        for point_pair in points_pairs {
            add_corridor(level, rng, point_pair.0, point_pair.1);
        }
    }
}
// This is the part that needs to determine the locations of door ents.
fn add_doors_to_rooms(level: &mut Level) {
    let mut door_idxs: Vec<usize> = Vec::new();
    let level_width = level.width as u32;
    for room in level.rooms.iter() {
        for x in room.rect.x1..=room.rect.x2 {
            for y in room.rect.y1..=room.rect.y2 {
                // Checks the left and right walls of a room and adds a door if the tile is empty with a wall above & below
                if x == room.rect.x1 || x == room.rect.x2 {
                    if !level_utils::tile_at_xy_is_wall(&level, x, y)
                        && level_utils::tile_is_between_walls_vertical(&level, x, y)
                    {
                        door_idxs.push(level_utils::xy_idx(level_width, x, y));
                    }
                }

                // Checks the up and bottom walls of a room and adds a door if the tile is empty with a wall left & right
                if y == room.rect.y1 || y == room.rect.y2 {
                    if !level_utils::tile_at_xy_is_wall(&level, x, y)
                        && level_utils::tile_is_between_walls_horizontal(level, x, y)
                    {
                        door_idxs.push(level_utils::xy_idx(level_width, x, y));
                    }
                }
            }
        }
    }
    for idx in door_idxs.iter() {
        level_utils::set_tile_to_door(level, *idx as usize);
    }
}

fn make_rect_square(rect: &mut Rect) {
    let size_height = rect.y2 - rect.y1;
    let size_width = rect.x2 - rect.x1;
    let smallest_side = i32::min(size_width, size_height);
    rect.x1 = rect.x1 + (size_width - smallest_side) / 2;
    rect.x2 = rect.x1 + smallest_side;
    rect.y1 = rect.y1 + (size_height - smallest_side) / 2;
    rect.y2 = rect.y1 + smallest_side;
}


fn add_exit(level: &mut Level, rng: &mut RandomNumberGenerator) {
    let rect = level.rooms[0].rect;
    let exit_idx = level_utils::get_random_spawn_point(&rect, level, rng) as i32;
    level.tiles[exit_idx as usize] = TileType::Exit;
    level.exit = Some(exit_idx as usize);
}

fn add_down_stairs(level: &mut Level, rng: &mut RandomNumberGenerator) {
    let rect = level.rooms[level.rooms.len() - 1].rect;
    let stairs_idx = level_utils::get_random_spawn_point(&rect, level, rng) as i32;
    level.tiles[stairs_idx as usize] = TileType::DownStairs;
    level.stairs_down = Some(stairs_idx as usize);
}

fn add_up_stairs(level: &mut Level, rng: &mut RandomNumberGenerator) {
    let rect = level.rooms[0].rect;
    let stairs_idx = level_utils::get_random_spawn_point(&rect, level, rng) as i32;
    level.tiles[stairs_idx as usize] = TileType::UpStairs;
    level.stairs_up = Some(stairs_idx as usize);
}


pub fn build(depth: usize, width: usize, height: usize, is_top_floor:bool, is_bottom_floor: bool) -> Level {
    let mut level = Level::new(depth, width, height);
    let mut rng = RandomNumberGenerator::new();
    let mut rects = generate_rects_for_level(level.width as i32, level.height as i32, &mut rng);
    dbg!(&rects);
    let room_count = rng.range(2, rects.len() as i32);
    let mut room_rects = get_x_random_elements(&mut rng, room_count as u32, &mut rects);
    room_rects.iter_mut().for_each(|r| match rng.range(0, 6) {
        1 => {
            make_rect_square(r);
            add_circular_room(&mut level, r)
        }
        _ => add_rectangular_room(&mut level, r),
    });
    level.rooms = room_rects.iter().map(|r| Room::new(*r)).collect();
    add_nearest_neighbor_corridors(&mut level, &mut rng);
    add_doors_to_rooms(&mut level);
    update_level_from_room_features(&mut level, &mut rng);
    if !is_top_floor {
        add_up_stairs(&mut level, &mut rng);
    } else {
        add_exit(&mut level, &mut rng);
    }
    if !is_bottom_floor {
        add_down_stairs(&mut level, &mut rng);
    }
    update_room_stamps_from_level(&mut level);
    decorate_level(&mut level, &mut rng);
    update_level_from_room_stamps(&mut level);
    level_utils::populate_blocked(&mut level);
    level_utils::populate_opaque(&mut level);
    level
}
