use std::fmt::Display;

use super::ui::{ui_hud::UIHud, ui_map::UIMap};
use super::utils::{get_render_data, get_render_offset};
use crate::components::{CombatStats, Position, Viewshed};
use crate::dungeon::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    dungeon::Dungeon,
    level_utils,
};
use crate::menu::MenuOption;
use crate::services::GameLog;
use crate::ui_components::{UILineVertical, UIMenuBox, UIMenuItemGroup, UIParagraph};
use rltk::Rltk;
use specs::{Entity, World, WorldExt};

const SCREEN_PADDING: u8 = 4;

pub struct ScreenMapNestedMenu<'a, T: ToString, S: ToString> {
    menu_options: Box<[&'a MenuOption<T>]>,
    sub_menu_options: Box<[&'a MenuOption<S>]>,
    sub_menu_active: bool,
    description: &'a str,
    title: &'a str,
    cta: &'a str,
}

impl<'a, T: ToString, S: ToString> ScreenMapNestedMenu<'a, T, S> {
    pub fn new(
        menu_options: Box<[&'a MenuOption<T>]>,
        sub_menu_options: Box<[&'a MenuOption<S>]>,
        sub_menu_active: bool,
        description: &'a str,
        title: &'a str,
        cta: &'a str,
    ) -> Self {
        Self {
            menu_options,
            sub_menu_options,
            sub_menu_active,
            description,
            title,
            cta,
        }
    }

    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        ctx.cls();
        let log = world.fetch::<GameLog>();
        let player_ent = world.fetch::<Entity>();
        let positions = world.read_storage::<Position>();
        let player_position = positions.get(*player_ent).unwrap();
        let combat_stats = world.read_storage::<CombatStats>();
        let player_stats = combat_stats.get(*player_ent).unwrap();

        let dungeon = world.fetch::<Dungeon>();
        let level = dungeon.levels.get(&player_position.level).unwrap();
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
        let log_entries = log.entries.iter().map(String::as_str).collect();

        UIHud::new(
            player_position.level,
            player_stats.hp,
            player_stats.max_hp,
            &log_entries,
        )
        .draw(ctx);
        let mut menu = UIMenuItemGroup::new(0, 0, &self.menu_options, !self.sub_menu_active);
        let mut sub_menu = UIMenuItemGroup::new(0, 0, &self.sub_menu_options, self.sub_menu_active);
        let description_box_size =
            MAP_WIDTH as u32 - menu.width - sub_menu.width - 6 - (SCREEN_PADDING as u32 + 1) * 2;
        let mut description_box = UIParagraph::new(0, 0, description_box_size, self.description);
        let height = [menu.height, sub_menu.height, description_box.height]
            .iter()
            .max()
            .unwrap()
            + 2;
        let y = (MAP_HEIGHT as u32 / 2 - height / 2) as i32;
        menu.y = y;
        menu.x = SCREEN_PADDING as i32 + 1;
        sub_menu.x = menu.x + menu.width as i32 + 3;
        sub_menu.y = y;
        description_box.x = sub_menu.x + sub_menu.width as i32 + 3;
        description_box.y = y;

        UIMenuBox::new(
            SCREEN_PADDING as i32,
            y - 1,
            MAP_WIDTH - SCREEN_PADDING * 2,
            height as u8 + 2,
            Some(self.cta),
            Some(self.title),
        )
        .draw(ctx);

        menu.draw(ctx);
        sub_menu.draw(ctx);
        description_box.draw(ctx);
        UILineVertical::new(sub_menu.x - 2, y, height).draw(ctx);
        UILineVertical::new(description_box.x - 2, y, height).draw(ctx);
    }
}
