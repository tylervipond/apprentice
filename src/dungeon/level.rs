use super::level_utils;
use super::room::Room;
use super::tile_type::TileType;
use rltk::{Algorithm2D, BaseMap, DistanceAlg::Pythagoras, Point, SmallVec};
use serde::{Deserialize, Serialize};
use specs::Entity;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Level {
    pub height: usize,
    pub width: usize,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Room>,
    pub revealed_tiles: Vec<bool>,
    pub lit_tiles: Vec<bool>, // can we skip serializing this?d
    pub blocked: Vec<bool>,
    pub opaque: Vec<bool>,
    pub depth: usize,
    pub stairs_down: Option<usize>,
    pub stairs_up: Option<usize>,
    pub exit: Option<usize>,
    #[serde(skip_serializing, skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Level {
    pub fn new(depth: usize, width: usize, height: usize) -> Self {
        let map_count = width * height;
        Self {
            tiles: (0..map_count).map(|_| TileType::Wall).collect(),
            rooms: vec![], // TODO: determine if this is useful beyond the level building phase
            width,
            height,
            revealed_tiles: (0..map_count).map(|_| false).collect(),
            lit_tiles: (0..map_count).map(|_| false).collect(),
            blocked: (0..map_count).map(|_| false).collect(),
            opaque: (0..map_count).map(|_| false).collect(),
            tile_content: (0..map_count).map(|_| vec![]).collect(),
            stairs_down: None,
            stairs_up: None,
            exit: None,
            depth,
        }
    }
    pub fn get_costs_for_tile(&self, idx: usize, diagonal: bool) -> f32 {
        let cost = match diagonal {
            true => 1.45,
            false => 1.0,
        };
        match self.blocked[idx] {
            true => match self.tiles[idx] {
                TileType::Door => cost + 1.0,
                _ => cost + 25.0,
            },
            false => cost,
        }
    }
}

impl BaseMap for Level {
    fn is_opaque(&self, idx: usize) -> bool {
        self.opaque[idx]
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        for (idx, diagonal) in [
            (idx - 1, false),
            (idx + 1, false),
            (idx - self.width as usize, false),
            (idx + self.width as usize, false),
            (idx - self.width as usize - 1, true),
            (idx - self.width as usize + 1, true),
            (idx + self.width as usize - 1, true),
            (idx + self.width as usize + 1, true),
        ]
        .iter()
        {
            if level_utils::is_exit_valid(self, *idx) {
                exits.push((*idx, self.get_costs_for_tile(*idx, *diagonal)))
            }
        }
        exits
    }
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let p1 = self.index_to_point2d(idx1);
        let p2 = self.index_to_point2d(idx2);
        Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Level {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
