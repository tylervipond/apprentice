use super::ui::ui_map_screen::UIMapScreen;
use super::utils::{get_render_data, get_render_offset, get_render_offset_for_xy};
use crate::components::{CombatStats, Hidden, Hiding, Name, Position, Viewshed};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::services::GameLog;
use rltk::Rltk;
use specs::{Entity, Join, World, WorldExt};

pub struct ScreenMapGeneric {
    offset_x: i32,
    offset_y: i32,
}

impl ScreenMapGeneric {
    pub fn new(offset_x: i32, offset_y: i32) -> Self {
        Self { offset_x, offset_y }
    }

    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        let log = world.fetch::<GameLog>();
        let player_ent = world.fetch::<Entity>();
        let combat_stats = world.read_storage::<CombatStats>();
        let player_stats = combat_stats.get(*player_ent).unwrap();
        let hiding = world.read_storage::<Hiding>();
        let entities = world.entities();
        let names = world.read_storage::<Name>();
        let positions = world.read_storage::<Position>();
        let hidden = world.read_storage::<Hidden>();
        let (mouse_x, mouse_y) = ctx.mouse_pos();
        let dungeon = world.fetch::<Dungeon>();
        let player_position = positions.get(*player_ent).unwrap();
        let viewsheds = world.read_storage::<Viewshed>();
        let player_viewshed = viewsheds.get(*player_ent).unwrap();
        let level = dungeon.get_level(player_position.level).unwrap();
        let level_width = level.width as u32;
        let (center_x, center_y) = level_utils::idx_xy(level_width, player_position.idx);
        let center_x = center_x + self.offset_x;
        let center_y = center_y + self.offset_y;
        let render_offset = get_render_offset(center_x, center_y);
        let mouse_offset = get_render_offset_for_xy(center_x, center_y, mouse_x, mouse_y);
        let mouse_idx = level_utils::xy_idx(level_width, mouse_offset.0, mouse_offset.1);
        let tool_tip_lines: Box<[String]> = match player_viewshed.visible_tiles.contains(&mouse_idx)
        {
            true => (
                &names,
                &positions,
                (&hidden).maybe(),
                (&hiding).maybe(),
                &entities,
            )
                .join()
                .filter(|(_name, position, hidden, hiding, entity)| {
                    let visible_to_player = match hidden {
                        Some(h) => h.found_by.contains(&*player_ent),
                        None => true,
                    };
                    let hiding = match hiding {
                        Some(_) => *entity != *player_ent,
                        None => false,
                    };
                    visible_to_player
                        && !hiding
                        && position.level == player_position.level
                        && position.idx == mouse_idx as usize
                })
                .map(|(name, _position, _hidden, hiding, _entity)| match hiding {
                    Some(_) => format!("{} (hidden)", name.name),
                    _ => name.name.clone(),
                })
                // .map(|s| s.clone())
                .collect(),
            false => Box::new([]),
        };
        let tool_tip_lines: Box<[&str]> = tool_tip_lines.iter().map(|line| line.as_str()).collect();
        let render_data = get_render_data(world);
        let log_entries = log.entries.iter().map(String::as_str).collect();
        ctx.cls();
        UIMapScreen::new(
            mouse_x,
            mouse_y,
            &tool_tip_lines,
            &log_entries,
            player_position.level,
            player_stats.hp,
            player_stats.max_hp,
            level,
            &render_data,
            render_offset,
            &player_viewshed.visible_tiles
        )
        .draw(ctx);
    }
}
