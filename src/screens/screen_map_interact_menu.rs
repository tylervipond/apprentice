use std::fmt::Display;

use super::ui::ui_map::UIMap;
use super::utils::{get_render_data, get_render_offset};
use crate::components::{Position, Viewshed};
use crate::dungeon::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    dungeon::Dungeon,
    level_utils,
};
use crate::menu::MenuOption;
use crate::ui_components::ui_dynamic_menu::UIDynamicMenu;
use rltk::Rltk;
use specs::{Entity, World, WorldExt};

pub struct ScreenMapInteractMenu<'a, T: Display + Copy> {
    menu_options: Box<[&'a MenuOption<T>]>,
    title: Option<&'a str>,
    cta: Option<&'a str>,
}

impl<'a, T: Display + Copy> ScreenMapInteractMenu<'a, T> {
    pub fn new(
        menu_options: Box<[&'a MenuOption<T>]>,
        title: Option<&'a str>,
        cta: Option<&'a str>,
    ) -> Self {
        Self {
            menu_options,
            title,
            cta,
        }
    }

    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        ctx.cls();
        let player_ent = world.fetch::<Entity>();
        let positions = world.read_storage::<Position>();
        let player_position = positions.get(*player_ent).unwrap();

        let dungeon = world.fetch::<Dungeon>();
        let level = dungeon.get_level(player_position.level).unwrap();
        let render_data = get_render_data(world);
        let positions = world.read_storage::<Position>();
        let player_position = positions.get(*player_ent).unwrap();
        let (center_x, center_y) = level_utils::idx_xy(level.width as u32, player_position.idx);
        let render_offset = get_render_offset(center_x, center_y);
        let viewsheds = world.read_storage::<Viewshed>();
        let player_viewshed = viewsheds.get(*player_ent).unwrap();
        UIMap::new(
            level,
            &render_data,
            render_offset,
            &player_viewshed.visible_tiles,
        )
        .draw(ctx);

        let mut menu = UIDynamicMenu::new(0, 0, &self.menu_options, self.cta, self.title);
        menu.y = (MAP_HEIGHT / 2 - menu.height / 2) as i32;
        menu.x = (MAP_WIDTH / 2 - menu.width / 2) as i32;
        menu.draw(ctx);
    }
}
