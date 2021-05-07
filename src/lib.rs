use rltk::RltkBuilder;
use wasm_bindgen::prelude::*;
#[macro_use]
extern crate specs_derive;
extern crate serde;
mod ai;
mod artwork;
mod components;
mod control;
mod copy;
#[cfg(debug_assertions)]
mod debug;
mod dungeon;
mod entity_option;
mod entity_set;
mod interaction_type;
mod inventory;
mod menu;
mod music;
mod patches;
mod persistence;
mod player;
mod ranged;
mod run_state;
mod screens;
mod services;
mod settings;
mod spawner;
mod state;
mod systems;
mod types;
mod ui_components;
mod user_actions;
mod utils;
mod world_utils;
// mod states;

use music::Music;
use run_state::RunState;
use screens::{SCREEN_HEIGHT, SCREEN_WIDTH};
use settings::Settings;
use state::State;

#[wasm_bindgen]
pub fn start() {
    let gs = State {
        world: world_utils::get_world(),
        run_state: RunState::MainMenu { highlighted: 0 },
        queued_action: None,
        settings: Settings::load(),
        music: Music::new(),
    };
    let context = RltkBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap()
        .with_title("Apprentice")
        .with_advanced_input(true)
        .with_fullscreen(false)
        .build()
        .expect("failed to create context");
    rltk::main_loop(context, gs).expect("failed to start apprentice");
}
