use web_sys::HtmlAudioElement;
pub struct Music {
    audio: HtmlAudioElement
}

impl Music {
    pub fn new() -> Self {
        let mut audio = HtmlAudioElement::new_with_src("resources/dungeon_music.mp3").expect("couldn't create music");
        audio.set_loop(true);
        Self {
            audio
        }
    }

    pub fn play_music(&mut self) {
        self.audio.play().expect("couldn't play music");
    }
    pub fn pause_music(&mut self) {
        self.audio.pause().expect("couldn't pause music");
    }
}
