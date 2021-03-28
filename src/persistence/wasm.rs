use std::str;
use specs::World;
use wasm_bindgen::prelude::*;
use super::common::{save_game_with_writer, load_game_from_string};


#[wasm_bindgen(module = "/js/persistence.js")]
extern "C" {
    fn save_game_data(data: &str);
    fn delete_game_data();
    fn has_game_data() -> bool;
    fn load_game_data() -> String;
}


pub fn save_game(world: &mut World) {
    let writer = Vec::<u8>::new();
    let serializer = save_game_with_writer(world, writer);
    let serializer = serializer.into_inner();
    let save_str = str::from_utf8(&serializer).unwrap();
    save_game_data(save_str);
}

pub fn delete_save() {
    delete_game_data();
}

pub fn load_game(world: &mut World) {
    let game_string = load_game_data();
    load_game_from_string(world, game_string)
}

pub fn has_save_game() -> bool {
    has_game_data()
}
