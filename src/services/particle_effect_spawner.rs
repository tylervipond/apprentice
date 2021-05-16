use rltk::{RGB, to_cp437, ORANGE, BLACK, BLUE};

pub struct ParticleEffectSpawnerRequest {
    pub idx: usize,
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: u16,
    pub lifetime: f32,
    pub level: usize,
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
        idx: usize,
        fg: RGB,
        bg: RGB,
        glyph: u16,
        lifetime: f32,
        level: usize,
    ) {
        self.requests.push(ParticleEffectSpawnerRequest {
            idx,
            fg,
            bg,
            glyph,
            lifetime,
            level,
        })
    }

    pub fn request_attack_particle(&mut self, idx: usize, level: usize) {
        self.request(
            idx,
            RGB::named(ORANGE),
            RGB::named(BLACK),
            to_cp437('‼'),
            200.0,
            level,
          );
    }

    pub fn request_search_particle(&mut self, idx: usize, level: usize) {
        self.request(
            idx,
            RGB::named(BLUE),
            RGB::named(BLACK),
            to_cp437('?'),
            200.0,
            level,
          );
    }
}
