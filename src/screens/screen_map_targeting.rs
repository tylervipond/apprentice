use super::ui::ui_hud::UIHud;
use super::ui::ui_map::UIMap;
use super::ui::ui_mouse_pos::UIMousePos;
use super::utils::{get_render_data, get_render_offset};
use crate::components::{CombatStats, Position, Viewshed};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::ranged;
use crate::services::GameLog;
use crate::ui_components::{Style, UITextLine};
use rltk::{Rltk, BLACK, BLUE, CYAN, RGB, YELLOW};
use specs::{Entity, World, WorldExt};

pub struct ScreenMapTargeting {
    range: u32,
    target: Option<usize>,
    cta: Option<String>,
}

impl ScreenMapTargeting {
    pub fn new(range: u32, target: Option<usize>, cta: Option<String>) -> Self {
        Self { range, target, cta }
    }
    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        ctx.cls();
        let log = world.fetch::<GameLog>();
        let player_ent = world.fetch::<Entity>();
        let combat_stats = world.read_storage::<CombatStats>();
        let player_stats = combat_stats.get(*player_ent).unwrap();
        let positions = world.read_storage::<Position>();
        let player_position = positions.get(*player_ent).unwrap();
        let viewsheds = world.read_storage::<Viewshed>();
        let player_viewshed = viewsheds.get(*player_ent).unwrap();
        let dungeon = world.fetch::<Dungeon>();
        let level = dungeon.get_level(player_position.level).unwrap();
        let render_data = get_render_data(world);
        let (center_x, center_y) = level_utils::idx_xy(level.width as u32, player_position.idx);
        let render_offset = get_render_offset(center_x, center_y);

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
        if let Some(cta) = &self.cta {
            UITextLine::new(
                1,
                0,
                &cta,
                Some(Style {
                    fg: YELLOW,
                    bg: BLACK,
                }),
            )
            .draw(ctx);
        }
        let visible_tiles = ranged::get_visible_tiles_in_range(world, self.range);
        visible_tiles.iter().for_each(|tile| {
            let (x, y) = level_utils::idx_xy(level.width as u32, *tile);
            ctx.set_bg(x - render_offset.0, y - render_offset.1, RGB::named(BLUE));
        });
        let (mouse_x, mouse_y) = ctx.mouse_pos();
        UIMousePos::new(mouse_x, mouse_y).draw(ctx);
        if let Some(target) = self.target {
            let (x, y) = level_utils::idx_xy(level.width as u32, target);
            ctx.set_bg(x - render_offset.0, y - render_offset.1, RGB::named(CYAN))
        }
    }
}
