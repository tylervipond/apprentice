use std::fs::{read_to_string, remove_file, File};
use std::path::Path;
use specs::World;
use super::common::{load_game_from_string, save_game_with_writer};

const SAVE_FILE_PATH: &str = "./tell-lands-save.json";

pub fn load_game(world: &mut World) {
    let game_string = read_to_string(SAVE_FILE_PATH).unwrap();
    load_game_from_string(world, game_string);
}

pub fn has_save_game() -> bool {
    Path::new(SAVE_FILE_PATH).exists()
}

pub fn delete_save() {
    if has_save_game() {
        remove_file(SAVE_FILE_PATH).expect("unable to delete save file")
    }
}

pub fn save_game(world: &mut World) {
    let writer = File::create(SAVE_FILE_PATH).unwrap();
    save_game_with_writer(world, writer);
}