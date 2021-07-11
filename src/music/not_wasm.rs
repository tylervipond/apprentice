use rodio::{OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Music {
    sink: Sink,
    stream: OutputStream,
}

impl Music {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let (queue_input, queue_output) = rodio::queue::queue(false);
        let dungeon_music_r2_file = File::open("resources/dungeon_music_r2.mp3").unwrap();
        let dungeon_music = rodio::Decoder::new(BufReader::new(dungeon_music_r2_file)).unwrap();
        let marching_music_file = File::open("resources/marching_music.mp3").unwrap();
        let marching_music = rodio::Decoder::new(BufReader::new(marching_music_file)).unwrap();
        let ambient_music_file = File::open("resources/app_amb1.mp3").unwrap();
        let ambient_music = rodio::Decoder::new(BufReader::new(ambient_music_file)).unwrap();
        let apprentice_4_music_file = File::open("resources/apprentice4.mp3").unwrap();
        let apprentice_4_music = rodio::Decoder::new(BufReader::new(apprentice_4_music_file)).unwrap();

        queue_input.append(dungeon_music);
        queue_input.append(marching_music);
        queue_input.append(ambient_music);
        queue_input.append(apprentice_4_music);

        let queue_output = queue_output.repeat_infinite();
        sink.append(queue_output);
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
