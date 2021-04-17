use rodio::{OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Music {
    sink: Sink,
    stream: OutputStream
}

impl Music {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let file = File::open("resources/dungeon_music_r1.mp3").unwrap();
        let source = rodio::Decoder::new(BufReader::new(file))
            .unwrap()
            .repeat_infinite();
        sink.append(source);
        sink.pause();
        Self { sink, stream }
    }

    pub fn play_music(&mut self) {
       self.sink.play();
    }

    pub fn pause_music(&mut self) {
        self.sink.pause();
    }
}
