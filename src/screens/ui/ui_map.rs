use std::collections::HashSet;

use crate::dungeon::{level::Level, level_utils, tile_type::TileType};
use crate::screens::constants::MAP_WIDTH;
use rltk::{Rltk, RGB};

pub struct RenderData {
    pub idx: usize,
    pub fg: RGB,
    pub bg: RGB,
    pub layer: i32,
    pub glyph: u16,
}

pub fn is_revealed_and_wall_or_door(level: &Level, x: i32, y: i32) -> bool {
    let idx = level_utils::xy_idx(level.width as u32, x, y) as usize;
    match level.tiles.get(idx) {
        Some(tile) => {
            (*tile == TileType::Wall || *tile == TileType::Door) && level.revealed_tiles[idx]
        }
        None => false,
    }
}

fn is_revealed_and_ledge(level: &Level, x: i32, y: i32) -> bool {
    let idx = level_utils::xy_idx(level.width as u32, x, y) as usize;
    match level.tiles.get(idx) {
        Some(tile) => (*tile == TileType::Ledge) && level.revealed_tiles[idx],
        None => false,
    }
}

pub fn get_wall_tile(level: &Level, x: i32, y: i32) -> u16 {
    let mut mask: u8 = 0;
    if is_revealed_and_wall_or_door(level, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall_or_door(level, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall_or_door(level, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall_or_door(level, x + 1, y) {
        mask += 8;
    }
    match mask {
        0 => 9,
        1 | 2 | 3 => 186,
        4 | 8 | 12 => 205,
        5 => 188,
        6 => 187,
        7 => 185,
        9 => 200,
        10 => 201,
        11 => 204,
        13 => 202,
        14 => 203,
        15 => 206,
        _ => 35,
    }
}

fn get_ledge_tile(level: &Level, x: i32, y: i32) -> u16 {
    let mut mask: u8 = 0;
    if is_revealed_and_ledge(level, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_ledge(level, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_ledge(level, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_ledge(level, x + 1, y) {
        mask += 8;
    }
    match mask {
        0 => 9,
        1 | 2 | 3 => 179,
        4 | 8 | 12 => 196,
        5 => 217,
        6 => 191,
        7 => 180,
        9 => 192,
        10 => 218,
        11 => 195,
        13 => 193,
        14 => 194,
        15 => 197,
        _ => 35,
    }
}

fn get_bg_color(tile_type: &TileType) -> RGB {
    match tile_type {
        TileType::WaterDeep => RGB::named(rltk::BLUE),
        _ => RGB::named(rltk::BLACK),
    }
}

pub struct UIMap<'a> {
    level: &'a Level,
    renderables: &'a Vec<RenderData>,
    render_offset: (i32, i32),
    visible_tiles: &'a HashSet<usize>,
}

impl<'a> UIMap<'a> {
    pub fn new(
        level: &'a Level,
        renderables: &'a Vec<RenderData>,
        render_offset: (i32, i32),
        visible_tiles: &'a HashSet<usize>,
    ) -> Self {
        Self {
            level,
            renderables,
            render_offset,
            visible_tiles,
        }
    }

    pub fn draw(&mut self, ctx: &mut Rltk) {
        // this could be better, the level knows what ents are there and we can get the renderables from there
        // why bother collecting them beforehand?
        for (i, tile) in self.level.tiles.iter().enumerate() {
            if self.level.revealed_tiles[i] {
                let x = i % MAP_WIDTH as usize;
                let y = (i - (i % MAP_WIDTH as usize)) / MAP_WIDTH as usize;
                let character = match tile {
                    TileType::Floor | TileType::Door => rltk::to_cp437('.'),
                    TileType::Wall => get_wall_tile(&self.level, x as i32, y as i32),
                    TileType::Column => 9,
                    TileType::DownStairs => rltk::to_cp437('>'),
                    TileType::UpStairs => rltk::to_cp437('<'),
                    TileType::Exit => 219,
                    TileType::WaterDeep => 176,
                    TileType::Ledge => get_ledge_tile(&self.level, x as i32, y as i32),
                };
                let foreground_color = match self.visible_tiles.contains(&i) {
                    true => rltk::GREEN,
                    false => rltk::WHITE,
                };
                ctx.set(
                    x as i32 - self.render_offset.0,
                    y as i32 - self.render_offset.1,
                    RGB::named(foreground_color),
                    get_bg_color(tile),
                    character,
                )
            }
        }
        // since we're just going to reset the context, it seems a shame we did the above
        // `work
        for r in self.renderables.iter() {
            let (x, y) = level_utils::idx_xy(self.level.width as u32, r.idx);
            ctx.set(
                x - self.render_offset.0,
                y - self.render_offset.1,
                r.fg,
                r.bg,
                r.glyph,
            );
        }
    }
}
