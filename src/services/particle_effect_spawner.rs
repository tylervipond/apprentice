use rltk::{RGB, to_cp437, ORANGE, BLACK, BLUE};

pub struct ParticleEffectSpawnerRequest {
    pub x: i32,
    pub y: i32,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: u16,
    pub lifetime: f32,
    pub level: u8,
}

pub struct ParticleEffectSpawner {
    pub requests: Vec<ParticleEffectSpawnerRequest>,
}

impl ParticleEffectSpawner {
    pub fn new() -> Self {
        ParticleEffectSpawner {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: u16,
        lifetime: f32,
        level: u8,
    ) {
        self.requests.push(ParticleEffectSpawnerRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
            level,
        })
    }

    pub fn request_attack_particle(&mut self, x: i32, y: i32, level: u8) {
        self.request(
            x,
            y,
            RGB::named(ORANGE),
            RGB::named(BLACK),
            to_cp437('‼'),
            200.0,
            level,
          );
    }

    pub fn request_search_particle(&mut self, x: i32, y: i32, level: u8) {
        self.request(
            x,
            y,
            RGB::named(BLUE),
            RGB::named(BLACK),
            to_cp437('?'),
            200.0,
            level,
          );
    }
}
