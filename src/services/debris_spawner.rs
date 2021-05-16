use rltk::{BLACK, DARK_GRAY, RGB, to_cp437};

pub struct DebrisSpawnerRequest {
    pub idx: usize,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: u16,
    pub level: usize,
    pub name: String,
    pub flammable: bool
}

pub struct DebrisSpawner {
    pub requests: Vec<DebrisSpawnerRequest>,
}

impl DebrisSpawner {
    pub fn new() -> Self {
        DebrisSpawner {
            requests: Vec::new(),
        }
    }

    pub fn request(&mut self, idx: usize, fg: RGB, bg: RGB, glyph: u16, level: usize, name: String, flammable: bool) {
        self.requests.push(DebrisSpawnerRequest {
            idx,
            fg,
            bg,
            glyph,
            level,
            name,
            flammable
        })
    }

    pub fn request_burnt_debris(&mut self, idx: usize, level: usize) {
        self.request(
            idx,
            RGB::named(BLACK),
            RGB::named(DARK_GRAY),
            to_cp437('#'),
            level,
            String::from("Burnt Debris"),
            false
        )
    }
}
