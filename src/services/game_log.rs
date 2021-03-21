pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    pub fn add(&mut self, message: String) {
        self.entries.insert(0, message);
    }
}
