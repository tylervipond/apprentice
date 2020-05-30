use super::ui::ui_map_screen::UIMapScreen;
use super::utils::{get_render_data, get_render_offset};
use crate::components::{CombatStats, DungeonLevel, Hidden, Name, Position};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::services::GameLog;
use rltk::Rltk;
use specs::{Entity, Join, World, WorldExt};

pub struct ScreenMapGeneric {}

impl ScreenMapGeneric {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        let log = world.fetch::<GameLog>();
        let player_ent = world.fetch::<Entity>();
        let levels = world.read_storage::<DungeonLevel>();
        let player_level = levels.get(*player_ent).unwrap();
        let combat_stats = world.read_storage::<CombatStats>();
        let player_stats = combat_stats.get(*player_ent).unwrap();

        let names = world.read_storage::<Name>();
        let positions = world.read_storage::<Position>();
        let hidden = world.read_storage::<Hidden>();
        let (mouse_x, mouse_y) = ctx.mouse_pos();
        let dungeon = world.fetch::<Dungeon>();
        let level = dungeon.levels.get(&player_level.level).unwrap();
        let render_offset = get_render_offset(world);
        let tool_tip_lines = match level
            .visible_tiles
            .get(level_utils::xy_idx(&level, mouse_x + render_offset.0, mouse_y + render_offset.1) as usize)
        {
            Some(visible) => match visible {
                true => (&names, &positions, &levels, (&hidden).maybe())
                    .join()
                    .filter(|(_name, position, level, hidden)| {
                        let visible_to_player = match hidden {
                            Some(h) => h.found_by.contains(&*player_ent),
                            None => true,
                        };
                        visible_to_player
                            && level.level == player_level.level
                            && position.x - render_offset.0 == mouse_x
                            && position.y - render_offset.1 == mouse_y
                    })
                    .map(|(name, _position, _level, _hidden)| name.name.to_owned())
                    .collect(),
                false => Vec::new(),
            },
            None => Vec::new(),
        };
        let render_data = get_render_data(world);
        ctx.cls();
        UIMapScreen::new(
            mouse_x,
            mouse_y,
            &tool_tip_lines,
            &log.entries,
            player_level.level,
            player_stats.hp,
            player_stats.max_hp,
            level,
            &render_data,
            render_offset,
        )
        .draw(ctx);
    }
}
