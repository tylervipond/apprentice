use crate::types::ItemType;

pub struct ItemSpawnerRequest {
    pub idx: usize,
    pub level: u8,
    pub item_type: ItemType,
}

pub struct ItemSpawner {
    pub requests: Vec<ItemSpawnerRequest>,
}

impl ItemSpawner {
    pub fn new() -> Self {
        ItemSpawner {
            requests: Vec::new(),
        }
    }

    pub fn request(&mut self, idx: usize, level: u8, item_type: ItemType) {
        self.requests.push(ItemSpawnerRequest {
            idx,
            level,
            item_type,
        })
    }
}
