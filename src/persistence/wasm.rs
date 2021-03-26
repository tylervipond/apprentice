use super::common::{load_game_from_string, save_game_with_writer};
use specs::World;
use std::str;
use web_sys::Storage;

const SAVE_FILE_PATH: &str = "tell-lands-save";
// as the game gets bigger, this will need to be split into more and more parts
// to ensure we don't get past the JS localstorage quota
const SAVE_PARTS_COUNT: usize = 4;

fn get_save_path(save_part_number: usize) -> String {
    format!("{}-{}", SAVE_FILE_PATH, save_part_number)
}

fn get_local_storage() -> Storage {
    let window = web_sys::window().expect("no global `window` exists");
    window.local_storage().unwrap().expect("no local storage")
}

pub fn save_game(world: &mut World) {
    let writer = Vec::<u8>::new();
    let serializer = save_game_with_writer(world, writer);
    let storage = get_local_storage();
    let serializer = serializer.into_inner();
    let save_str = str::from_utf8(&serializer).unwrap();
    let mut save_str = String::from(save_str);
    let save_size = save_str.chars().count() / SAVE_PARTS_COUNT;
    for i in 0..=SAVE_PARTS_COUNT {
        let amount_to_drain = usize::min(save_str.chars().count(), save_size);
        let value: String = save_str.drain(0..amount_to_drain).collect();
        let key = get_save_path(i);
        storage
            .set_item(&key, &value)
            .expect(&format!("could not write {} to local storage", key));
    }
}

pub fn delete_save() {
    if has_save_game() {
        let storage = get_local_storage();
        for i in 0..=SAVE_PARTS_COUNT {
            let key = get_save_path(i);
            storage
                .remove_item(&key)
                .expect(&format!("couldn't delete file {}", key));
        }
    }
}

pub fn load_game(world: &mut World) {
    let storage = get_local_storage();
    let mut save_string = String::new();
    for i in 0..=SAVE_PARTS_COUNT {
        let key = get_save_path(i);
        match storage.get_item(&key) {
            Ok(r) => match r {
                Some(save_string_part) => save_string.push_str(&save_string_part),
                _ => (),
            },
            _ => (),
        }
    }
    load_game_from_string(world, save_string)
}

pub fn has_save_game() -> bool {
    let storage = get_local_storage();
    for i in 0..=SAVE_PARTS_COUNT {
        let key = get_save_path(i);
        let file_found = match storage.get_item(&key) {
            Ok(r) => match r {
                Some(_) => true,
                _ => false,
            },
            _ => false,
        };
        if !file_found {
            return false;
        }
    }
    true
}
