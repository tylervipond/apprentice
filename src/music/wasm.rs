use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/audio.js")]
extern "C" {
    fn setupAudio();
    fn playAudio();
    fn pauseAudio();
}

pub struct Music {}

impl Music {
    pub fn new() -> Self {
        setupAudio();
        Music {}
    }

    pub fn play_music(&mut self) {
        playAudio();
    }
    pub fn pause_music(&mut self) {
        pauseAudio();
    }
}
