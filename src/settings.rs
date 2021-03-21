use crate::{
    control::ControlMap,
    user_actions::{
        InteractionTargetingAction, MapAction, MenuAction, StaticAction, TargetingAction,
    },
};
use ron::from_str;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    env,
    fs::{DirBuilder, File},
    io::{Read, Write},
};
#[cfg(target_arch = "wasm32")]
use web_sys;

const DEFAULT_CONTROLS_STRING: &str = include_str!("./default_settings/controls.ron");

#[derive(Serialize, Deserialize)]
pub struct ControlScheme {
    pub map: ControlMap<MapAction>,
    pub menu: ControlMap<MenuAction>,
    pub horizontal_menu: ControlMap<MenuAction>,
    pub interaction: ControlMap<InteractionTargetingAction>,
    pub static_screen: ControlMap<StaticAction>,
    pub targeting: ControlMap<TargetingAction>,
}

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
fn get_settings_dir() -> String {
    let home_dir = env::var("APPDATA").unwrap();
    format!("{}/Apprentice", home_dir)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_settings_dir() -> String {
    let home_dir = env::var("HOME").unwrap();
    format!("{}/Library/Application Support/apprentice", home_dir)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_settings_filepath() -> String {
    format!("{}/key-bindings.ron", get_settings_dir())
}

impl ControlScheme {
    pub fn default() -> Self {
        from_str::<Self>(DEFAULT_CONTROLS_STRING).unwrap()
    }

    fn as_ron_string(&self) -> String {
        let my_config = PrettyConfig::new()
            .with_depth_limit(4)
            .with_indentor("\t".to_owned());
        to_string_pretty(self, my_config).unwrap()
    }

    fn from_ron_string(controls_str: &str) -> Self {
        match from_str::<Self>(controls_str) {
            Ok(controls) => controls,
            _ => Self::default(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        DirBuilder::new()
            .recursive(true)
            .create(get_settings_dir())
            .expect("failed to create settings dir");
        let mut file =
            File::create(get_settings_filepath()).expect("failed to create settings file");
        file.write_all(self.as_ron_string().as_bytes())
            .expect("failed to populate settings file");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let window = web_sys::window().expect("no global `window` exists");
        let storage = window.local_storage().unwrap().expect("no local storage");
        storage
            .set_item("key-bindings.ron", self.as_ron_string().as_str())
            .expect("could not write to local storage");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        if let Ok(mut settings_file) = File::open(get_settings_filepath()) {
            let mut settings = String::new();
            if let Ok(_) = settings_file.read_to_string(&mut settings) {
                return Self::from_ron_string(settings.as_str());
            }
        }
        Self::default()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let storage = window.local_storage().unwrap().expect("no local storage");
        match storage.get_item("key-bindings.ron") {
            Ok(r) => match r {
                Some(controls_string) => Self::from_ron_string(controls_string.as_str()),
                _ => Self::default(),
            },
            _ => Self::default(),
        }
    }
}
pub struct Settings {
    pub control_scheme: ControlScheme,
}

impl Settings {
    pub fn load() -> Self {
        Settings {
            control_scheme: ControlScheme::load(),
        }
    }
    pub fn save(&self) {
        self.control_scheme.save();
    }
}
