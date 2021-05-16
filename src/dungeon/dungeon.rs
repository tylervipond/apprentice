use super::level::Level;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Dungeon {
  pub levels: Vec<Level>,
}

impl Dungeon {
  pub fn get_level(&self, floor: usize) -> Option<&Level> {
    self.levels.get(floor)
  }

  pub fn get_level_mut(&mut self, floor: usize) -> Option<&mut Level> {
    self.levels.get_mut(floor)
  }
}
