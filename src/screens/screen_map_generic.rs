use super::ui::ui_map::RenderData;
use super::ui::ui_map_screen::UIMapScreen;
use crate::components::{
    combat_stats::CombatStats, dungeon_level::DungeonLevel, hidden::Hidden, name::Name,
    position::Position, renderable::Renderable,
};
use crate::dungeon::{dungeon::Dungeon, operations::xy_idx};
use crate::game_log::GameLog;
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
        let tool_tip_lines = match level
            .visible_tiles
            .get(xy_idx(&level, mouse_x, mouse_y) as usize)
        {
            Some(visible) => match visible {
                true => (&names, &positions, &levels, !&hidden).join().fold(
                    Vec::new(),
                    |mut acc, (name, position, level, _)| {
                        if level.level == player_level.level
                            && position.x == mouse_x
                            && position.y == mouse_y
                        {
                            acc.push(name.name.to_owned());
                        }
                        acc
                    },
                ),
                false => Vec::new(),
            },
            None => Vec::new(),
        };
        let renderables = world.read_storage::<Renderable>();

        let mut renderables = (&positions, &renderables, &levels, !&hidden)
            .join()
            .filter(|(p, _r, l, _h)| {
                let idx = xy_idx(&level, p.x, p.y) as usize;
                return l.level == player_level.level && level.visible_tiles[idx];
            })
            .map(|(p, r, _l, _h)| RenderData {
                x: p.x,
                y: p.y,
                fg: r.fg,
                bg: r.bg,
                glyph: r.glyph,
                layer: r.layer,
            })
            .collect::<Vec<RenderData>>();
        renderables.sort_unstable_by(|a, b| b.layer.cmp(&a.layer));
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
            &renderables,
        )
        .draw(ctx);
    }
}
